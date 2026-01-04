//! Application controller and coordination layer
//!
//! The controller orchestrates between input, domain, UI, and platform layers.
//! It maintains stable configuration (grids, monitors) and handles state transitions.

use crate::app::state::{ AppState, StateEvent, StateMachine };
use crate::domain::grid::Grid;
use crate::input::{ HotkeyError, HotkeyManager, HotkeyModifier, VirtualKey };
use crate::input::{ KeyEvent, KeyboardCaptureError, KeyboardCaptureGuard };
use crate::platform::monitors::{ enumerate_monitors, Monitor, MonitorError };
use crate::ui::{ OverlayError, OverlayManager };
use std::sync::{ Arc, Mutex };
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW,
    MSG,
    PM_REMOVE,
    PeekMessageW,
    TranslateMessage,
    WM_QUIT,
};

/// Application errors that can occur during controller operations
#[derive(Debug)]
pub enum AppError {
    /// Monitor enumeration failed
    MonitorError(MonitorError),
    /// Failed to create grids for monitors
    GridCreationFailed(String),
    /// No suitable monitors found for grid positioning
    NoSuitableMonitors,
    /// Hotkey management failed
    HotkeyError(HotkeyError),
    /// Overlay management failed
    OverlayError(OverlayError),
    /// Keyboard capture failed
    KeyboardCaptureError(KeyboardCaptureError),
}

impl From<MonitorError> for AppError {
    fn from(err: MonitorError) -> Self {
        AppError::MonitorError(err)
    }
}

impl From<HotkeyError> for AppError {
    fn from(err: HotkeyError) -> Self {
        AppError::HotkeyError(err)
    }
}

impl From<OverlayError> for AppError {
    fn from(err: OverlayError) -> Self {
        AppError::OverlayError(err)
    }
}

impl From<KeyboardCaptureError> for AppError {
    fn from(err: KeyboardCaptureError) -> Self {
        AppError::KeyboardCaptureError(err)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::MonitorError(e) => write!(f, "Monitor error: {:?}", e),
            AppError::GridCreationFailed(msg) => write!(f, "Grid creation failed: {}", msg),
            AppError::NoSuitableMonitors => write!(f, "No suitable monitors for grid positioning"),
            AppError::HotkeyError(e) => write!(f, "Hotkey error: {:?}", e),
            AppError::OverlayError(e) => write!(f, "Overlay error: {:?}", e),
            AppError::KeyboardCaptureError(e) => write!(f, "Keyboard capture error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

/// RAII wrapper for global hotkey management
///
/// Automatically starts/stops the hotkey manager and manages hotkey registration.
/// Provides thread-safe access to hotkey functionality.
pub struct HotkeyManagerGuard {
    manager: HotkeyManager,
    main_hotkey_id: Option<u32>,
}

impl HotkeyManagerGuard {
    /// Create a new hotkey manager and register the main application hotkey
    pub fn new() -> Result<Self, AppError> {
        println!("HotkeyManagerGuard: Creating new hotkey manager...");
        let mut manager = HotkeyManager::new();

        // Start the message loop
        println!("HotkeyManagerGuard: Starting hotkey manager...");
        manager.start()?;
        println!("HotkeyManagerGuard: Hotkey manager started successfully");

        Ok(Self {
            manager,
            main_hotkey_id: None,
        })
    }

    /// Register the main application hotkey
    pub fn register_main_hotkey<F>(
        &mut self,
        modifiers: &[HotkeyModifier],
        key: VirtualKey,
        callback: F
    ) -> Result<(), AppError>
        where F: Fn() + Send + Sync + 'static
    {
        let hotkey_id = self.manager.register_hotkey(modifiers, key, Arc::new(callback))?;

        self.main_hotkey_id = Some(hotkey_id);
        Ok(())
    }

    /// Check if the hotkey manager is running
    pub fn is_running(&self) -> bool {
        self.manager.is_running()
    }
}

impl Drop for HotkeyManagerGuard {
    fn drop(&mut self) {
        // Unregister main hotkey if registered
        if let Some(id) = self.main_hotkey_id {
            let _ = self.manager.unregister_hotkey(id);
        }

        // Manager will be automatically stopped by its own Drop implementation
    }
}

/// RAII wrapper for overlay management
///
/// Automatically manages overlay windows and their lifecycle.
/// Provides thread-safe access to overlay functionality.
pub struct OverlayManagerGuard {
    manager: OverlayManager,
}

impl OverlayManagerGuard {
    /// Create a new overlay manager and initialize with monitors and grids
    pub fn new(monitors: &[Monitor], grids: &[Grid]) -> Result<Self, AppError> {
        let mut manager = OverlayManager::new();

        // Initialize overlay windows for all monitors with their grids
        manager.initialize(monitors, grids)?;

        Ok(Self { manager })
    }

    /// Show overlays on all monitors
    pub fn show_all(&mut self) {
        self.manager.show_all();
    }

    /// Hide overlays on all monitors
    pub fn hide_all(&mut self) {
        self.manager.hide_all();
    }

    /// Toggle overlay visibility
    pub fn toggle(&mut self) {
        self.manager.toggle();
    }

    /// Check if overlays are visible
    pub fn is_visible(&self) -> bool {
        self.manager.is_visible()
    }

    /// Get overlay count
    pub fn overlay_count(&self) -> usize {
        self.manager.overlay_count()
    }

    /// Set which monitor is active (shows letters)
    pub fn set_active_monitor(&mut self, monitor_index: usize) {
        self.manager.set_active_monitor(monitor_index);
    }

    /// Get the currently active monitor
    pub fn get_active_monitor(&self) -> Option<usize> {
        self.manager.get_active_monitor()
    }

    /// Render grid content for all overlays
    pub fn render_grids(&mut self) {
        self.manager.render_all_grids();
    }
}

impl Drop for OverlayManagerGuard {
    fn drop(&mut self) {
        // Hide all overlays before cleanup
        self.manager.hide_all();
    }
}

/// RAII wrapper for keyboard capture
///
/// Manages keyboard input capture during modal selection mode.
/// Automatically starts/stops capture based on application state.
pub struct KeyboardCaptureManager {
    capture: Option<KeyboardCaptureGuard>,
    main_window: HWND,
}

impl KeyboardCaptureManager {
    pub fn new(main_window: HWND) -> Self {
        Self {
            capture: None,
            main_window,
        }
    }

    /// Start keyboard capture
    pub fn start_capture(&mut self) -> Result<(), KeyboardCaptureError> {
        if self.capture.is_none() {
            let guard = KeyboardCaptureGuard::new(self.main_window)?;
            self.capture = Some(guard);
        }
        Ok(())
    }

    /// Stop keyboard capture
    pub fn stop_capture(&mut self) {
        self.capture = None;
    }

    /// Check if currently capturing
    pub fn is_capturing(&self) -> bool {
        self.capture.as_ref().map_or(false, |c| c.is_capturing())
    }

    /// Get the message ID for keyboard events
    pub fn message_id() -> u32 {
        KeyboardCaptureGuard::message_id()
    }

    /// Parse a keyboard message
    pub fn parse_message(wparam: windows::Win32::Foundation::WPARAM) -> Option<KeyEvent> {
        KeyboardCaptureGuard::parse_message(wparam)
    }
}

impl Drop for KeyboardCaptureManager {
    fn drop(&mut self) {
        self.stop_capture();
    }
}

/// Main application controller
///
/// Coordinates between all components and maintains stable configuration.
/// The state is thread-safe and can be shared between components.
pub struct AppController {
    /// Current application state (thread-safe)
    state: Arc<Mutex<AppState>>,
    /// Hotkey management with guaranteed cleanup
    hotkey_manager: HotkeyManagerGuard,
    /// Overlay management with guaranteed cleanup
    overlay_manager: OverlayManagerGuard,
    /// Keyboard capture management
    keyboard_capture: KeyboardCaptureManager,
    /// Available monitors (stable configuration)
    monitors: Vec<Monitor>,
    /// Grid instances per monitor (stable configuration)
    grids: Vec<Grid>,
    /// Main window handle for message processing
    main_window: HWND,
}

impl AppController {
    /// Creates a new application controller
    ///
    /// # Arguments
    /// * `main_window` - Main window handle for message processing
    ///
    /// # Returns
    /// AppController instance or AppError if initialization fails
    pub fn new(main_window: HWND) -> Result<Self, AppError> {
        // Initialize monitors using Phase 1 infrastructure
        let monitors = enumerate_monitors()?;
        if monitors.is_empty() {
            return Err(AppError::NoSuitableMonitors);
        }

        // Create grids for each monitor using Phase 2 domain logic
        let mut grids = Vec::new();
        for (i, monitor) in monitors.iter().enumerate() {
            match Grid::new(2, 3, monitor.work_area) {
                // 2 rows, 3 columns (Q W E / A S D)
                Ok(grid) => {
                    grids.push(grid);
                    println!(
                        "Monitor {}: Created 2x3 grid (2 rows, 3 cols) for {}x{} area",
                        i,
                        monitor.work_area.w,
                        monitor.work_area.h
                    );
                }
                Err(e) => {
                    return Err(AppError::GridCreationFailed(format!("Monitor {}: {:?}", i, e)));
                }
            }
        }

        if grids.is_empty() {
            return Err(AppError::NoSuitableMonitors);
        }

        // Initialize RAII-wrapped components
        // TEMPORARY: Skip hotkey manager for debugging overlay rendering
        println!("AppController: Skipping hotkey registration for debugging");
        let hotkey_manager = HotkeyManagerGuard::new()?;
        let mut overlay_manager = OverlayManagerGuard::new(&monitors, &grids)?;
        let keyboard_capture = KeyboardCaptureManager::new(main_window);

        // TEMPORARY: Start in selecting mode to immediately show overlays
        println!("AppController: Starting in SELECTING mode for debugging");
        let initial_state = AppState::Selecting(crate::app::state::SelectingState::new(0));
        let state = Arc::new(Mutex::new(initial_state));

        // Show overlays immediately
        println!("AppController: Showing overlays at startup for verification");
        overlay_manager.set_active_monitor(0);
        overlay_manager.show_all();
        // TEMPORARILY DISABLED: render_grids() uses UpdateLayeredWindow which conflicts with SetLayeredWindowAttributes
        // overlay_manager.render_grids();
        println!("AppController: Overlays should now be visible on all monitors");

        // Start keyboard capture for user input
        let mut kb_capture = KeyboardCaptureManager::new(main_window);
        if let Err(e) = kb_capture.start_capture() {
            eprintln!("Failed to start keyboard capture: {}", e);
            return Err(AppError::from(e));
        }
        println!("AppController: Keyboard capture started - ready for input");
        let keyboard_capture = kb_capture;

        Ok(Self {
            state,
            hotkey_manager,
            overlay_manager,
            keyboard_capture,
            monitors,
            grids,
            main_window,
        })
    }

    /// Gets the current application state (thread-safe)
    ///
    /// # Returns
    /// Copy of current application state
    pub fn get_state(&self) -> AppState {
        self.state.lock().unwrap().clone()
    }

    /// Gets the number of available monitors
    ///
    /// # Returns
    /// Count of monitors with valid grids
    pub fn monitor_count(&self) -> usize {
        self.monitors.len()
    }

    /// Gets a reference to a specific monitor
    ///
    /// # Arguments
    /// * `index` - Monitor index
    ///
    /// # Returns
    /// Monitor reference or None if index is invalid
    pub fn get_monitor(&self, index: usize) -> Option<&Monitor> {
        self.monitors.get(index)
    }

    /// Gets a reference to a specific grid
    ///
    /// # Arguments
    /// * `index` - Grid/monitor index
    ///
    /// # Returns
    /// Grid reference or None if index is invalid
    pub fn get_grid(&self, index: usize) -> Option<&Grid> {
        self.grids.get(index)
    }

    /// Processes a state event using the state machine
    ///
    /// # Arguments
    /// * `event` - Event to process
    ///
    /// # Returns
    /// The new state after processing
    pub fn process_event(&self, event: StateEvent) -> AppState {
        let mut state_guard = self.state.lock().unwrap();
        let current_state = state_guard.clone();
        let new_state = StateMachine::process_event(current_state, event, self.monitor_count());
        *state_guard = new_state.clone();
        new_state
    }

    /// Handle state transition side effects (must be called after process_event)
    pub fn handle_state_transition(&mut self, old_state: &AppState, new_state: &AppState) {
        match (old_state, new_state) {
            (AppState::Idle, AppState::Selecting(_)) => {
                println!("CONTROLLER: Transitioning to Selecting state - showing overlays");
                // Show overlays when entering selection mode
                self.overlay_manager.show_all();
                println!("CONTROLLER: Overlays shown");
            }
            (AppState::Selecting(_), AppState::Idle) => {
                println!("CONTROLLER: Transitioning to Idle state - hiding overlays");
                // Hide overlays when exiting selection mode
                self.overlay_manager.hide_all();
                println!("CONTROLLER: Overlays hidden");
            }
            _ => {
                // No UI changes needed for other transitions
            }
        }
    }

    /// Handles hotkey press events
    ///
    /// Coordinates state transitions with overlay display and keyboard capture.
    pub fn handle_hotkey(&mut self) {
        println!("AppController: Hotkey pressed");
        let new_state = self.process_event(StateEvent::HotkeyPressed);

        match new_state {
            AppState::Idle => {
                println!("Switched to Idle state");
                // Hide overlays and stop keyboard capture
                self.overlay_manager.hide_all();
                self.keyboard_capture.stop_capture();
            }
            AppState::Selecting(ref selecting) => {
                println!(
                    "Switched to Selecting state on monitor {}",
                    selecting.active_monitor_index
                );
                // Show overlays and start keyboard capture
                self.overlay_manager.set_active_monitor(selecting.active_monitor_index);
                self.overlay_manager.show_all();

                // Start keyboard capture
                if let Err(e) = self.keyboard_capture.start_capture() {
                    eprintln!("Failed to start keyboard capture: {}", e);
                    // Fall back to idle on capture failure
                    let _ = self.process_event(StateEvent::SelectionCancelled);
                    self.overlay_manager.hide_all();
                }
            }
        }
    }

    /// Handles key press events during selection
    ///
    /// # Arguments
    /// * `key` - Character key that was pressed
    pub fn handle_key_press(&mut self, key: char) {
        println!("AppController: Key pressed: '{}'", key);

        let current_state = self.get_state();
        if let AppState::Selecting(mut selecting) = current_state {
            // Process key with current grid
            if let Some(grid) = self.get_grid(selecting.active_monitor_index) {
                if grid.contains_key(key) {
                    println!(
                        "Valid grid key: '{}' on monitor {}",
                        key,
                        selecting.active_monitor_index
                    );

                    // Convert key to coordinates
                    if let Ok(coords) = grid.key_to_coords(key) {
                        // Update selection
                        match selecting.selection.add_coords(coords) {
                            Ok(_) => {
                                // Update state with new selection progress
                                let new_state = AppState::Selecting(selecting.clone());
                                *self.state.lock().unwrap() = new_state;
                                
                                // Check if selection is complete
                                if selecting.selection.is_complete() {
                                    println!("Selection completed!");
                                    // Apply selection and return to idle
                                    self.apply_selection();
                                } else {
                                    // Update overlay rendering to show selection progress
                                    self.overlay_manager.render_grids();
                                }
                            }
                            Err(e) => {
                                println!("Selection error: {:?} - cancelling", e);
                                let _ = self.process_event(StateEvent::SelectionCancelled);
                                self.overlay_manager.hide_all();
                                self.keyboard_capture.stop_capture();
                            }
                        }
                    } else {
                        println!("Failed to convert key '{}' to coordinates", key);
                        let _ = self.process_event(StateEvent::SelectionCancelled);
                        self.overlay_manager.hide_all();
                        self.keyboard_capture.stop_capture();
                    }
                } else {
                    println!("Invalid grid key: '{}' (silent ignore)", key);
                }
            }
        } else {
            println!("Key press ignored - not in selecting mode");
        }
    }

    /// Handles navigation events (arrow keys)
    ///
    /// # Arguments
    /// * `direction` - Navigation direction
    pub fn handle_navigation(&mut self, direction: crate::app::state::NavigationDirection) {
        println!("AppController: Navigation: {:?}", direction);

        let new_state = self.process_event(StateEvent::Navigation(direction));
        if let AppState::Selecting(selecting) = new_state {
            println!("Switched to monitor {}", selecting.active_monitor_index);
            // Update overlay rendering to show new active monitor
            self.overlay_manager.set_active_monitor(selecting.active_monitor_index);
            self.overlay_manager.render_grids();
        }
    }

    /// Handles selection timeout (30 seconds)
    ///
    /// Called when selection has been active for 30 seconds without completion.
    pub fn handle_selection_timeout(&mut self) {
        println!("AppController: Selection timed out");
        let new_state = self.process_event(StateEvent::SelectionTimedOut);

        if let AppState::Idle = new_state {
            println!("Selection cancelled due to timeout");
            self.overlay_manager.hide_all();
            self.keyboard_capture.stop_capture();
        }
    }

    /// Handles escape key or manual cancellation
    pub fn handle_cancellation(&mut self) {
        println!("AppController: Selection cancelled");
        let new_state = self.process_event(StateEvent::SelectionCancelled);

        if let AppState::Idle = new_state {
            println!("Returned to idle state");
            self.overlay_manager.hide_all();
            self.keyboard_capture.stop_capture();
        }
    }

    /// Applies completed selection to active window
    pub fn apply_selection(&mut self) {
        println!("AppController: Applying selection to active window");

        let current_state = self.get_state();
        if let AppState::Selecting(selecting) = current_state {
            println!("DEBUG: Active monitor: {}", selecting.active_monitor_index);
            println!("DEBUG: Selection state: {:?}", selecting.selection.state());
            
            // Get the selection rectangle
            if let Some((top_left, bottom_right)) = selecting.selection.get_normalized_coords() {
                println!("DEBUG: Got normalized coords: ({},{}) to ({},{})", 
                    top_left.row, top_left.col, bottom_right.row, bottom_right.col);
                    
                // Get the grid for the active monitor
                if let Some(grid) = self.get_grid(selecting.active_monitor_index) {
                    // Convert selection to screen rectangle
                    match grid.coords_to_rect(top_left, bottom_right) {
                        Ok(target_rect) => {
                            println!(
                                "Selection: ({},{}) to ({},{}) = screen rect ({},{}) {}x{}",
                                top_left.row, top_left.col,
                                bottom_right.row, bottom_right.col,
                                target_rect.x, target_rect.y,
                                target_rect.w, target_rect.h
                            );

                            // Get the active window and position it
                            match crate::platform::window::get_active_window() {
                                Ok(window_info) => {
                                    println!("Active window: {}", window_info.title);
                                    
                                    // Position the window
                                    match crate::platform::window::position_window(
                                        window_info.handle,
                                        target_rect
                                    ) {
                                        Ok(_) => {
                                            println!("✓ Window positioned successfully");
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to position window: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to get active window: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to convert selection to rectangle: {:?}", e);
                        }
                    }
                } else {
                    eprintln!("DEBUG: Failed to get grid for monitor {}", selecting.active_monitor_index);
                }
            } else {
                eprintln!("DEBUG: get_normalized_coords() returned None!");
            }
        } else {
            eprintln!("DEBUG: Not in Selecting state!");
        }

        // Transition back to idle
        let new_state = self.process_event(StateEvent::SelectionCompleted);
        if let AppState::Idle = new_state {
            println!("Selection completed, returned to idle");
            self.overlay_manager.hide_all();
            self.keyboard_capture.stop_capture();
        }
    }

    /// Processes keyboard events from the hook callback
    ///
    /// This should be called from the main window procedure when receiving
    /// keyboard events from the low-level hook.
    ///
    /// # Arguments
    /// * `wparam` - Windows message parameter containing virtual key code
    pub fn handle_keyboard_event(&mut self, wparam: windows::Win32::Foundation::WPARAM) {
        if let Some(key_event) = KeyboardCaptureManager::parse_message(wparam) {
            match key_event {
                KeyEvent::GridKey(ch) => {
                    self.handle_key_press(ch);
                }
                KeyEvent::Navigation(direction) => {
                    // Convert input navigation direction to app navigation direction
                    let app_direction = match direction {
                        crate::input::NavigationDirection::Left => {
                            crate::app::state::NavigationDirection::Left
                        }
                        crate::input::NavigationDirection::Right => {
                            crate::app::state::NavigationDirection::Right
                        }
                        crate::input::NavigationDirection::Up => {
                            crate::app::state::NavigationDirection::Up
                        }
                        crate::input::NavigationDirection::Down => {
                            crate::app::state::NavigationDirection::Down
                        }
                    };
                    self.handle_navigation(app_direction);
                }
                KeyEvent::Cancel => {
                    self.handle_cancellation();
                }
                KeyEvent::Invalid(_) => {
                    // Invalid keys are ignored (silent)
                }
            }
        }
    }

    /// Checks for selection timeout and handles it if necessary
    ///
    /// This should be called periodically from the main event loop.
    ///
    /// # Returns
    /// true if timeout occurred and was handled
    pub fn check_timeout(&mut self) -> bool {
        let current_state = self.get_state();
        if let AppState::Selecting(selecting) = current_state {
            if selecting.is_timed_out() {
                self.handle_selection_timeout();
                return true;
            }
        }
        false
    }

    /// Gets the keyboard capture message ID for Win32 message processing
    ///
    /// # Returns
    /// Custom Windows message ID that keyboard events are posted to
    pub fn get_keyboard_message_id() -> u32 {
        KeyboardCaptureManager::message_id()
    }

    /// Main event loop for processing keyboard events and timeouts
    pub fn run(&mut self) -> Result<(), AppError> {
        println!("AppController: Starting main event loop");
        println!(
            "Initialized with {} monitors and {} grids",
            self.monitors.len(),
            self.grids.len()
        );

        for (i, monitor) in self.monitors.iter().enumerate() {
            println!(
                "Monitor {}: {}x{} at ({}, {})",
                i,
                monitor.work_area.w,
                monitor.work_area.h,
                monitor.work_area.x,
                monitor.work_area.y
            );
        }

        println!("\n=== INTERACTIVE MODE ===");
        println!("Grid overlay visible. Press keys to select cells:");
        println!("  Q W E");
        println!("  A S D");
        println!("\nPress ESC to cancel or wait 30 seconds for timeout.");
        println!("========================\n");

        const WM_TACTILE_KEY_EVENT: u32 = 0x8000;

        unsafe {
            let mut msg = MSG::default();

            loop {
                // Check for selection timeout
                self.check_timeout();
                
                // Check if we're still in selecting mode
                if matches!(self.get_state(), AppState::Idle) {
                    println!("Exited selecting mode, stopping event loop");
                    break;
                }

                // Check for Windows messages
                let msg_result = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE);

                if msg_result.as_bool() {
                    if msg.message == WM_QUIT {
                        println!("Received WM_QUIT, exiting event loop");
                        break;
                    } else if msg.message == WM_TACTILE_KEY_EVENT {
                        // Handle keyboard event from hook
                        println!("Received keyboard event: vk_code={}", msg.wParam.0);
                        self.handle_keyboard_event(msg.wParam);
                    } else {
                        // Standard Windows message processing
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }

                // Small sleep to prevent busy waiting
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        println!("AppController: Event loop terminated");
        Ok(())
    }
}

impl Drop for AppController {
    fn drop(&mut self) {
        println!("AppController: Shutting down with RAII cleanup");
        // RAII wrappers will automatically clean up their resources
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_creation() {
        // This test depends on having valid monitors
        // In a real environment, this should pass
        // In CI, we might need to mock monitor enumeration
        let dummy_hwnd = HWND(0);
        match AppController::new(dummy_hwnd) {
            Ok(controller) => {
                assert!(controller.monitor_count() > 0);
                assert!(matches!(controller.get_state(), AppState::Idle));
            }
            Err(AppError::MonitorError(_)) => {
                // Expected in CI environments without displays
                println!("Test skipped - no monitors available");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    fn hotkey_toggles_state() {
        let dummy_hwnd = HWND(0);
        if let Ok(mut controller) = AppController::new(dummy_hwnd) {
            // Start in idle
            assert!(matches!(controller.get_state(), AppState::Idle));

            // Hotkey activates selection
            controller.handle_hotkey();
            assert!(matches!(controller.get_state(), AppState::Selecting(_)));

            // Hotkey again returns to idle
            controller.handle_hotkey();
            assert!(matches!(controller.get_state(), AppState::Idle));
        }
    }
}

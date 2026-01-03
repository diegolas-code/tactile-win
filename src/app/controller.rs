//! Application controller and coordination layer
//!
//! The controller orchestrates between input, domain, UI, and platform layers.
//! It maintains stable configuration (grids, monitors) and handles state transitions.

use std::sync::{Arc, Mutex};
use crate::app::state::{AppState, StateEvent, StateMachine, NavigationDirection};
use crate::input::{HotkeyManager, HotkeyError, HotkeyModifier, VirtualKey};
use crate::platform::monitors::{enumerate_monitors, Monitor, MonitorError};
use crate::domain::grid::Grid;
use crate::ui::{OverlayManager, OverlayError};

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

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::MonitorError(e) => write!(f, "Monitor error: {:?}", e),
            AppError::GridCreationFailed(msg) => write!(f, "Grid creation failed: {}", msg),
            AppError::NoSuitableMonitors => write!(f, "No suitable monitors for grid positioning"),
            AppError::HotkeyError(e) => write!(f, "Hotkey error: {:?}", e),
            AppError::OverlayError(e) => write!(f, "Overlay error: {:?}", e),
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
        let mut manager = HotkeyManager::new();
        
        // Start the message loop
        manager.start()?;
        
        Ok(Self {
            manager,
            main_hotkey_id: None,
        })
    }
    
    /// Register the main application hotkey (Ctrl+Alt+Space by default)
    pub fn register_main_hotkey<F>(&mut self, callback: F) -> Result<(), AppError>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let hotkey_id = self.manager.register_hotkey(
            &[HotkeyModifier::Control, HotkeyModifier::Alt],
            VirtualKey::Space,
            Arc::new(callback),
        )?;
        
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

/// RAII wrapper for keyboard capture (placeholder for now)
/// 
/// This will be implemented in Milestone 5: Keyboard Capture
pub struct KeyboardCaptureGuard {
    // Will contain KeyboardCapture in Milestone 5
    _placeholder: (),
}

impl KeyboardCaptureGuard {
    pub fn new() -> Result<Self, AppError> {
        // Placeholder implementation
        Ok(Self { _placeholder: () })
    }
}

impl Drop for KeyboardCaptureGuard {
    fn drop(&mut self) {
        // RAII cleanup will be implemented in Milestone 5
        println!("KeyboardCaptureGuard: Cleanup (placeholder)");
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
    /// Keyboard capture with guaranteed cleanup
    _keyboard_capture: KeyboardCaptureGuard,
    /// Available monitors (stable configuration)
    monitors: Vec<Monitor>,
    /// Grid instances per monitor (stable configuration)
    grids: Vec<Grid>,
}

impl AppController {
    /// Creates a new application controller
    /// 
    /// # Returns
    /// AppController instance or AppError if initialization fails
    pub fn new() -> Result<Self, AppError> {
        // Initialize monitors using Phase 1 infrastructure
        let monitors = enumerate_monitors()?;
        if monitors.is_empty() {
            return Err(AppError::NoSuitableMonitors);
        }

        // Create grids for each monitor using Phase 2 domain logic
        let mut grids = Vec::new();
        for (i, monitor) in monitors.iter().enumerate() {
            match Grid::new(3, 2, monitor.work_area) {
                Ok(grid) => {
                    grids.push(grid);
                    println!("Monitor {}: Created 3x2 grid for {}x{} area", 
                             i, monitor.work_area.w, monitor.work_area.h);
                },
                Err(e) => {
                    return Err(AppError::GridCreationFailed(
                        format!("Monitor {}: {:?}", i, e)
                    ));
                }
            }
        }

        if grids.is_empty() {
            return Err(AppError::NoSuitableMonitors);
        }

        // Initialize RAII-wrapped components
        let mut hotkey_manager = HotkeyManagerGuard::new()?;
        let overlay_manager = OverlayManagerGuard::new(&monitors, &grids)?;
        let keyboard_capture = KeyboardCaptureGuard::new()?;

        // Initialize with idle state
        let state = Arc::new(Mutex::new(AppState::default()));
        let state_for_callback = Arc::clone(&state);

        // Register main application hotkey
        hotkey_manager.register_main_hotkey(move || {
            // Create hotkey event and process it through state machine
            let mut state_guard = state_for_callback.lock().unwrap();
            let current_state = state_guard.clone();
            *state_guard = StateMachine::process_event(current_state, StateEvent::HotkeyPressed, 1);
        })?;

        Ok(Self {
            state,
            hotkey_manager,
            overlay_manager,
            _keyboard_capture: keyboard_capture,
            monitors,
            grids,
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

    /// Handles hotkey press events
    /// 
    /// This will be expanded in future milestones to coordinate with
    /// overlay display and keyboard capture.
    pub fn handle_hotkey(&self) {
        println!("AppController: Hotkey pressed");
        let new_state = self.process_event(StateEvent::HotkeyPressed);
        
        match new_state {
            AppState::Idle => {
                println!("Switched to Idle state");
                // TODO Milestone 3: Hide overlays
                // TODO Milestone 5: Stop keyboard capture
            },
            AppState::Selecting(ref selecting) => {
                println!("Switched to Selecting state on monitor {}", selecting.active_monitor_index);
                // TODO Milestone 3: Show overlays  
                // TODO Milestone 5: Start keyboard capture
            },
        }
    }

    /// Handles key press events during selection
    /// 
    /// # Arguments
    /// * `key` - Character key that was pressed
    /// 
    /// This will be expanded in future milestones to process selection
    /// and coordinate with UI updates.
    pub fn handle_key_press(&self, key: char) {
        println!("AppController: Key pressed: '{}'", key);
        
        let current_state = self.get_state();
        if let AppState::Selecting(selecting) = current_state {
            // TODO: Process key with current grid
            if let Some(grid) = self.get_grid(selecting.active_monitor_index) {
                if grid.contains_key(key) {
                    println!("Valid grid key: '{}' on monitor {}", key, selecting.active_monitor_index);
                    // TODO Milestone 4: Update overlay rendering
                    // TODO Milestone 6: Process selection logic
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
    pub fn handle_navigation(&self, direction: crate::app::state::NavigationDirection) {
        println!("AppController: Navigation: {:?}", direction);
        
        let new_state = self.process_event(StateEvent::Navigation(direction));
        if let AppState::Selecting(selecting) = new_state {
            println!("Switched to monitor {}", selecting.active_monitor_index);
            // TODO Milestone 4: Update overlay rendering to show new active monitor
        }
    }

    /// Handles selection timeout (30 seconds)
    /// 
    /// This will be called by the main event loop when selection times out.
    pub fn handle_selection_timeout(&self) {
        println!("AppController: Selection timed out");
        let new_state = self.process_event(StateEvent::SelectionTimedOut);
        
        if let AppState::Idle = new_state {
            println!("Selection cancelled due to timeout");
            // TODO Milestone 3: Hide overlays
            // TODO Milestone 5: Stop keyboard capture
        }
    }

    /// Handles escape key or manual cancellation
    pub fn handle_cancellation(&self) {
        println!("AppController: Selection cancelled");
        let new_state = self.process_event(StateEvent::SelectionCancelled);
        
        if let AppState::Idle = new_state {
            println!("Returned to idle state");
            // TODO Milestone 3: Hide overlays
            // TODO Milestone 5: Stop keyboard capture
        }
    }

    /// Applies completed selection to active window
    /// 
    /// This will be implemented in Milestone 6: Window Positioning Integration
    pub fn apply_selection(&self) {
        println!("AppController: Applying selection (placeholder)");
        
        // TODO Milestone 6: 
        // 1. Get selection rectangle from current state
        // 2. Get active window using Phase 1 platform code
        // 3. Position window using Phase 1 window management
        // 4. Handle errors and return to idle
        
        let new_state = self.process_event(StateEvent::SelectionCompleted);
        if let AppState::Idle = new_state {
            println!("Selection applied, returned to idle");
        }
    }

    /// Main event loop (placeholder for now)
    /// 
    /// This will be expanded in future milestones to handle:
    /// - Win32 message pump
    /// - Timeout checking
    /// - Component coordination
    pub fn run(&mut self) -> Result<(), AppError> {
        println!("AppController: Starting main event loop (placeholder)");
        println!("Initialized with {} monitors and {} grids", 
                 self.monitors.len(), self.grids.len());
        
        // For now, just demonstrate the components are working
        for (i, monitor) in self.monitors.iter().enumerate() {
            println!("Monitor {}: {}x{} at ({}, {})", 
                     i, 
                     monitor.work_area.w, 
                     monitor.work_area.h,
                     monitor.work_area.x,
                     monitor.work_area.y);
        }
        
        // TODO: Implement actual event loop in future milestones
        // TODO Milestone 2: Register hotkey and handle WM_HOTKEY messages  
        // TODO Milestone 3: Handle overlay events
        // TODO Milestone 5: Handle keyboard hook events
        
        println!("Event loop completed (placeholder)");
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
        match AppController::new() {
            Ok(controller) => {
                assert!(controller.monitor_count() > 0);
                assert!(matches!(controller.get_state(), AppState::Idle));
            },
            Err(AppError::MonitorError(_)) => {
                // Expected in CI environments without displays
                println!("Test skipped - no monitors available");
            },
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    fn hotkey_toggles_state() {
        if let Ok(controller) = AppController::new() {
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
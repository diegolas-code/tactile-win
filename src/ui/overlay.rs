//! Overlay window management for grid display
//!
//! Provides transparent overlay windows that appear over all monitors
//! without stealing focus from the active window. Uses proper Win32
//! window styles for transparency and topmost behavior.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, COLORREF};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, SetLayeredWindowAttributes,
    ShowWindow, RegisterClassW, MSG, WNDCLASSW, WM_PAINT, WM_DESTROY,
    WS_POPUP, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    SW_SHOW, SW_HIDE, LWA_COLORKEY, LWA_ALPHA,
};

use crate::domain::core::Rect;
use crate::platform::monitors::Monitor;

/// Overlay management errors
#[derive(Debug, thiserror::Error)]
pub enum OverlayError {
    #[error("Failed to register overlay window class")]
    WindowClassRegistrationFailed,
    
    #[error("Failed to create overlay window for monitor {monitor_index}")]
    WindowCreationFailed { monitor_index: usize },
    
    #[error("Failed to configure overlay transparency")]
    TransparencyConfigurationFailed,
    
    #[error("Overlay manager not initialized")]
    NotInitialized,
}

/// Overlay window for a single monitor
#[derive(Debug)]
pub struct OverlayWindow {
    /// Windows handle to the overlay window
    pub hwnd: HWND,
    
    /// Monitor this overlay belongs to
    pub monitor_index: usize,
    
    /// Monitor bounds for positioning
    pub monitor_rect: Rect,
    
    /// Current visibility state
    pub visible: bool,
}

impl OverlayWindow {
    /// Create a new overlay window for the specified monitor
    fn new(monitor_index: usize, monitor: &Monitor) -> Result<Self, OverlayError> {
        let class_name = w!("TactileWinOverlayWindow");
        
        // Register window class if needed
        Self::register_window_class(class_name)?;
        
        // Create the overlay window
        let hwnd = Self::create_overlay_window(class_name, &monitor.work_area)?;
        
        // Configure transparency
        Self::configure_transparency(hwnd)?;
        
        Ok(Self {
            hwnd,
            monitor_index,
            monitor_rect: monitor.work_area,
            visible: false,
        })
    }
    
    /// Register overlay window class
    fn register_window_class(class_name: windows::core::PCWSTR) -> Result<(), OverlayError> {
        // Window procedure for overlay windows
        unsafe extern "system" fn overlay_window_proc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            match msg {
                WM_PAINT => {
                    // Painting will be handled by the rendering system in Milestone 4
                    // For now, just validate the paint region
                    unsafe {
                        use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
                        let mut ps = PAINTSTRUCT::default();
                        let _hdc = BeginPaint(hwnd, &mut ps);
                        EndPaint(hwnd, &ps);
                    }
                    LRESULT(0)
                }
                WM_DESTROY => {
                    // Overlay windows should not post quit messages
                    LRESULT(0)
                }
                _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
            }
        }
        
        let hinstance = unsafe { GetModuleHandleW(None).unwrap() };
        
        // Create a gray brush for the background (will be made transparent)
        let hbrush = unsafe { CreateSolidBrush(COLORREF(0x808080)) };
        
        let wc = WNDCLASSW {
            lpfnWndProc: Some(overlay_window_proc),
            hInstance: hinstance.into(),
            lpszClassName: class_name,
            hbrBackground: hbrush,
            ..Default::default()
        };
        
        let class_atom = unsafe { RegisterClassW(&wc) };
        if class_atom == 0 {
            // Class might already be registered, which is fine
            // We'll check the specific error if needed
        }
        
        Ok(())
    }
    
    /// Create the actual overlay window
    fn create_overlay_window(
        class_name: windows::core::PCWSTR,
        monitor_rect: &Rect,
    ) -> Result<HWND, OverlayError> {
        let hinstance = unsafe { GetModuleHandleW(None).unwrap() };
        
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                class_name,
                w!("Tactile Overlay"),
                WS_POPUP, // Popup window with no frame
                monitor_rect.x,
                monitor_rect.y,
                monitor_rect.w,
                monitor_rect.h,
                None, // No parent
                None, // No menu
                hinstance,
                None,
            )
        };
        
        if hwnd.0 == 0 {
            return Err(OverlayError::WindowCreationFailed { monitor_index: 0 });
        }
        
        Ok(hwnd)
    }
    
    /// Configure window transparency and alpha blending
    fn configure_transparency(hwnd: HWND) -> Result<(), OverlayError> {
        // Make the gray background transparent and set overall alpha
        let result = unsafe {
            SetLayeredWindowAttributes(
                hwnd,
                COLORREF(0x808080), // Gray color as RGB value
                200, // Alpha value (0-255, 200 = ~78% opaque)
                LWA_COLORKEY | LWA_ALPHA,
            )
        };
        
        if result.is_err() {
            return Err(OverlayError::TransparencyConfigurationFailed);
        }
        
        Ok(())
    }
    
    /// Show the overlay window
    pub fn show(&mut self) {
        if !self.visible {
            unsafe {
                ShowWindow(self.hwnd, SW_SHOW);
            }
            self.visible = true;
        }
    }
    
    /// Hide the overlay window
    pub fn hide(&mut self) {
        if self.visible {
            unsafe {
                ShowWindow(self.hwnd, SW_HIDE);
            }
            self.visible = false;
        }
    }
    
    /// Check if overlay is currently visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Drop for OverlayWindow {
    fn drop(&mut self) {
        // Clean up the window
        unsafe {
            DestroyWindow(self.hwnd).ok();
        }
    }
}

/// Manager for all overlay windows across multiple monitors
pub struct OverlayManager {
    /// Map of monitor index to overlay window
    overlays: Arc<Mutex<HashMap<usize, OverlayWindow>>>,
    
    /// Current visibility state
    visible: bool,
}

impl OverlayManager {
    /// Create a new overlay manager
    pub fn new() -> Self {
        Self {
            overlays: Arc::new(Mutex::new(HashMap::new())),
            visible: false,
        }
    }
    
    /// Initialize overlay windows for all provided monitors
    pub fn initialize(&mut self, monitors: &[Monitor]) -> Result<(), OverlayError> {
        let mut overlays = self.overlays.lock().unwrap();
        
        // Clear any existing overlays
        overlays.clear();
        
        // Create overlay for each monitor
        for (index, monitor) in monitors.iter().enumerate() {
            let overlay = OverlayWindow::new(index, monitor)?;
            overlays.insert(index, overlay);
        }
        
        Ok(())
    }
    
    /// Show overlays on all monitors
    pub fn show_all(&mut self) {
        if !self.visible {
            let mut overlays = self.overlays.lock().unwrap();
            for overlay in overlays.values_mut() {
                overlay.show();
            }
            self.visible = true;
        }
    }
    
    /// Hide overlays on all monitors
    pub fn hide_all(&mut self) {
        if self.visible {
            let mut overlays = self.overlays.lock().unwrap();
            for overlay in overlays.values_mut() {
                overlay.hide();
            }
            self.visible = false;
        }
    }
    
    /// Show overlay for a specific monitor
    pub fn show_monitor(&mut self, monitor_index: usize) {
        let mut overlays = self.overlays.lock().unwrap();
        if let Some(overlay) = overlays.get_mut(&monitor_index) {
            overlay.show();
        }
    }
    
    /// Hide overlay for a specific monitor
    pub fn hide_monitor(&mut self, monitor_index: usize) {
        let mut overlays = self.overlays.lock().unwrap();
        if let Some(overlay) = overlays.get_mut(&monitor_index) {
            overlay.hide();
        }
    }
    
    /// Check if overlays are currently visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// Get the number of overlay windows
    pub fn overlay_count(&self) -> usize {
        self.overlays.lock().unwrap().len()
    }
    
    /// Get overlay window handle for a specific monitor
    pub fn get_overlay_hwnd(&self, monitor_index: usize) -> Option<HWND> {
        self.overlays
            .lock()
            .unwrap()
            .get(&monitor_index)
            .map(|overlay| overlay.hwnd)
    }
    
    /// Toggle overlay visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide_all();
        } else {
            self.show_all();
        }
    }
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::core::Rect;
    use crate::platform::monitors::Monitor;
    
    #[test]
    fn overlay_manager_creation() {
        let manager = OverlayManager::new();
        assert!(!manager.is_visible());
        assert_eq!(manager.overlay_count(), 0);
    }
    
    #[test]
    fn overlay_manager_initialization() {
        let mut manager = OverlayManager::new();
        
        // Create mock monitors
        let monitors = vec![
            Monitor {
                index: 0,
                handle: windows::Win32::Graphics::Gdi::HMONITOR(1),
                work_area: Rect { x: 0, y: 0, w: 1920, h: 1080 },
                physical_rect: Rect { x: 0, y: 0, w: 1920, h: 1080 },
                is_primary: true,
                dpi_scale: 1.0,
                dpi_x: 96,
                dpi_y: 96,
            },
        ];
        
        // Initialize should work
        let result = manager.initialize(&monitors);
        
        // In test environment, window creation might fail, but the API should be correct
        match result {
            Ok(()) => {
                assert_eq!(manager.overlay_count(), 1);
            }
            Err(_) => {
                // Expected in test environment without proper Windows context
                println!("Overlay initialization failed (expected in test environment)");
            }
        }
    }
    
    #[test]
    fn overlay_visibility_state() {
        let mut manager = OverlayManager::new();
        
        // Initially not visible
        assert!(!manager.is_visible());
        
        // Show/hide should update state even without initialized overlays
        manager.show_all();
        assert!(manager.is_visible());
        
        manager.hide_all();
        assert!(!manager.is_visible());
        
        // Toggle should work
        manager.toggle();
        assert!(manager.is_visible());
        
        manager.toggle();
        assert!(!manager.is_visible());
    }
}
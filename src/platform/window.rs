//! Window management and positioning
//!
//! This module handles:
//! - Getting the currently active window
//! - Checking if a window is resizable
//! - Moving and resizing windows to specific rectangles
//! - Preserving focus during window operations
//!
//! CRITICAL: All operations must preserve the active window's focus state

use crate::domain::core::Rect;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Error types for window operations
#[derive(Debug)]
pub enum WindowError {
    /// No active window found
    NoActiveWindow,
    /// Failed to get window information
    InfoFailed(HWND),
    /// Window cannot be resized (e.g., dialog boxes)
    NotResizable(HWND),
    /// Failed to position the window
    PositionFailed(HWND),
    /// Window handle is invalid
    InvalidHandle(HWND),
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowError::NoActiveWindow => write!(f, "No active window found"),
            WindowError::InfoFailed(hwnd) => write!(f, "Failed to get info for window {:?}", hwnd),
            WindowError::NotResizable(hwnd) => write!(f, "Window {:?} is not resizable", hwnd),
            WindowError::PositionFailed(hwnd) => write!(f, "Failed to position window {:?}", hwnd),
            WindowError::InvalidHandle(hwnd) => write!(f, "Invalid window handle {:?}", hwnd),
        }
    }
}

impl std::error::Error for WindowError {}

/// Information about a window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window handle
    pub handle: HWND,
    /// Window title (if available)
    pub title: String,
    /// Current window rectangle in screen coordinates
    pub rect: Rect,
    /// Whether the window can be resized
    pub is_resizable: bool,
    /// Whether this is a child window
    pub is_child: bool,
    /// Whether the window is currently maximized
    pub is_maximized: bool,
}

/// Gets the currently active (foreground) window
pub fn get_active_window() -> Result<WindowInfo, WindowError> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Err(WindowError::NoActiveWindow);
        }

        get_window_info(hwnd)
    }
}

/// Gets information about the specified window
pub fn get_window_info(hwnd: HWND) -> Result<WindowInfo, WindowError> {
    unsafe {
        // Validate the window handle
        if !IsWindow(hwnd).as_bool() {
            return Err(WindowError::InvalidHandle(hwnd));
        }

        // Get window title
        let mut title_buffer = [0u16; 512];
        let title_length = GetWindowTextW(hwnd, &mut title_buffer);
        let title = if title_length > 0 {
            String::from_utf16_lossy(&title_buffer[..title_length as usize])
        } else {
            String::from("<No Title>")
        };

        // Get window rectangle
        let mut window_rect = RECT::default();
        if GetWindowRect(hwnd, &mut window_rect).is_err() {
            return Err(WindowError::InfoFailed(hwnd));
        }

        let rect = Rect::new(
            window_rect.left,
            window_rect.top,
            window_rect.right - window_rect.left,
            window_rect.bottom - window_rect.top,
        );

        // Check if window is resizable by examining its style
        let style = WINDOW_STYLE(GetWindowLongW(hwnd, GWL_STYLE) as u32);
        let is_resizable = (style & WS_THICKFRAME) != WINDOW_STYLE(0);

        // Check if it's a child window
        let is_child = (style & WS_CHILD) != WINDOW_STYLE(0);

        // Check if window is maximized
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };

        let is_maximized = if GetWindowPlacement(hwnd, &mut placement).is_ok() {
            placement.showCmd == (SW_SHOWMAXIMIZED.0 as u32)
        } else {
            false
        };

        Ok(WindowInfo {
            handle: hwnd,
            title,
            rect,
            is_resizable,
            is_child,
            is_maximized,
        })
    }
}

/// Moves and resizes a window to the specified rectangle
///
/// This function:
/// - Preserves the window's Z-order
/// - Does not change focus
/// - Returns an error if the window is not resizable
pub fn position_window(hwnd: HWND, target_rect: Rect) -> Result<(), WindowError> {
    unsafe {
        // Validate the window handle
        if !IsWindow(hwnd).as_bool() {
            return Err(WindowError::InvalidHandle(hwnd));
        }

        // Get window info to check if it's resizable
        let window_info = get_window_info(hwnd)?;
        if !window_info.is_resizable {
            return Err(WindowError::NotResizable(hwnd));
        }

        // If window is maximized, restore it first
        if window_info.is_maximized {
            if !ShowWindow(hwnd, SW_RESTORE).as_bool() {
                return Err(WindowError::PositionFailed(hwnd));
            }

            // Give the window time to restore (avoid race conditions)
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Position the window
        // SWP_NOACTIVATE: Don't activate the window (preserve focus)
        // SWP_NOZORDER: Don't change Z-order position (HWND parameter ignored)
        let result = SetWindowPos(
            hwnd,
            HWND(0), // Ignored due to SWP_NOZORDER flag
            target_rect.x,
            target_rect.y,
            target_rect.w,
            target_rect.h,
            SWP_NOACTIVATE | SWP_NOZORDER,
        );

        if result.is_err() {
            return Err(WindowError::PositionFailed(hwnd));
        }

        Ok(())
    }
}

/// Positions the active window to the specified rectangle
///
/// This is a convenience function that combines getting the active window
/// and positioning it in one operation.
pub fn position_active_window(target_rect: Rect) -> Result<(), WindowError> {
    let window_info = get_active_window()?;
    position_window(window_info.handle, target_rect)
}

/// Checks if the specified window is suitable for grid positioning
///
/// Returns false for:
/// - Non-resizable windows (dialogs, etc.)
/// - Child windows
/// - System windows
pub fn is_window_suitable_for_positioning(hwnd: HWND) -> bool {
    match get_window_info(hwnd) {
        Ok(info) => info.is_resizable && !info.is_child,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_active_window() {
        // This test will only pass if there's actually an active window
        // In a real environment, there should always be at least the test runner window
        let result = get_active_window();

        // We can't guarantee an active window in all test environments,
        // but if we get one, it should be valid
        if let Ok(window_info) = result {
            assert!(window_info.handle.0 != 0);
            assert!(!window_info.title.is_empty() || window_info.title == "<No Title>");
            assert!(window_info.rect.w > 0);
            assert!(window_info.rect.h > 0);
        }
    }

    #[test]
    fn window_info_validation() {
        // Test with invalid handle
        let invalid_hwnd = HWND(999999);
        let result = get_window_info(invalid_hwnd);
        assert!(result.is_err());

        if let Err(WindowError::InvalidHandle(handle)) = result {
            assert_eq!(handle, invalid_hwnd);
        } else {
            panic!("Expected InvalidHandle error");
        }
    }
}

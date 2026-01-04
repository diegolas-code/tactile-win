//! General Windows platform utilities
//!
//! This module contains Win32 helper functions that don't fit into
//! other specialized platform modules.

use crate::domain::core::Rect;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

/// Gets the currently active (foreground) window
///
/// # Returns
/// Handle to the foreground window, or a null handle if no window is active
pub fn get_foreground_window() -> HWND {
    unsafe { GetForegroundWindow() }
}

/// Converts a domain rectangle to Windows RECT structure
///
/// # Arguments
/// * `rect` - Domain rectangle to convert
///
/// # Returns
/// Windows RECT structure
pub fn rect_to_win32_rect(rect: &Rect) -> windows::Win32::Foundation::RECT {
    windows::Win32::Foundation::RECT {
        left: rect.x,
        top: rect.y,
        right: rect.x + rect.w,
        bottom: rect.y + rect.h,
    }
}

/// Converts a Windows RECT to domain rectangle
///
/// # Arguments
/// * `rect` - Windows RECT structure
///
/// # Returns
/// Domain rectangle
pub fn win32_rect_to_rect(rect: &windows::Win32::Foundation::RECT) -> Rect {
    Rect {
        x: rect.left,
        y: rect.top,
        w: rect.right - rect.left,
        h: rect.bottom - rect.top,
    }
}

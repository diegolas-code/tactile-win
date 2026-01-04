//! Modal keyboard capture for grid selection
//!
//! This module implements low-level keyboard capture during modal selection mode.
//! Critical threading requirements:
//! - Hook callback runs on SYSTEM thread, NOT main thread
//! - Hook NEVER mutates application state directly
//! - All events are posted to main thread for processing
//! - This prevents deadlocks and race conditions

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CallNextHookEx, HHOOK, KBDLLHOOKSTRUCT, PostMessageW, SetWindowsHookExW,
            UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
        },
    },
    core::PCWSTR,
};

/// Custom window message for keyboard events from hook
const WM_TACTILE_KEY_EVENT: u32 = 0x8000; // WM_APP range

/// Errors that can occur during keyboard capture
#[derive(Debug, thiserror::Error)]
pub enum KeyboardCaptureError {
    #[error("Failed to install keyboard hook")]
    HookInstallationFailed,
    #[error("Hook callback registration failed")]
    CallbackRegistrationFailed,
    #[error("Failed to uninstall keyboard hook")]
    UninstallFailed,
}

/// Navigation direction for multi-monitor support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Key events that can be captured during modal mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyEvent {
    /// Grid selection key (Q, W, E, A, S, D, Z, X, C, V, etc.)
    GridKey(char),
    /// Navigation between monitors
    Navigation(NavigationDirection),
    /// Cancel selection
    Cancel,
    /// Invalid key (for debugging)
    Invalid(u32),
}

impl KeyEvent {
    /// Convert Windows virtual key code to KeyEvent
    fn from_vk_code(vk_code: u32) -> Option<Self> {
        match vk_code {
            // Grid keys (QWERTY layout)
            0x51 => Some(KeyEvent::GridKey('Q')), // Q
            0x57 => Some(KeyEvent::GridKey('W')), // W
            0x45 => Some(KeyEvent::GridKey('E')), // E
            0x52 => Some(KeyEvent::GridKey('R')), // R
            0x54 => Some(KeyEvent::GridKey('T')), // T
            0x59 => Some(KeyEvent::GridKey('Y')), // Y
            0x55 => Some(KeyEvent::GridKey('U')), // U
            0x49 => Some(KeyEvent::GridKey('I')), // I
            0x4f => Some(KeyEvent::GridKey('O')), // O
            0x50 => Some(KeyEvent::GridKey('P')), // P

            0x41 => Some(KeyEvent::GridKey('A')), // A
            0x53 => Some(KeyEvent::GridKey('S')), // S
            0x44 => Some(KeyEvent::GridKey('D')), // D
            0x46 => Some(KeyEvent::GridKey('F')), // F
            0x47 => Some(KeyEvent::GridKey('G')), // G
            0x48 => Some(KeyEvent::GridKey('H')), // H
            0x4a => Some(KeyEvent::GridKey('J')), // J
            0x4b => Some(KeyEvent::GridKey('K')), // K
            0x4c => Some(KeyEvent::GridKey('L')), // L

            0x5a => Some(KeyEvent::GridKey('Z')), // Z
            0x58 => Some(KeyEvent::GridKey('X')), // X
            0x43 => Some(KeyEvent::GridKey('C')), // C
            0x56 => Some(KeyEvent::GridKey('V')), // V
            0x42 => Some(KeyEvent::GridKey('B')), // B
            0x4e => Some(KeyEvent::GridKey('N')), // N
            0x4d => Some(KeyEvent::GridKey('M')), // M

            // Navigation keys
            0x25 => Some(KeyEvent::Navigation(NavigationDirection::Left)), // VK_LEFT
            0x27 => Some(KeyEvent::Navigation(NavigationDirection::Right)), // VK_RIGHT
            0x26 => Some(KeyEvent::Navigation(NavigationDirection::Up)),   // VK_UP
            0x28 => Some(KeyEvent::Navigation(NavigationDirection::Down)), // VK_DOWN

            // Cancel keys
            0x1b => Some(KeyEvent::Cancel), // VK_ESCAPE

            // Any other key during selection mode - treat as invalid and cancel
            _ => Some(KeyEvent::Invalid(vk_code)),
        }
    }
}

/// Global state for keyboard hook callback
/// CRITICAL: This must be minimal and thread-safe
static mut KEYBOARD_CAPTURE_STATE: Option<KeyboardCaptureState> = None;

struct KeyboardCaptureState {
    target_hwnd: HWND,
}

fn call_next_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Manages modal keyboard capture during selection mode
#[derive(Debug)]
pub struct KeyboardCapture {
    hook: Option<HHOOK>,
    target_hwnd: HWND,
}

impl KeyboardCapture {
    /// Create new keyboard capture manager for the specified target window
    pub fn new(target_hwnd: HWND) -> Self {
        Self {
            hook: None,
            target_hwnd,
        }
    }

    /// Start capturing keyboard input
    ///
    /// CRITICAL: This installs a low-level keyboard hook that runs on system thread.
    /// The hook callback posts events to main thread - never mutates state directly.
    pub fn start_capture(&mut self) -> Result<(), KeyboardCaptureError> {
        if self.hook.is_some() {
            // Already capturing
            return Ok(());
        }

        unsafe {
            // Set up global state for hook callback
            KEYBOARD_CAPTURE_STATE = Some(KeyboardCaptureState {
                target_hwnd: self.target_hwnd,
            });

            // Install low-level keyboard hook
            let hinstance = GetModuleHandleW(PCWSTR::null())
                .map_err(|_| KeyboardCaptureError::HookInstallationFailed)?;

            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), hinstance, 0)
                .map_err(|_| KeyboardCaptureError::HookInstallationFailed)?;

            self.hook = Some(hook);
        }

        Ok(())
    }

    /// Stop capturing keyboard input
    pub fn stop_capture(&mut self) -> Result<(), KeyboardCaptureError> {
        if let Some(hook) = self.hook.take() {
            unsafe {
                // Remove hook
                UnhookWindowsHookEx(hook).map_err(|_| KeyboardCaptureError::UninstallFailed)?;

                // Clear global state
                KEYBOARD_CAPTURE_STATE = None;
            }
        }
        Ok(())
    }

    /// Check if currently capturing
    pub fn is_capturing(&self) -> bool {
        self.hook.is_some()
    }
}

impl Drop for KeyboardCapture {
    fn drop(&mut self) {
        // Guaranteed cleanup
        let _ = self.stop_capture();
    }
}

/// Low-level keyboard hook procedure
///
/// CRITICAL THREADING NOTES:
/// - This runs on SYSTEM thread, NOT main application thread
/// - NEVER mutate application state from this callback
/// - NEVER call blocking operations from this callback
/// - Only post messages to main thread for processing
/// - Must call CallNextHookEx to maintain system stability
unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // Only process HC_ACTION
    if code < 0 {
        return call_next_hook(code, wparam, lparam);
    }

    // Check if we have valid state
    let state = unsafe {
        match &*std::ptr::addr_of!(KEYBOARD_CAPTURE_STATE) {
            Some(state) => state,
            None => {
                return call_next_hook(code, wparam, lparam);
            }
        }
    };

    // Only handle key down events
    if wparam.0 != (WM_KEYDOWN as usize) && wparam.0 != (WM_SYSKEYDOWN as usize) {
        return call_next_hook(code, wparam, lparam);
    }

    // Parse keyboard data
    let keyboard_data = lparam.0 as *const KBDLLHOOKSTRUCT;
    let vk_code = unsafe { (*keyboard_data).vkCode };

    // Convert to KeyEvent
    let key_event = KeyEvent::from_vk_code(vk_code);

    match key_event {
        Some(_event) => {
            // Valid tactile key - consume it and post to main thread
            let _ = unsafe {
                PostMessageW(
                    state.target_hwnd,
                    WM_TACTILE_KEY_EVENT,
                    WPARAM(vk_code as usize),
                    LPARAM(0),
                )
            };

            // Return non-zero to consume the key (don't pass to other applications)
            LRESULT(1)
        }
        None => {
            // Not a tactile key - pass through to system
            call_next_hook(code, wparam, lparam)
        }
    }
}

/// RAII wrapper for keyboard capture with guaranteed cleanup
#[derive(Debug)]
pub struct KeyboardCaptureGuard {
    capture: KeyboardCapture,
}

impl KeyboardCaptureGuard {
    /// Create new keyboard capture guard
    pub fn new(target_hwnd: HWND) -> Result<Self, KeyboardCaptureError> {
        let mut capture = KeyboardCapture::new(target_hwnd);
        capture.start_capture()?;

        Ok(Self { capture })
    }

    /// Check if currently capturing
    pub fn is_capturing(&self) -> bool {
        self.capture.is_capturing()
    }

    /// Get the custom message ID for keyboard events
    pub fn message_id() -> u32 {
        WM_TACTILE_KEY_EVENT
    }

    /// Parse a Windows message into a KeyEvent
    ///
    /// Call this from your main window procedure when receiving WM_TACTILE_KEY_EVENT
    pub fn parse_message(wparam: WPARAM) -> Option<KeyEvent> {
        let vk_code = wparam.0 as u32;
        KeyEvent::from_vk_code(vk_code)
    }
}

impl Drop for KeyboardCaptureGuard {
    fn drop(&mut self) {
        // Guaranteed cleanup on guard destruction
        let _ = self.capture.stop_capture();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_event_conversion() {
        // Test grid keys
        assert_eq!(KeyEvent::from_vk_code(0x51), Some(KeyEvent::GridKey('Q')));
        assert_eq!(KeyEvent::from_vk_code(0x41), Some(KeyEvent::GridKey('A')));
        assert_eq!(KeyEvent::from_vk_code(0x5a), Some(KeyEvent::GridKey('Z')));

        // Test navigation keys
        assert_eq!(
            KeyEvent::from_vk_code(0x25),
            Some(KeyEvent::Navigation(NavigationDirection::Left))
        );
        assert_eq!(
            KeyEvent::from_vk_code(0x27),
            Some(KeyEvent::Navigation(NavigationDirection::Right))
        );

        // Test cancel key
        assert_eq!(KeyEvent::from_vk_code(0x1b), Some(KeyEvent::Cancel));

        // Test invalid key
        assert_eq!(KeyEvent::from_vk_code(0x01), None); // VK_LBUTTON
    }

    #[test]
    fn keyboard_capture_creation() {
        use windows::Win32::Foundation::HWND;

        let hwnd = HWND(0); // Dummy HWND for testing
        let capture = KeyboardCapture::new(hwnd);

        assert!(!capture.is_capturing());
        assert!(capture.hook.is_none());
    }

    // Note: Testing actual hook installation requires valid HWND and message loop
    // These would be integration tests run with actual window
}

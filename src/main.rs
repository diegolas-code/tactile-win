//! Tactile-Win: Grid-based window positioning for Windows
//!
//! Phase 1: Infrastructure (DPI awareness, monitor enumeration, window management) ✓
//! Phase 2: Domain Logic (keyboard layout, grid geometry, selection process) ✓

use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::{
    CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
    SetConsoleCtrlHandler,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

mod app;
mod config;
mod domain;
mod input;
mod platform;
mod ui;

static SHUTDOWN_FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();

unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT
        | CTRL_SHUTDOWN_EVENT => {
            if let Some(flag) = SHUTDOWN_FLAG.get() {
                flag.store(true, Ordering::SeqCst);
            }
            BOOL(1)
        }
        _ => BOOL(0),
    }
}

fn install_console_ctrl_handler(flag: Arc<AtomicBool>) -> windows::core::Result<()> {
    let _ = SHUTDOWN_FLAG.set(flag);
    unsafe {
        SetConsoleCtrlHandler(Some(console_ctrl_handler), BOOL(1))?;
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CRITICAL: Set DPI awareness before any other Windows API calls
    // This ensures our application gets real pixel coordinates instead of scaled ones
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
    }

    println!("Tactile-Win: Starting Application\n");

    // Handle console signals (Ctrl+C, close, logoff, shutdown) gracefully
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    install_console_ctrl_handler(shutdown_flag.clone())?;

    // Create a dummy window for message processing
    // This is needed for the keyboard hook to post messages to
    let main_window = create_main_window()?;

    // Create and run the main application controller
    match app::controller::AppController::new(main_window) {
        Ok(mut app) => {
            println!("Application controller initialized successfully");

            // Start the main event loop
            if let Err(e) = app.run(shutdown_flag.clone()) {
                eprintln!("Application error: {}", e);
                return Err(format!("Application error: {}", e).into());
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize application: {}", e);
            return Err(format!("Failed to initialize application: {}", e).into());
        }
    }

    println!("Tactile-Win application terminated normally");
    Ok(())
}

/// Creates a minimal hidden window for message processing
///
/// This window is needed to receive messages from the keyboard hook
fn create_main_window() -> Result<HWND, Box<dyn std::error::Error>> {
    unsafe {
        let instance = GetModuleHandleW(PCWSTR::null())?;

        // Register window class
        let class_name = "TactileWinMainWindow";
        let class_name_wide: Vec<u16> = class_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR::from_raw(class_name_wide.as_ptr()),
            ..Default::default()
        };

        RegisterClassW(&wc);

        // Create hidden window
        let window_name: Vec<u16> = "TactileWin"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR::from_raw(class_name_wide.as_ptr()),
            PCWSTR::from_raw(window_name.as_ptr()),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            None,
            None,
            instance,
            None,
        );

        if hwnd.0 == 0 {
            return Err("Failed to create window".into());
        }

        Ok(hwnd)
    }
}

/// Window procedure for the main window
/// Handles keyboard events from the low-level keyboard hook
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Check for custom keyboard event message
    const WM_TACTILE_KEY_EVENT: u32 = 0x8000;

    if msg == WM_TACTILE_KEY_EVENT {
        // Get the application controller from window user data
        // For now, just log the event - we'll need to pass controller reference
        println!(
            "Main window: Received keyboard event, vk_code: {}",
            wparam.0
        );
        // TODO: Call controller.handle_keyboard_event(wparam) once we can access controller
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

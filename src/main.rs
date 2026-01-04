//! Tactile-Win: Grid-based window positioning for Windows
//!
//! Phase 1: Infrastructure (DPI awareness, monitor enumeration, window management) ✓
//! Phase 2: Domain Logic (keyboard layout, grid geometry, selection process) ✓

use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

mod app;
mod domain;
mod input;
mod platform;
mod ui;

use domain::core::Rect;
use domain::grid::Grid;
use domain::keyboard::GridCoords;
use domain::selection::Selection;
use platform::{ monitors, window };

// Phase 1 Constants
const DEFAULT_GRID_COLS: u32 = 3;
const DEFAULT_GRID_ROWS: u32 = 2;
const MIN_CELL_WIDTH: i32 = 480;
const MIN_CELL_HEIGHT: i32 = 360;
const MIN_MONITOR_HEIGHT: i32 = 600;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CRITICAL: Set DPI awareness before any other Windows API calls
    // This ensures our application gets real pixel coordinates instead of scaled ones
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
    }

    println!("Tactile-Win: Starting Application\n");

    // Create a dummy window for message processing
    // This is needed for the keyboard hook to post messages to
    let main_window = create_main_window()?;

    // Create and run the main application controller
    match app::controller::AppController::new(main_window) {
        Ok(mut app) => {
            println!("Application controller initialized successfully");

            // Start the main event loop
            if let Err(e) = app.run() {
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
        let window_name: Vec<u16> = "TactileWin".encode_utf16().chain(std::iter::once(0)).collect();

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
            None
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
    lparam: LPARAM
) -> LRESULT {
    // Check for custom keyboard event message
    const WM_TACTILE_KEY_EVENT: u32 = 0x8000;
    
    if msg == WM_TACTILE_KEY_EVENT {
        // Get the application controller from window user data
        // For now, just log the event - we'll need to pass controller reference
        println!("Main window: Received keyboard event, vk_code: {}", wparam.0);
        // TODO: Call controller.handle_keyboard_event(wparam) once we can access controller
    }
    
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Demonstrates Phase 2 domain logic integration
fn demo_phase2_integration() {
    println!("=== Phase 2 Domain Logic Demo ===");

    // Create a sample monitor work area (1920x1080)
    let work_area = Rect::new(0, 0, 1920, 1080);
    println!("Sample monitor: {}x{}", work_area.w, work_area.h);

    // Create a 3x2 grid
    let grid = match Grid::new(3, 2, work_area) {
        Ok(grid) => {
            println!(
                "✓ Grid created: {} rows x {} columns",
                grid.dimensions().0,
                grid.dimensions().1
            );
            println!("  Cell size: {}x{}", grid.cell_size().0, grid.cell_size().1);
            grid
        }
        Err(e) => {
            println!("✗ Grid creation failed: {:?}", e);
            return;
        }
    };

    // Show keyboard mapping
    println!("  Valid keys: {:?}", grid.valid_keys());

    // Demo 1: Single cell selection (Q key)
    println!("\\n1. Single cell selection (Q):");
    match grid.key_to_rect('Q') {
        Ok(rect) => println!("   Q -> ({}, {}) {}x{}", rect.x, rect.y, rect.w, rect.h),
        Err(e) => println!("   Error: {:?}", e),
    }

    // Demo 2: Multi-cell selection using selection process
    println!("\\n2. Two-step selection process (Q → S):");
    let mut selection = Selection::new();

    // Start with Q (0,0)
    let q_coords = GridCoords::new(0, 0);
    if let Err(e) = selection.start(q_coords) {
        println!("   Error starting selection: {:?}", e);
        return;
    }
    println!("   Started at Q (0,0)");

    // Complete with S (1,1)
    let s_coords = GridCoords::new(1, 1);
    if let Err(e) = selection.complete(s_coords) {
        println!("   Error completing selection: {:?}", e);
        return;
    }
    println!("   Completed at S (1,1)");

    // Show selection results
    if let Some((tl, br)) = selection.get_normalized_coords() {
        println!("   Normalized: ({},{}) to ({},{})", tl.row, tl.col, br.row, br.col);
        if let Some((w, h)) = selection.get_dimensions() {
            println!(
                "   Covers: {} cols x {} rows = {} cells",
                w,
                h,
                selection.get_cell_count().unwrap()
            );
        }

        // Convert to screen rectangle
        match grid.coords_to_rect(tl, br) {
            Ok(rect) => println!("   Screen rect: ({}, {}) {}x{}", rect.x, rect.y, rect.w, rect.h),
            Err(e) => println!("   Conversion error: {:?}", e),
        }
    }

    // Demo 3: Direct key-based selection
    println!("\\n3. Direct key selection (Q to X):");
    match grid.keys_to_rect('Q', 'X') {
        Ok(rect) => {
            println!("   Q→X covers full grid width x 2 rows");
            println!("   Screen rect: ({}, {}) {}x{}", rect.x, rect.y, rect.w, rect.h);
        }
        Err(e) => println!("   Error: {:?}", e),
    }

    println!("✓ Phase 2 domain logic working correctly");
}

/// Validates Phase 1 infrastructure components
fn run_phase1_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Phase 1 Validation ===");

    // 1. Monitor enumeration
    println!("1. Enumerating monitors...");
    let monitors = monitors::enumerate_monitors()?;
    println!("   Found {} monitors", monitors.len());

    for (i, monitor) in monitors.iter().enumerate() {
        println!(
            "   Monitor {}: {}x{} at ({}, {}) - DPI: {:.1}x - Primary: {}",
            i,
            monitor.work_area.w,
            monitor.work_area.h,
            monitor.work_area.x,
            monitor.work_area.y,
            monitor.dpi_scale,
            monitor.is_primary
        );

        // 2. Size validation per monitor
        let can_support_grid = monitor.can_support_grid(
            DEFAULT_GRID_COLS,
            DEFAULT_GRID_ROWS,
            MIN_CELL_WIDTH,
            MIN_CELL_HEIGHT
        );

        let should_reject = monitor.should_reject(MIN_MONITOR_HEIGHT);

        println!("      Can support 3x2 grid: {}", can_support_grid);
        println!("      Should reject (too small): {}", should_reject);

        if can_support_grid && !should_reject {
            let cell_w = monitor.work_area.w / (DEFAULT_GRID_COLS as i32);
            let cell_h = monitor.work_area.h / (DEFAULT_GRID_ROWS as i32);
            println!("      Cell size would be: {}x{}", cell_w, cell_h);
        }
    }

    // 3. Verify we have at least one usable monitor
    let usable_monitors: Vec<_> = monitors
        .iter()
        .filter(|m| {
            m.can_support_grid(
                DEFAULT_GRID_COLS,
                DEFAULT_GRID_ROWS,
                MIN_CELL_WIDTH,
                MIN_CELL_HEIGHT
            ) && !m.should_reject(MIN_MONITOR_HEIGHT)
        })
        .collect();

    if usable_monitors.is_empty() {
        return Err("No monitors meet the minimum requirements for grid positioning".into());
    }

    println!("   {} monitors are suitable for grid positioning", usable_monitors.len());

    // 4. Window management test
    println!("2. Testing window management...");
    match window::get_active_window() {
        Ok(window_info) => {
            println!("   Active window: \"{}\"", window_info.title);
            println!(
                "   Position: {}x{} at ({}, {})",
                window_info.rect.w,
                window_info.rect.h,
                window_info.rect.x,
                window_info.rect.y
            );
            println!("   Resizable: {}", window_info.is_resizable);
            println!("   Maximized: {}", window_info.is_maximized);
            println!(
                "   Suitable for positioning: {}",
                window::is_window_suitable_for_positioning(window_info.handle)
            );
        }
        Err(e) => {
            println!("   No active window available: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates window positioning with a simple test
fn demo_window_positioning() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Window Positioning Demo ===");

    let window_info = window::get_active_window()?;

    if !window::is_window_suitable_for_positioning(window_info.handle) {
        return Err("Active window is not suitable for positioning".into());
    }

    let monitors = monitors::enumerate_monitors()?;
    let primary_monitor = monitors
        .iter()
        .find(|m| m.is_primary)
        .ok_or("No primary monitor found")?;

    // Calculate a simple left-half position
    let target_rect = Rect::new(
        primary_monitor.work_area.x,
        primary_monitor.work_area.y,
        primary_monitor.work_area.w / 2,
        primary_monitor.work_area.h
    );

    println!("Positioning window to left half of primary monitor...");
    if window_info.is_maximized {
        println!("Window is maximized - will restore first, then position");
    }
    println!(
        "Target rectangle: {}x{} at ({}, {})",
        target_rect.w,
        target_rect.h,
        target_rect.x,
        target_rect.y
    );

    window::position_window(window_info.handle, target_rect)?;
    println!("Window positioned successfully!");

    Ok(())
}

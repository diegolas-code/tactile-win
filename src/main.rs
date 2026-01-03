//! Tactile-Win: Phase 1 - Infrastructure Implementation
//! 
//! This is the main entry point for the application.
//! Phase 1 Focus: DPI awareness, monitor enumeration, and basic window management.

use windows::Win32::Foundation::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

mod domain;
mod platform;

use domain::core::Rect;
use platform::{monitors, window};

// Phase 1 Constants
const DEFAULT_GRID_COLS: u32 = 3;
const DEFAULT_GRID_ROWS: u32 = 2;
const MIN_CELL_WIDTH: i32 = 480;
const MIN_CELL_HEIGHT: i32 = 360;
const MIN_MONITOR_HEIGHT: i32 = 600;

fn main() -> windows::core::Result<()> {
    // CRITICAL: Set DPI awareness before any other Windows API calls
    // This ensures our application gets real pixel coordinates instead of scaled ones
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
    }

    println!("Tactile-Win Phase 1: Infrastructure");
    println!("DPI Awareness initialized successfully");

    // Phase 1 Validation: Monitor enumeration and size validation
    match run_phase1_validation() {
        Ok(_) => println!("Phase 1 validation completed successfully!"),
        Err(e) => {
            eprintln!("Phase 1 validation failed: {}", e);
            return Err(windows::core::Error::from_win32());
        }
    }

    // Phase 1 Demo: Position active window if available
    if let Err(e) = demo_window_positioning() {
        println!("Window positioning demo failed: {}", e);
        println!("(This is expected if no suitable active window is available)");
    }

    Ok(())
}

/// Validates Phase 1 infrastructure components
fn run_phase1_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Phase 1 Validation ===");
    
    // 1. Monitor enumeration
    println!("1. Enumerating monitors...");
    let monitors = monitors::enumerate_monitors()?;
    println!("   Found {} monitors", monitors.len());
    
    for (i, monitor) in monitors.iter().enumerate() {
        println!("   Monitor {}: {}x{} at ({}, {}) - DPI: {:.1}x - Primary: {}", 
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
            let cell_w = monitor.work_area.w / DEFAULT_GRID_COLS as i32;
            let cell_h = monitor.work_area.h / DEFAULT_GRID_ROWS as i32;
            println!("      Cell size would be: {}x{}", cell_w, cell_h);
        }
    }
    
    // 3. Verify we have at least one usable monitor
    let usable_monitors: Vec<_> = monitors.iter()
        .filter(|m| m.can_support_grid(DEFAULT_GRID_COLS, DEFAULT_GRID_ROWS, MIN_CELL_WIDTH, MIN_CELL_HEIGHT) 
                    && !m.should_reject(MIN_MONITOR_HEIGHT))
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
            println!("   Position: {}x{} at ({}, {})", 
                window_info.rect.w, window_info.rect.h,
                window_info.rect.x, window_info.rect.y
            );
            println!("   Resizable: {}", window_info.is_resizable);
            println!("   Maximized: {}", window_info.is_maximized);
            println!("   Suitable for positioning: {}", 
                window::is_window_suitable_for_positioning(window_info.handle));
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
    let primary_monitor = monitors.iter()
        .find(|m| m.is_primary)
        .ok_or("No primary monitor found")?;
    
    // Calculate a simple left-half position
    let target_rect = Rect::new(
        primary_monitor.work_area.x,
        primary_monitor.work_area.y,
        primary_monitor.work_area.w / 2,
        primary_monitor.work_area.h,
    );
    
    println!("Positioning window to left half of primary monitor...");
    if window_info.is_maximized {
        println!("Window is maximized - will restore first, then position");
    }
    println!("Target rectangle: {}x{} at ({}, {})", 
        target_rect.w, target_rect.h, target_rect.x, target_rect.y);
    
    window::position_window(window_info.handle, target_rect)?;
    println!("Window positioned successfully!");
    
    Ok(())
}

//! Monitor enumeration and DPI-aware coordinate handling
//!
//! This module is responsible for:
//! - Enumerating all connected monitors
//! - Getting DPI information for each monitor  
//! - Converting logical coordinates to real pixels
//! - Providing work area information (excluding taskbar)
//!
//! CRITICAL: This module must handle the Windows virtual coordinate system
//! where secondary monitors can have negative coordinates.

use crate::domain::core::Rect;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::HiDpi::*;

/// Represents a monitor with all necessary information for grid calculations
#[derive(Debug, Clone)]
pub struct Monitor {
    /// Windows handle to the monitor
    pub handle: HMONITOR,
    /// Zero-based index for stable identification
    pub index: usize,
    /// Physical rectangle in real pixels (DPI-normalized)
    pub physical_rect: Rect,
    /// Work area in real pixels (excluding taskbar)
    pub work_area: Rect,
    /// DPI scale factor (1.0 = 96 DPI, 1.25 = 120 DPI, etc.)
    pub dpi_scale: f32,
    /// Raw DPI values
    pub dpi_x: u32,
    pub dpi_y: u32,
    /// Whether this is the primary monitor
    pub is_primary: bool,
}

impl Monitor {
    /// Returns true if this monitor can support a grid with the given dimensions
    /// ensuring each cell meets the minimum size requirement
    pub fn can_support_grid(
        &self,
        grid_cols: u32,
        grid_rows: u32,
        min_cell_width: i32,
        min_cell_height: i32,
    ) -> bool {
        let cell_width = self.work_area.w / grid_cols as i32;
        let cell_height = self.work_area.h / grid_rows as i32;

        cell_width >= min_cell_width && cell_height >= min_cell_height
    }

    /// Returns true if this monitor should be rejected due to size constraints
    pub fn should_reject(&self, min_height: i32) -> bool {
        self.work_area.h < min_height
    }
}

/// Error types for monitor operations
#[derive(Debug)]
pub enum MonitorError {
    /// Failed to enumerate monitors
    EnumerationFailed,
    /// Failed to get monitor information
    InfoFailed(HMONITOR),
    /// Failed to get DPI information
    DpiFailed(HMONITOR),
    /// No monitors found during enumeration
    NoMonitors,
    /// Monitor not found at specified location
    MonitorNotFound,
    /// Failed to lookup monitor information
    MonitorLookupFailed,
}

impl std::fmt::Display for MonitorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorError::EnumerationFailed => write!(f, "Failed to enumerate monitors"),
            MonitorError::InfoFailed(handle) => {
                write!(f, "Failed to get info for monitor {:?}", handle)
            }
            MonitorError::DpiFailed(handle) => {
                write!(f, "Failed to get DPI for monitor {:?}", handle)
            }
            MonitorError::NoMonitors => write!(f, "No monitors found during enumeration"),
            MonitorError::MonitorNotFound => write!(f, "Monitor not found at specified location"),
            MonitorError::MonitorLookupFailed => write!(f, "Failed to lookup monitor information"),
        }
    }
}

impl std::error::Error for MonitorError {}

/// Context for monitor enumeration callback
struct EnumContext {
    monitors: Vec<Monitor>,
    next_index: usize,
}

/// Callback function for monitor enumeration
///
/// **Resilience Strategy**: This callback continues enumeration even if individual
/// monitors fail to provide complete information (monitor info or DPI data).
/// **Decision**: We prefer to continue with partial data rather than abort the
/// entire enumeration process. This ensures the application remains functional
/// even with problematic monitors or drivers.
unsafe extern "system" fn enum_monitor_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    unsafe {
        let context = &mut *(lparam.0 as *mut EnumContext);

        // Get monitor info
        let mut monitor_info = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        if GetMonitorInfoW(hmonitor, &mut monitor_info.monitorInfo) == FALSE {
            // Continue enumeration even if one monitor fails - prefer partial data over complete failure
            return TRUE;
        }

        // Get DPI information
        let mut dpi_x: u32 = 96;
        let mut dpi_y: u32 = 96;

        if GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y).is_err() {
            // Fallback to system DPI if per-monitor DPI fails - continue with reasonable defaults
            dpi_x = 96;
            dpi_y = 96;
        }

        // Convert Windows RECT to our domain Rect (these are already in real pixels due to DPI awareness)
        let physical_rect = Rect::new(
            monitor_info.monitorInfo.rcMonitor.left,
            monitor_info.monitorInfo.rcMonitor.top,
            monitor_info.monitorInfo.rcMonitor.right - monitor_info.monitorInfo.rcMonitor.left,
            monitor_info.monitorInfo.rcMonitor.bottom - monitor_info.monitorInfo.rcMonitor.top,
        );

        let work_area = Rect::new(
            monitor_info.monitorInfo.rcWork.left,
            monitor_info.monitorInfo.rcWork.top,
            monitor_info.monitorInfo.rcWork.right - monitor_info.monitorInfo.rcWork.left,
            monitor_info.monitorInfo.rcWork.bottom - monitor_info.monitorInfo.rcWork.top,
        );

        let is_primary = (monitor_info.monitorInfo.dwFlags & 1) != 0; // MONITORINFOF_PRIMARY = 1
        let dpi_scale = dpi_x as f32 / 96.0;

        let monitor = Monitor {
            handle: hmonitor,
            index: context.next_index,
            physical_rect,
            work_area,
            dpi_scale,
            dpi_x,
            dpi_y,
            is_primary,
        };

        context.monitors.push(monitor);
        context.next_index += 1;

        TRUE // Continue enumeration
    }
}

/// Enumerates all monitors and returns them with DPI-aware coordinates
pub fn enumerate_monitors() -> Result<Vec<Monitor>, MonitorError> {
    let mut context = EnumContext {
        monitors: Vec::new(),
        next_index: 0,
    };

    unsafe {
        if EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitor_proc),
            LPARAM(&mut context as *mut _ as isize),
        ) == FALSE
        {
            return Err(MonitorError::EnumerationFailed);
        }
    }

    if context.monitors.is_empty() {
        return Err(MonitorError::NoMonitors);
    }

    // Sort monitors by index to ensure consistent ordering
    context.monitors.sort_by_key(|m| m.index);

    Ok(context.monitors)
}

/// Gets the monitor containing the specified point
pub fn get_monitor_from_point(x: i32, y: i32) -> Result<Monitor, MonitorError> {
    let point = POINT { x, y };

    unsafe {
        let hmonitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        if hmonitor.is_invalid() {
            return Err(MonitorError::MonitorNotFound);
        }

        // Get all monitors and find the one with matching handle
        let monitors = enumerate_monitors()?;
        monitors
            .into_iter()
            .find(|m| m.handle == hmonitor)
            .ok_or(MonitorError::MonitorLookupFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_enumerate_monitors() {
        let result = enumerate_monitors();
        assert!(result.is_ok(), "Should be able to enumerate monitors");

        let monitors = result.unwrap();
        assert!(!monitors.is_empty(), "Should find at least one monitor");

        // Verify primary monitor exists
        assert!(
            monitors.iter().any(|m| m.is_primary),
            "Should have a primary monitor"
        );

        // Verify indices are sequential
        for (i, monitor) in monitors.iter().enumerate() {
            assert_eq!(monitor.index, i, "Monitor indices should be sequential");
        }
    }

    #[test]
    fn monitor_grid_validation() {
        let monitor = Monitor {
            handle: HMONITOR(0),
            index: 0,
            physical_rect: Rect::new(0, 0, 1920, 1080),
            work_area: Rect::new(0, 0, 1920, 1040), // 40px taskbar
            dpi_scale: 1.0,
            dpi_x: 96,
            dpi_y: 96,
            is_primary: true,
        };

        // 3x2 grid with 480x360 minimum should work
        assert!(monitor.can_support_grid(3, 2, 480, 360));

        // 5x5 grid with 480x360 minimum should not work
        assert!(!monitor.can_support_grid(5, 5, 480, 360));
    }

    #[test]
    fn monitor_rejection_logic() {
        let small_monitor = Monitor {
            handle: HMONITOR(0),
            index: 0,
            physical_rect: Rect::new(0, 0, 800, 600),
            work_area: Rect::new(0, 0, 800, 560),
            dpi_scale: 1.0,
            dpi_x: 96,
            dpi_y: 96,
            is_primary: true,
        };

        // Should be rejected if minimum height is 600 (work_area.h = 560 < 600)
        assert!(small_monitor.should_reject(600));
        assert!(small_monitor.should_reject(700));
    }
}

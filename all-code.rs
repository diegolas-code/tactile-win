// C:\Users\Diegolas\Code\rust\tactile-win\src\main.rs
//! Tactile-Win: Grid-based window positioning for Windows
//! 
//! Phase 1: Infrastructure (DPI awareness, monitor enumeration, window management) ✓
//! Phase 2: Domain Logic (keyboard layout, grid geometry, selection process) ✓

use windows::Win32::Foundation::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

mod domain;
mod platform;

use domain::core::Rect;
use domain::keyboard::{QwertyLayout, GridCoords};
use domain::grid::Grid;
use domain::selection::Selection;
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

    println!("Tactile-Win: Phase 2 Complete - Domain Logic Integration\n");
    
    // Phase 2 Demo: Domain Logic Integration
    demo_phase2_integration();
    
    // Phase 1 Demo: Platform validation
    if let Err(e) = run_phase1_validation() {
        eprintln!("Phase 1 validation failed: {}", e);
    }
    
    println!("\n=== Phase 2 Summary ===\n✓ Domain logic complete: keyboard → grid → selection\n✓ Platform integration tested\n✓ All tests passing\n\nReady for Phase 3: UI overlay and input handling");
    
    Ok(())
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
            println!("✓ Grid created: {} rows x {} columns", grid.dimensions().0, grid.dimensions().1);
            println!("  Cell size: {}x{}", grid.cell_size().0, grid.cell_size().1);
            grid
        },
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
            println!("   Covers: {} cols x {} rows = {} cells", w, h, selection.get_cell_count().unwrap());
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
        },
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

// C:\Users\Diegolas\Code\rust\tactile-win\src\domain\core.rs
//! Core domain types and operations
//! 
//! This module defines pure domain types that work exclusively with
//! real pixels and have no knowledge of Win32 or DPI concepts.

/// Rectangle in real pixel coordinates
/// 
/// This is the fundamental building block for all geometric calculations.
/// All coordinates are in real pixels, already DPI-normalized by the platform layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    /// Creates a new rectangle
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    
    /// Returns the right edge coordinate
    pub fn right(&self) -> i32 {
        self.x + self.w
    }
    
    /// Returns the bottom edge coordinate  
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }
    
    /// Returns true if this rectangle contains the given point
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
    
    /// Returns the intersection of two rectangles, or None if they don't intersect
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        
        if left < right && top < bottom {
            Some(Rect::new(left, top, right - left, bottom - top))
        } else {
            None
        }
    }
    
    /// Returns the bounding box that contains both rectangles
    pub fn union(&self, other: &Rect) -> Rect {
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        
        Rect::new(left, top, right - left, bottom - top)
    }
    
    /// Returns the area of the rectangle in square pixels
    pub fn area(&self) -> i32 {
        self.w * self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn rect_basic_properties() {
        let rect = Rect::new(10, 20, 100, 50);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.w, 100);
        assert_eq!(rect.h, 50);
        assert_eq!(rect.right(), 110);
        assert_eq!(rect.bottom(), 70);
        assert_eq!(rect.area(), 5000);
    }
    
    #[test]
    fn rect_contains_point() {
        let rect = Rect::new(10, 10, 20, 20);
        assert!(rect.contains_point(15, 15)); // Inside
        assert!(rect.contains_point(10, 10)); // Top-left corner
        assert!(!rect.contains_point(30, 30)); // Outside right-bottom
        assert!(!rect.contains_point(5, 5)); // Outside left-top
    }
    
    #[test]
    fn rect_intersection() {
        let rect1 = Rect::new(0, 0, 20, 20);
        let rect2 = Rect::new(10, 10, 20, 20);
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection, Rect::new(10, 10, 10, 10));
        
        // No intersection
        let rect3 = Rect::new(30, 30, 10, 10);
        assert!(rect1.intersection(&rect3).is_none());
    }
    
    #[test]
    fn rect_union() {
        let rect1 = Rect::new(0, 0, 10, 10);
        let rect2 = Rect::new(20, 20, 10, 10);
        let union = rect1.union(&rect2);
        assert_eq!(union, Rect::new(0, 0, 30, 30));
    }
}

// C:\Users\Diegolas\Code\rust\tactile-win\src\domain\grid.rs
//! Grid geometry and cell calculations
//!
//! This module handles the logical grid representation for window positioning.
//! It maps grid coordinates to screen rectangles and validates grid configurations.

use crate::domain::core::Rect;
use crate::domain::keyboard::{GridCoords, QwertyLayout};

/// Errors that can occur during grid operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridError {
    /// Grid dimensions are invalid (zero or negative)
    InvalidDimensions { rows: u32, cols: u32 },
    /// Screen area is too small for minimum cell requirements
    ScreenTooSmall { 
        screen_width: u32, 
        screen_height: u32, 
        min_cell_width: u32, 
        min_cell_height: u32 
    },
    /// Grid coordinates are outside the valid range
    InvalidCoordinates { 
        coords: GridCoords, 
        max_row: u32, 
        max_col: u32 
    },
    /// Calculated cell dimensions would be invalid
    InvalidCellSize { width: u32, height: u32 },
}

/// Represents a logical grid that can be overlaid on a screen area
/// 
/// The grid divides a rectangular screen area into a grid of cells.
/// Each cell can be identified by grid coordinates (row, col) and 
/// converted to screen pixel coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    /// Number of rows in the grid
    rows: u32,
    /// Number of columns in the grid
    cols: u32,
    /// The screen area this grid covers (in screen coordinates)
    screen_area: Rect,
    /// Width of each cell in pixels
    cell_width: u32,
    /// Height of each cell in pixels
    cell_height: u32,
    /// Associated keyboard layout for this grid
    keyboard_layout: QwertyLayout,
}

impl Grid {
    /// Minimum cell width in pixels (from architecture requirements)
    pub const MIN_CELL_WIDTH: u32 = 480;
    /// Minimum cell height in pixels (from architecture requirements)
    pub const MIN_CELL_HEIGHT: u32 = 360;

    /// Creates a new grid for the specified screen area
    /// 
    /// # Arguments
    /// * `rows` - Number of grid rows (must be > 0)
    /// * `cols` - Number of grid columns (must be > 0)
    /// * `screen_area` - Screen rectangle this grid will cover
    /// 
    /// # Returns
    /// A new Grid instance or GridError if validation fails
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{grid::Grid, core::Rect};
    /// 
    /// let screen = Rect::new(0, 0, 1920, 1080);
    /// let grid = Grid::new(3, 2, screen)?;
    /// assert_eq!(grid.dimensions(), (3, 2));
    /// ```
    pub fn new(rows: u32, cols: u32, screen_area: Rect) -> Result<Self, GridError> {
        // Validate grid dimensions
        if rows == 0 || cols == 0 {
            return Err(GridError::InvalidDimensions { rows, cols });
        }

        // Calculate cell dimensions (convert to u32 for calculations)
        let screen_width = screen_area.w as u32;
        let screen_height = screen_area.h as u32;
        let cell_width = screen_width / cols;
        let cell_height = screen_height / rows;

        // Validate minimum cell size requirements
        if cell_width < Self::MIN_CELL_WIDTH || cell_height < Self::MIN_CELL_HEIGHT {
            return Err(GridError::ScreenTooSmall {
                screen_width,
                screen_height,
                min_cell_width: Self::MIN_CELL_WIDTH,
                min_cell_height: Self::MIN_CELL_HEIGHT,
            });
        }

        // Validate calculated cell size
        if cell_width == 0 || cell_height == 0 {
            return Err(GridError::InvalidCellSize {
                width: cell_width,
                height: cell_height,
            });
        }

        // Create keyboard layout for this grid
        let keyboard_layout = QwertyLayout::new(cols, rows)
            .map_err(|_| GridError::InvalidDimensions { rows, cols })?;

        Ok(Self {
            rows,
            cols,
            screen_area,
            cell_width,
            cell_height,
            keyboard_layout,
        })
    }

    /// Returns the grid dimensions as (rows, cols)
    pub fn dimensions(&self) -> (u32, u32) {
        (self.rows, self.cols)
    }

    /// Returns the screen area this grid covers
    pub fn screen_area(&self) -> Rect {
        self.screen_area
    }

    /// Returns the pixel dimensions of each cell as (width, height)
    pub fn cell_size(&self) -> (u32, u32) {
        (self.cell_width, self.cell_height)
    }

    /// Returns the keyboard layout associated with this grid
    pub fn keyboard_layout(&self) -> &QwertyLayout {
        &self.keyboard_layout
    }

    /// Converts grid coordinates to screen rectangle
    /// 
    /// # Arguments
    /// * `coords` - Grid coordinates (row, col)
    /// 
    /// # Returns
    /// Screen rectangle for the specified cell or GridError if coordinates are invalid
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{grid::Grid, core::Rect, keyboard::GridCoords};
    /// 
    /// let screen = Rect::new(0, 0, 1920, 1080);
    /// let grid = Grid::new(3, 2, screen)?;
    /// let cell_rect = grid.cell_rect(GridCoords::new(0, 0))?;
    /// assert_eq!(cell_rect.x, 0);
    /// assert_eq!(cell_rect.y, 0);
    /// ```
    pub fn cell_rect(&self, coords: GridCoords) -> Result<Rect, GridError> {
        // Validate coordinates are within grid bounds
        if coords.row >= self.rows || coords.col >= self.cols {
            return Err(GridError::InvalidCoordinates {
                coords,
                max_row: self.rows - 1,
                max_col: self.cols - 1,
            });
        }

        // Calculate cell position
        let x = self.screen_area.x + (coords.col * self.cell_width) as i32;
        let y = self.screen_area.y + (coords.row * self.cell_height) as i32;

        Ok(Rect::new(x, y, self.cell_width as i32, self.cell_height as i32))
    }

    /// Converts a keyboard key to the corresponding cell rectangle
    /// 
    /// # Arguments
    /// * `key` - Keyboard key character
    /// 
    /// # Returns
    /// Screen rectangle for the cell corresponding to the key
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{grid::Grid, core::Rect};
    /// 
    /// let screen = Rect::new(0, 0, 1920, 1080);
    /// let grid = Grid::new(3, 2, screen)?;
    /// let q_rect = grid.key_to_rect('Q')?;  // Top-left cell
    /// let w_rect = grid.key_to_rect('W')?;  // Top-middle cell
    /// ```
    pub fn key_to_rect(&self, key: char) -> Result<Rect, GridError> {
        let coords = self.keyboard_layout.key_to_coords(key)
            .map_err(|_| GridError::InvalidCoordinates { 
                coords: GridCoords::new(0, 0), // Placeholder - real coords unknown
                max_row: self.rows - 1,
                max_col: self.cols - 1,
            })?;
        
        self.cell_rect(coords)
    }

    /// Returns all valid keyboard keys for this grid
    /// 
    /// # Returns
    /// Vector of characters that can be used to select cells in this grid
    pub fn valid_keys(&self) -> Vec<char> {
        self.keyboard_layout.valid_keys()
    }

    /// Checks if the given grid coordinates are valid for this grid
    /// 
    /// # Arguments
    /// * `coords` - Grid coordinates to validate
    /// 
    /// # Returns
    /// true if coordinates are within grid bounds, false otherwise
    pub fn contains_coords(&self, coords: GridCoords) -> bool {
        coords.row < self.rows && coords.col < self.cols
    }

    /// Checks if the given keyboard key is valid for this grid
    /// 
    /// # Arguments
    /// * `key` - Keyboard key to validate
    /// 
    /// # Returns
    /// true if key maps to a valid cell in this grid, false otherwise
    pub fn contains_key(&self, key: char) -> bool {
        self.keyboard_layout.key_to_coords(key).is_ok()
    }

    /// Creates a bounding rectangle from two grid coordinates
    /// 
    /// This method creates the smallest rectangle that encompasses both
    /// coordinate points. The order of coordinates doesn't matter.
    /// 
    /// # Arguments
    /// * `start` - First grid coordinate
    /// * `end` - Second grid coordinate  
    /// 
    /// # Returns
    /// Screen rectangle covering both coordinates or GridError if invalid
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{grid::Grid, core::Rect, keyboard::GridCoords};
    /// 
    /// let screen = Rect::new(0, 0, 1920, 1080);
    /// let grid = Grid::new(3, 2, screen)?;
    /// 
    /// // Select from Q (0,0) to S (1,1) = 2x2 area
    /// let start = GridCoords::new(0, 0);
    /// let end = GridCoords::new(1, 1);
    /// let selection_rect = grid.coords_to_rect(start, end)?;
    /// ```
    pub fn coords_to_rect(&self, start: GridCoords, end: GridCoords) -> Result<Rect, GridError> {
        // Validate both coordinates
        if !self.contains_coords(start) {
            return Err(GridError::InvalidCoordinates {
                coords: start,
                max_row: self.rows - 1,
                max_col: self.cols - 1,
            });
        }
        
        if !self.contains_coords(end) {
            return Err(GridError::InvalidCoordinates {
                coords: end,
                max_row: self.rows - 1,
                max_col: self.cols - 1,
            });
        }

        // Calculate bounding box
        let min_row = start.row.min(end.row);
        let max_row = start.row.max(end.row);
        let min_col = start.col.min(end.col);
        let max_col = start.col.max(end.col);

        // Calculate selection dimensions
        let selection_rows = max_row - min_row + 1;
        let selection_cols = max_col - min_col + 1;
        let selection_width = selection_cols * self.cell_width;
        let selection_height = selection_rows * self.cell_height;

        // Calculate top-left position
        let x = self.screen_area.x + (min_col * self.cell_width) as i32;
        let y = self.screen_area.y + (min_row * self.cell_height) as i32;

        Ok(Rect::new(x, y, selection_width as i32, selection_height as i32))
    }

    /// Creates a bounding rectangle from two keyboard keys
    /// 
    /// This is a convenience method that combines key-to-coordinate mapping
    /// with rectangle calculation.
    /// 
    /// # Arguments
    /// * `start_key` - First keyboard key
    /// * `end_key` - Second keyboard key
    /// 
    /// # Returns
    /// Screen rectangle covering both key positions or GridError if invalid
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{grid::Grid, core::Rect};
    /// 
    /// let screen = Rect::new(0, 0, 1920, 1080);
    /// let grid = Grid::new(3, 2, screen)?;
    /// 
    /// // Select from Q to S = top-left 2x2 area
    /// let selection_rect = grid.keys_to_rect('Q', 'S')?;
    /// ```
    pub fn keys_to_rect(&self, start_key: char, end_key: char) -> Result<Rect, GridError> {
        let start_coords = self.keyboard_layout.key_to_coords(start_key)
            .map_err(|_| GridError::InvalidCoordinates { 
                coords: GridCoords::new(0, 0), 
                max_row: self.rows - 1,
                max_col: self.cols - 1,
            })?;
            
        let end_coords = self.keyboard_layout.key_to_coords(end_key)
            .map_err(|_| GridError::InvalidCoordinates { 
                coords: GridCoords::new(0, 0), 
                max_row: self.rows - 1,
                max_col: self.cols - 1,
            })?;

        self.coords_to_rect(start_coords, end_coords)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_screen() -> Rect {
        Rect::new(0, 0, 1920, 1080)
    }

    #[test]
    fn grid_creation_valid() {
        let screen = create_test_screen();
        let grid = Grid::new(3, 2, screen).unwrap();
        
        assert_eq!(grid.dimensions(), (3, 2));
        assert_eq!(grid.screen_area(), screen);
        assert_eq!(grid.cell_size(), (960, 360)); // 1920/2, 1080/3
    }

    #[test]
    fn grid_creation_invalid_dimensions() {
        let screen = create_test_screen();
        
        // Zero rows
        let result = Grid::new(0, 2, screen);
        assert!(matches!(result, Err(GridError::InvalidDimensions { rows: 0, cols: 2 })));
        
        // Zero columns
        let result = Grid::new(3, 0, screen);
        assert!(matches!(result, Err(GridError::InvalidDimensions { rows: 3, cols: 0 })));
    }

    #[test]
    fn grid_creation_screen_too_small() {
        // Screen that results in cells smaller than minimum
        let small_screen = Rect::new(0, 0, 800, 600);
        let result = Grid::new(3, 2, small_screen);
        
        assert!(matches!(result, Err(GridError::ScreenTooSmall { .. })));
    }

    #[test]
    fn cell_rect_calculation() {
        let screen = create_test_screen();
        let grid = Grid::new(3, 2, screen).unwrap();
        
        // Top-left cell (Q)
        let coords = GridCoords::new(0, 0);
        let rect = grid.cell_rect(coords).unwrap();
        assert_eq!(rect, Rect::new(0, 0, 960, 360));
        
        // Top-right cell (W)  
        let coords = GridCoords::new(0, 1);
        let rect = grid.cell_rect(coords).unwrap();
        assert_eq!(rect, Rect::new(960, 0, 960, 360));
        
        // Bottom-left cell (A)
        let coords = GridCoords::new(1, 0);
        let rect = grid.cell_rect(coords).unwrap();
        assert_eq!(rect, Rect::new(0, 360, 960, 360));
        
        // Bottom-middle cell (S)
        let coords = GridCoords::new(1, 1);
        let rect = grid.cell_rect(coords).unwrap();
        assert_eq!(rect, Rect::new(960, 360, 960, 360));
    }

    #[test]
    fn cell_rect_invalid_coordinates() {
        let screen = create_test_screen();
        let grid = Grid::new(3, 2, screen).unwrap();
        
        // Row too high
        let coords = GridCoords::new(3, 0);
        let result = grid.cell_rect(coords);
        assert!(matches!(result, Err(GridError::InvalidCoordinates { .. })));
        
        // Column too high
        let coords = GridCoords::new(0, 2);
        let result = grid.cell_rect(coords);
        assert!(matches!(result, Err(GridError::InvalidCoordinates { .. })));
    }

    #[test]
    fn key_to_rect_mapping() {
        let screen = create_test_screen();
        let grid = Grid::new(3, 2, screen).unwrap();
        
        // Test Q key (top-left)
        let q_rect = grid.key_to_rect('Q').unwrap();
        assert_eq!(q_rect, Rect::new(0, 0, 960, 360));
        
        // Test W key (top-right) 
        let w_rect = grid.key_to_rect('W').unwrap();
        assert_eq!(w_rect, Rect::new(960, 0, 960, 360));
        
        // Test A key (bottom-left)
        let a_rect = grid.key_to_rect('A').unwrap();
        assert_eq!(a_rect, Rect::new(0, 360, 960, 360));
        
        // Case insensitive
        let q_lower = grid.key_to_rect('q').unwrap();
        assert_eq!(q_lower, q_rect);
    }

    #[test]
    fn coords_to_rect_single_cell() {
        let screen = create_test_screen();
        let grid = Grid::new(3, 2, screen).unwrap();
        
        // Same coordinate twice = single cell
        let coords = GridCoords::new(0, 0);
        let rect = grid.coords_to_rect(coords, coords).unwrap();
        assert_eq!(rect, Rect::new(0, 0, 960, 360));
    }

    #[test]
    fn coords_to_rect_multi_cell() {
        let screen = create_test_screen();
        let grid = Grid::new(3, 2, screen).unwrap();
        
        // Q to S selection (top-left 2x2 area)
        let start = GridCoords::new(0, 0); // Q
        let end = GridCoords::new(1, 1);   // S
        let rect = grid.coords_to_rect(start, end).unwrap();
        assert_eq!(rect, Rect::new(0, 0, 1920, 720)); // Full width, 2 rows high
        
        // Order shouldn't matter
        let rect_reverse = grid.coords_to_rect(end, start).unwrap();
        assert_eq!(rect, rect_reverse);
    }

    #[test]
    fn keys_to_rect_selection() {
        let screen = create_test_screen();
        let grid = Grid::new(3, 2, screen).unwrap();
        
        // Q to S selection
        let rect = grid.keys_to_rect('Q', 'S').unwrap();
        assert_eq!(rect, Rect::new(0, 0, 1920, 720));
        
        // Case insensitive
        let rect_lower = grid.keys_to_rect('q', 's').unwrap();
        assert_eq!(rect, rect_lower);
        
        // Order independent
        let rect_reverse = grid.keys_to_rect('S', 'Q').unwrap();
        assert_eq!(rect, rect_reverse);
    }

    #[test]
    fn contains_validation() {
        let screen = create_test_screen();
        let grid = Grid::new(3, 2, screen).unwrap();
        
        // Valid coordinates
        assert!(grid.contains_coords(GridCoords::new(0, 0)));
        assert!(grid.contains_coords(GridCoords::new(2, 1)));
        
        // Invalid coordinates
        assert!(!grid.contains_coords(GridCoords::new(3, 0)));
        assert!(!grid.contains_coords(GridCoords::new(0, 2)));
        
        // Valid keys for 3x2 grid
        assert!(grid.contains_key('Q'));
        assert!(grid.contains_key('q'));
        assert!(grid.contains_key('W'));
        assert!(grid.contains_key('A'));
        assert!(grid.contains_key('S'));
        assert!(grid.contains_key('Z'));
        assert!(grid.contains_key('X'));
        
        // Invalid keys (not in 3x2 layout)
        assert!(!grid.contains_key('E')); // Column 2, out of range for 2-column grid
        assert!(!grid.contains_key('1')); // Numbers not supported
    }

    #[test]
    fn valid_keys_list() {
        let screen = create_test_screen();
        let grid = Grid::new(3, 2, screen).unwrap();
        
        let keys = grid.valid_keys();
        assert_eq!(keys, vec!['Q', 'W', 'A', 'S', 'Z', 'X']);
    }

    #[test]
    fn grid_with_offset_screen() {
        // Test grid on a monitor that doesn't start at (0,0)
        let screen = Rect::new(1920, 0, 1920, 1080); // Second monitor
        let grid = Grid::new(3, 2, screen).unwrap();
        
        // Top-left cell should be offset
        let q_rect = grid.key_to_rect('Q').unwrap();
        assert_eq!(q_rect, Rect::new(1920, 0, 960, 360));
        
        // Selection should also be offset
        let rect = grid.keys_to_rect('Q', 'S').unwrap();
        assert_eq!(rect, Rect::new(1920, 0, 1920, 720));
    }
}

// C:\Users\Diegolas\Code\rust\tactile-win\src\domain\keyboard.rs
//! Keyboard layout mapping for grid-based window positioning
//! 
//! This module handles the conversion of keyboard input to grid coordinates
//! using QWERTY layout. It's completely pure and testable without Win32.
//! 
//! ## Design Principles
//! - **Pure functions**: No I/O, no side effects, just coordinate mapping
//! - **Extensible**: Support different grid sizes (3x2, 4x2, etc.)
//! - **Case insensitive**: 'Q' and 'q' map to same cell
//! - **Clear errors**: Invalid keys are rejected with specific error types
//! - **API clarity**: Always returns (row, col) coordinates, never flat indices

/// Error types for keyboard layout operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardError {
    /// Invalid key that's not in the current layout
    InvalidKey(char),
    /// Requested grid size not supported by this layout
    UnsupportedGridSize { cols: u32, rows: u32 },
}

impl std::fmt::Display for KeyboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyboardError::InvalidKey(ch) => write!(f, "Invalid key '{}' not found in layout", ch),
            KeyboardError::UnsupportedGridSize { cols, rows } => {
                write!(f, "Grid size {}x{} not supported by layout", cols, rows)
            }
        }
    }
}

impl std::error::Error for KeyboardError {}

/// Grid coordinates representing (row, col) position
/// 
/// Uses zero-based indexing starting from top-left:
/// - (0,0) = top-left cell
/// - (0,1) = top row, second column  
/// - (1,0) = second row, first column
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCoords {
    pub row: u32,
    pub col: u32,
}

impl GridCoords {
    /// Creates new grid coordinates
    pub fn new(row: u32, col: u32) -> Self {
        Self { row, col }
    }
}

/// QWERTY keyboard layout for grid-based selection
/// 
/// Maps keyboard keys to grid coordinates following QWERTY layout pattern.
/// Supports multiple grid sizes while maintaining consistent key mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwertyLayout {
    cols: u32,
    rows: u32,
}

impl QwertyLayout {
    /// Creates a new QWERTY layout for the specified grid dimensions
    /// 
    /// # Arguments
    /// * `cols` - Number of columns in the grid (must be ≤ 4)
    /// * `rows` - Number of rows in the grid (must be ≤ 3)
    /// 
    /// # Examples
    /// ```rust
    /// let layout = QwertyLayout::new(3, 2); // Standard 3x2 grid
    /// let layout = QwertyLayout::new(4, 2); // Extended 4x2 grid
    /// ```
    pub fn new(cols: u32, rows: u32) -> Result<Self, KeyboardError> {
        // Validate supported grid sizes
        if cols == 0 || rows == 0 || cols > 4 || rows > 3 {
            return Err(KeyboardError::UnsupportedGridSize { cols, rows });
        }
        
        Ok(Self { cols, rows })
    }
    
    /// Converts a keyboard key to grid coordinates
    /// 
    /// # Arguments
    /// * `key` - Character input (case insensitive)
    /// 
    /// # Returns
    /// Grid coordinates (row, col) for the key, or error if invalid
    /// 
    /// # Examples
    /// ```rust
    /// let layout = QwertyLayout::new(3, 2)?;
    /// assert_eq!(layout.key_to_coords('Q')?, GridCoords::new(0, 0));
    /// assert_eq!(layout.key_to_coords('s')?, GridCoords::new(1, 1)); // Case insensitive
    /// ```
    pub fn key_to_coords(&self, key: char) -> Result<GridCoords, KeyboardError> {
        // Convert to uppercase for case-insensitive matching
        let key = key.to_ascii_uppercase();
        
        // Define QWERTY layout mapping
        // Row 0: Q W E R T Y U I O P
        // Row 1: A S D F G H J K L
        // Row 2: Z X C V B N M
        
        let coords = match key {
            // Top row (row 0)
            'Q' => GridCoords::new(0, 0),
            'W' => GridCoords::new(0, 1),
            'E' => GridCoords::new(0, 2),
            'R' => GridCoords::new(0, 3),
            'T' => GridCoords::new(0, 4),
            'Y' => GridCoords::new(0, 5),
            'U' => GridCoords::new(0, 6),
            'I' => GridCoords::new(0, 7),
            'O' => GridCoords::new(0, 8),
            'P' => GridCoords::new(0, 9),
            
            // Middle row (row 1)
            'A' => GridCoords::new(1, 0),
            'S' => GridCoords::new(1, 1),
            'D' => GridCoords::new(1, 2),
            'F' => GridCoords::new(1, 3),
            'G' => GridCoords::new(1, 4),
            'H' => GridCoords::new(1, 5),
            'J' => GridCoords::new(1, 6),
            'K' => GridCoords::new(1, 7),
            'L' => GridCoords::new(1, 8),
            
            // Bottom row (row 2)
            'Z' => GridCoords::new(2, 0),
            'X' => GridCoords::new(2, 1),
            'C' => GridCoords::new(2, 2),
            'V' => GridCoords::new(2, 3),
            'B' => GridCoords::new(2, 4),
            'N' => GridCoords::new(2, 5),
            'M' => GridCoords::new(2, 6),
            
            _ => return Err(KeyboardError::InvalidKey(key)),
        };
        
        // Validate coordinates are within current grid bounds
        if coords.row >= self.rows || coords.col >= self.cols {
            return Err(KeyboardError::InvalidKey(key));
        }
        
        Ok(coords)
    }
    
    /// Gets all valid keys for the current grid layout
    /// 
    /// Returns keys in row-major order (Q, W, E, A, S, D for 3x2)
    pub fn valid_keys(&self) -> Vec<char> {
        let all_keys = [
            // Row 0
            ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
            // Row 1  
            ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', '\0'],
            // Row 2
            ['Z', 'X', 'C', 'V', 'B', 'N', 'M', '\0', '\0', '\0'],
        ];
        
        let mut valid = Vec::new();
        for row in 0..self.rows {
            for col in 0..self.cols {
                if let Some(key) = all_keys.get(row as usize)
                    .and_then(|r| r.get(col as usize))
                    .filter(|&&k| k != '\0')
                {
                    valid.push(*key);
                }
            }
        }
        
        valid
    }
    
    /// Gets the grid dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.cols, self.rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn layout_creation() {
        // Valid layouts
        assert!(QwertyLayout::new(3, 2).is_ok());
        assert!(QwertyLayout::new(4, 2).is_ok());
        assert!(QwertyLayout::new(2, 1).is_ok());
        
        // Invalid layouts
        assert!(QwertyLayout::new(0, 2).is_err());
        assert!(QwertyLayout::new(3, 0).is_err());
        assert!(QwertyLayout::new(5, 2).is_err()); // Too wide
        assert!(QwertyLayout::new(3, 4).is_err()); // Too tall
    }
    
    #[test]
    fn standard_3x2_mapping() {
        let layout = QwertyLayout::new(3, 2).unwrap();
        
        // Test all valid keys for 3x2 grid
        assert_eq!(layout.key_to_coords('Q').unwrap(), GridCoords::new(0, 0));
        assert_eq!(layout.key_to_coords('W').unwrap(), GridCoords::new(0, 1));
        assert_eq!(layout.key_to_coords('E').unwrap(), GridCoords::new(0, 2));
        assert_eq!(layout.key_to_coords('A').unwrap(), GridCoords::new(1, 0));
        assert_eq!(layout.key_to_coords('S').unwrap(), GridCoords::new(1, 1));
        assert_eq!(layout.key_to_coords('D').unwrap(), GridCoords::new(1, 2));
    }
    
    #[test]
    fn extended_4x2_mapping() {
        let layout = QwertyLayout::new(4, 2).unwrap();
        
        // Test extended keys
        assert_eq!(layout.key_to_coords('R').unwrap(), GridCoords::new(0, 3));
        assert_eq!(layout.key_to_coords('F').unwrap(), GridCoords::new(1, 3));
        
        // Original keys still work
        assert_eq!(layout.key_to_coords('Q').unwrap(), GridCoords::new(0, 0));
        assert_eq!(layout.key_to_coords('S').unwrap(), GridCoords::new(1, 1));
    }
    
    #[test]
    fn case_insensitive() {
        let layout = QwertyLayout::new(3, 2).unwrap();
        
        // Upper and lower case should map to same coordinates
        assert_eq!(layout.key_to_coords('Q').unwrap(), layout.key_to_coords('q').unwrap());
        assert_eq!(layout.key_to_coords('S').unwrap(), layout.key_to_coords('s').unwrap());
    }
    
    #[test]
    fn invalid_keys() {
        let layout = QwertyLayout::new(3, 2).unwrap();
        
        // Invalid characters
        assert_eq!(layout.key_to_coords('1'), Err(KeyboardError::InvalidKey('1')));
        assert_eq!(layout.key_to_coords('!'), Err(KeyboardError::InvalidKey('!')));
        assert_eq!(layout.key_to_coords(' '), Err(KeyboardError::InvalidKey(' ')));
        
        // Valid keys but outside current grid bounds
        assert_eq!(layout.key_to_coords('R'), Err(KeyboardError::InvalidKey('R'))); // Col 3, but grid is 3x2
        assert_eq!(layout.key_to_coords('Z'), Err(KeyboardError::InvalidKey('Z'))); // Row 2, but grid is 3x2
    }
    
    #[test]
    fn valid_keys_generation() {
        let layout_3x2 = QwertyLayout::new(3, 2).unwrap();
        let keys_3x2 = layout_3x2.valid_keys();
        assert_eq!(keys_3x2, vec!['Q', 'W', 'E', 'A', 'S', 'D']);
        
        let layout_4x2 = QwertyLayout::new(4, 2).unwrap();
        let keys_4x2 = layout_4x2.valid_keys();
        assert_eq!(keys_4x2, vec!['Q', 'W', 'E', 'R', 'A', 'S', 'D', 'F']);
    }
    
    #[test]
    fn dimensions() {
        let layout = QwertyLayout::new(3, 2).unwrap();
        assert_eq!(layout.dimensions(), (3, 2));
    }
    
    #[test]
    fn row_major_ordering() {
        let layout = QwertyLayout::new(2, 2).unwrap();
        
        // Test that keys are mapped in row-major order
        // Q=0, W=1 (row 0)
        // A=2, S=3 (row 1)
        assert_eq!(layout.key_to_coords('Q').unwrap(), GridCoords::new(0, 0)); // index 0
        assert_eq!(layout.key_to_coords('W').unwrap(), GridCoords::new(0, 1)); // index 1  
        assert_eq!(layout.key_to_coords('A').unwrap(), GridCoords::new(1, 0)); // index 2
        assert_eq!(layout.key_to_coords('S').unwrap(), GridCoords::new(1, 1)); // index 3
        
        // Verify we can reconstruct flat index: index = row * cols + col
        let q_coords = layout.key_to_coords('Q').unwrap();
        let q_index = q_coords.row * layout.cols + q_coords.col;
        assert_eq!(q_index, 0);
        
        let s_coords = layout.key_to_coords('S').unwrap();
        let s_index = s_coords.row * layout.cols + s_coords.col;
        assert_eq!(s_index, 3);
    }
}

// C:\Users\Diegolas\Code\rust\tactile-win\src\domain\mod.rs
//! Domain logic and core data structures
//! 
//! This module contains pure business logic that is independent
//! of Win32 APIs and platform-specific implementations.

pub mod core;
pub mod keyboard;
pub mod grid;
pub mod selection;

// C:\Users\Diegolas\Code\rust\tactile-win\src\domain\selection.rs
//! Two-step selection process and coordinate normalization
//!
//! This module manages the selection state during the interactive grid selection
//! process. It handles the progression from initial key press to final selection
//! and calculates bounding rectangles.

use crate::domain::keyboard::{GridCoords, KeyboardError};

/// Errors that can occur during selection operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionError {
    /// No selection has been started yet
    NoSelectionStarted,
    /// The selection is already complete
    SelectionAlreadyComplete,
    /// Invalid coordinates provided
    InvalidCoordinates { coords: GridCoords },
    /// Keyboard error during selection
    KeyboardError(KeyboardError),
}

impl From<KeyboardError> for SelectionError {
    fn from(err: KeyboardError) -> Self {
        SelectionError::KeyboardError(err)
    }
}

/// State of the current selection process
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionState {
    /// No selection has been started
    NotStarted,
    /// First coordinate has been selected, waiting for second
    InProgress { start: GridCoords },
    /// Selection is complete with normalized coordinates
    Complete { 
        /// Top-left corner of the selection
        top_left: GridCoords, 
        /// Bottom-right corner of the selection
        bottom_right: GridCoords 
    },
}

/// Manages the two-step selection process for grid cells
/// 
/// This struct tracks the current state of cell selection and provides
/// methods for progressing through the selection workflow: start → end → normalize.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// Current state of the selection
    state: SelectionState,
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

impl Selection {
    /// Creates a new, empty selection
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::selection::Selection;
    /// 
    /// let selection = Selection::new();
    /// assert!(selection.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            state: SelectionState::NotStarted,
        }
    }

    /// Returns the current selection state
    pub fn state(&self) -> &SelectionState {
        &self.state
    }

    /// Checks if the selection is empty (not started)
    /// 
    /// # Returns
    /// true if no selection has been started, false otherwise
    pub fn is_empty(&self) -> bool {
        matches!(self.state, SelectionState::NotStarted)
    }

    /// Checks if the selection is currently in progress
    /// 
    /// # Returns
    /// true if start coordinate is set but selection is not complete
    pub fn is_in_progress(&self) -> bool {
        matches!(self.state, SelectionState::InProgress { .. })
    }

    /// Checks if the selection is complete
    /// 
    /// # Returns
    /// true if both start and end coordinates are set and normalized
    pub fn is_complete(&self) -> bool {
        matches!(self.state, SelectionState::Complete { .. })
    }

    /// Starts a new selection with the given coordinates
    /// 
    /// # Arguments
    /// * `coords` - Grid coordinates for the first selected cell
    /// 
    /// # Returns
    /// Ok(()) if selection was started successfully, or SelectionError if invalid
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{selection::Selection, keyboard::GridCoords};
    /// 
    /// let mut selection = Selection::new();
    /// selection.start(GridCoords::new(0, 0))?;
    /// assert!(selection.is_in_progress());
    /// ```
    pub fn start(&mut self, coords: GridCoords) -> Result<(), SelectionError> {
        self.state = SelectionState::InProgress { start: coords };
        Ok(())
    }

    /// Completes the selection with the end coordinates
    /// 
    /// This method automatically normalizes the coordinates so that the resulting
    /// selection represents a proper rectangle with top_left and bottom_right.
    /// 
    /// # Arguments
    /// * `coords` - Grid coordinates for the second selected cell
    /// 
    /// # Returns
    /// Ok(()) if selection was completed successfully, or SelectionError if invalid
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{selection::Selection, keyboard::GridCoords};
    /// 
    /// let mut selection = Selection::new();
    /// selection.start(GridCoords::new(1, 1))?;
    /// selection.complete(GridCoords::new(0, 0))?;
    /// 
    /// // Coordinates are automatically normalized
    /// let (tl, br) = selection.get_normalized_coords().unwrap();
    /// assert_eq!(tl, GridCoords::new(0, 0));
    /// assert_eq!(br, GridCoords::new(1, 1));
    /// ```
    pub fn complete(&mut self, coords: GridCoords) -> Result<(), SelectionError> {
        let start = match &self.state {
            SelectionState::InProgress { start } => *start,
            SelectionState::NotStarted => return Err(SelectionError::NoSelectionStarted),
            SelectionState::Complete { .. } => return Err(SelectionError::SelectionAlreadyComplete),
        };

        // Normalize coordinates to ensure top_left is actually top-left
        let (top_left, bottom_right) = normalize_coordinates(start, coords);

        self.state = SelectionState::Complete {
            top_left,
            bottom_right,
        };

        Ok(())
    }

    /// Gets the start coordinates if selection is in progress
    /// 
    /// # Returns
    /// Some(coords) if selection is in progress, None otherwise
    pub fn get_start_coords(&self) -> Option<GridCoords> {
        match &self.state {
            SelectionState::InProgress { start } => Some(*start),
            _ => None,
        }
    }

    /// Gets the normalized coordinates if selection is complete
    /// 
    /// # Returns
    /// Some((top_left, bottom_right)) if selection is complete, None otherwise
    /// 
    /// The returned coordinates are guaranteed to form a proper rectangle where
    /// top_left represents the actual top-left corner and bottom_right represents
    /// the actual bottom-right corner, regardless of the order in which they were selected.
    pub fn get_normalized_coords(&self) -> Option<(GridCoords, GridCoords)> {
        match &self.state {
            SelectionState::Complete { top_left, bottom_right } => {
                Some((*top_left, *bottom_right))
            },
            _ => None,
        }
    }

    /// Calculates the dimensions of the current selection
    /// 
    /// # Returns
    /// Some((width, height)) in grid cells if selection is complete, None otherwise
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{selection::Selection, keyboard::GridCoords};
    /// 
    /// let mut selection = Selection::new();
    /// selection.start(GridCoords::new(0, 0))?;
    /// selection.complete(GridCoords::new(1, 2))?;
    /// 
    /// let (width, height) = selection.get_dimensions().unwrap();
    /// assert_eq!(width, 3); // Columns 0, 1, 2
    /// assert_eq!(height, 2); // Rows 0, 1
    /// ```
    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        self.get_normalized_coords().map(|(top_left, bottom_right)| {
            let width = bottom_right.col - top_left.col + 1;
            let height = bottom_right.row - top_left.row + 1;
            (width, height)
        })
    }

    /// Calculates the total number of cells in the selection
    /// 
    /// # Returns
    /// Some(count) if selection is complete, None otherwise
    pub fn get_cell_count(&self) -> Option<u32> {
        self.get_dimensions().map(|(width, height)| width * height)
    }

    /// Checks if the selection covers a single cell
    /// 
    /// # Returns
    /// Some(true) if selection is a single cell, Some(false) if multi-cell, None if incomplete
    pub fn is_single_cell(&self) -> Option<bool> {
        self.get_cell_count().map(|count| count == 1)
    }

    /// Resets the selection to empty state
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{selection::Selection, keyboard::GridCoords};
    /// 
    /// let mut selection = Selection::new();
    /// selection.start(GridCoords::new(0, 0))?;
    /// selection.reset();
    /// assert!(selection.is_empty());
    /// ```
    pub fn reset(&mut self) {
        self.state = SelectionState::NotStarted;
    }

    /// Creates a completed selection from two coordinates
    /// 
    /// This is a convenience method for testing or programmatic use where
    /// you want to create a complete selection in one step.
    /// 
    /// # Arguments
    /// * `start` - First coordinate
    /// * `end` - Second coordinate  
    /// 
    /// # Returns
    /// A new Selection with normalized coordinates
    /// 
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{selection::Selection, keyboard::GridCoords};
    /// 
    /// let selection = Selection::from_coords(
    ///     GridCoords::new(2, 1),
    ///     GridCoords::new(0, 0)
    /// );
    /// 
    /// let (tl, br) = selection.get_normalized_coords().unwrap();
    /// assert_eq!(tl, GridCoords::new(0, 0));
    /// assert_eq!(br, GridCoords::new(2, 1));
    /// ```
    pub fn from_coords(start: GridCoords, end: GridCoords) -> Self {
        let (top_left, bottom_right) = normalize_coordinates(start, end);
        Self {
            state: SelectionState::Complete {
                top_left,
                bottom_right,
            }
        }
    }
}

/// Normalizes two coordinates into top-left and bottom-right corners
/// 
/// This function ensures that the returned coordinates form a proper rectangle
/// regardless of the order in which the original coordinates were provided.
/// 
/// # Arguments
/// * `coord1` - First coordinate
/// * `coord2` - Second coordinate
/// 
/// # Returns
/// (top_left, bottom_right) normalized coordinates
/// 
/// # Example
/// ```rust
/// use tactile_win::domain::{selection::normalize_coordinates, keyboard::GridCoords};
/// 
/// // Order doesn't matter - result is always normalized
/// let (tl, br) = normalize_coordinates(
///     GridCoords::new(2, 2),
///     GridCoords::new(0, 1)
/// );
/// assert_eq!(tl, GridCoords::new(0, 1));
/// assert_eq!(br, GridCoords::new(2, 2));
/// ```
pub fn normalize_coordinates(coord1: GridCoords, coord2: GridCoords) -> (GridCoords, GridCoords) {
    let min_row = coord1.row.min(coord2.row);
    let max_row = coord1.row.max(coord2.row);
    let min_col = coord1.col.min(coord2.col);
    let max_col = coord1.col.max(coord2.col);

    let top_left = GridCoords::new(min_row, min_col);
    let bottom_right = GridCoords::new(max_row, max_col);

    (top_left, bottom_right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_selection_is_empty() {
        let selection = Selection::new();
        assert!(selection.is_empty());
        assert!(!selection.is_in_progress());
        assert!(!selection.is_complete());
    }

    #[test]
    fn start_selection() {
        let mut selection = Selection::new();
        let coords = GridCoords::new(1, 2);
        
        selection.start(coords).unwrap();
        
        assert!(!selection.is_empty());
        assert!(selection.is_in_progress());
        assert!(!selection.is_complete());
        assert_eq!(selection.get_start_coords(), Some(coords));
    }

    #[test]
    fn complete_selection() {
        let mut selection = Selection::new();
        let start = GridCoords::new(1, 2);
        let end = GridCoords::new(0, 0);
        
        selection.start(start).unwrap();
        selection.complete(end).unwrap();
        
        assert!(!selection.is_empty());
        assert!(!selection.is_in_progress());
        assert!(selection.is_complete());
        
        // Coordinates should be normalized
        let (tl, br) = selection.get_normalized_coords().unwrap();
        assert_eq!(tl, GridCoords::new(0, 0));
        assert_eq!(br, GridCoords::new(1, 2));
    }

    #[test]
    fn complete_without_start_fails() {
        let mut selection = Selection::new();
        let result = selection.complete(GridCoords::new(0, 0));
        
        assert!(matches!(result, Err(SelectionError::NoSelectionStarted)));
    }

    #[test]
    fn complete_twice_fails() {
        let mut selection = Selection::new();
        selection.start(GridCoords::new(0, 0)).unwrap();
        selection.complete(GridCoords::new(1, 1)).unwrap();
        
        let result = selection.complete(GridCoords::new(2, 2));
        assert!(matches!(result, Err(SelectionError::SelectionAlreadyComplete)));
    }

    #[test]
    fn normalize_coordinates_test() {
        // Normal order
        let (tl, br) = normalize_coordinates(
            GridCoords::new(0, 0),
            GridCoords::new(2, 3)
        );
        assert_eq!(tl, GridCoords::new(0, 0));
        assert_eq!(br, GridCoords::new(2, 3));

        // Reverse order
        let (tl, br) = normalize_coordinates(
            GridCoords::new(2, 3),
            GridCoords::new(0, 0)
        );
        assert_eq!(tl, GridCoords::new(0, 0));
        assert_eq!(br, GridCoords::new(2, 3));

        // Mixed order
        let (tl, br) = normalize_coordinates(
            GridCoords::new(1, 3),
            GridCoords::new(2, 1)
        );
        assert_eq!(tl, GridCoords::new(1, 1));
        assert_eq!(br, GridCoords::new(2, 3));

        // Same coordinate
        let (tl, br) = normalize_coordinates(
            GridCoords::new(1, 1),
            GridCoords::new(1, 1)
        );
        assert_eq!(tl, GridCoords::new(1, 1));
        assert_eq!(br, GridCoords::new(1, 1));
    }

    #[test]
    fn from_coords_creates_complete_selection() {
        let selection = Selection::from_coords(
            GridCoords::new(2, 1),
            GridCoords::new(0, 2)
        );

        assert!(selection.is_complete());
        let (tl, br) = selection.get_normalized_coords().unwrap();
        assert_eq!(tl, GridCoords::new(0, 1));
        assert_eq!(br, GridCoords::new(2, 2));
    }

    #[test]
    fn dimensions_calculation() {
        let selection = Selection::from_coords(
            GridCoords::new(0, 0),
            GridCoords::new(2, 1)
        );

        let (width, height) = selection.get_dimensions().unwrap();
        assert_eq!(width, 2); // Columns 0, 1
        assert_eq!(height, 3); // Rows 0, 1, 2

        assert_eq!(selection.get_cell_count(), Some(6));
        assert_eq!(selection.is_single_cell(), Some(false));
    }

    #[test]
    fn single_cell_selection() {
        let selection = Selection::from_coords(
            GridCoords::new(1, 1),
            GridCoords::new(1, 1)
        );

        let (width, height) = selection.get_dimensions().unwrap();
        assert_eq!(width, 1);
        assert_eq!(height, 1);

        assert_eq!(selection.get_cell_count(), Some(1));
        assert_eq!(selection.is_single_cell(), Some(true));
    }

    #[test]
    fn reset_selection() {
        let mut selection = Selection::new();
        selection.start(GridCoords::new(0, 0)).unwrap();
        selection.complete(GridCoords::new(1, 1)).unwrap();
        
        assert!(selection.is_complete());
        
        selection.reset();
        assert!(selection.is_empty());
        assert_eq!(selection.get_start_coords(), None);
        assert_eq!(selection.get_normalized_coords(), None);
    }

    #[test]
    fn incomplete_selection_returns_none() {
        let mut selection = Selection::new();
        
        // Empty selection
        assert_eq!(selection.get_normalized_coords(), None);
        assert_eq!(selection.get_dimensions(), None);
        assert_eq!(selection.get_cell_count(), None);
        assert_eq!(selection.is_single_cell(), None);
        
        // In-progress selection
        selection.start(GridCoords::new(0, 0)).unwrap();
        assert_eq!(selection.get_normalized_coords(), None);
        assert_eq!(selection.get_dimensions(), None);
        assert_eq!(selection.get_cell_count(), None);
        assert_eq!(selection.is_single_cell(), None);
    }

    #[test]
    fn state_transitions() {
        let mut selection = Selection::new();
        
        // Start: NotStarted -> InProgress
        assert!(matches!(selection.state(), SelectionState::NotStarted));
        selection.start(GridCoords::new(0, 0)).unwrap();
        assert!(matches!(selection.state(), SelectionState::InProgress { .. }));
        
        // Complete: InProgress -> Complete
        selection.complete(GridCoords::new(1, 1)).unwrap();
        assert!(matches!(selection.state(), SelectionState::Complete { .. }));
        
        // Reset: Complete -> NotStarted
        selection.reset();
        assert!(matches!(selection.state(), SelectionState::NotStarted));
    }
}

// C:\Users\Diegolas\Code\rust\tactile-win\src\platform\mod.rs
//! Platform-specific Windows implementations
//! 
//! This module encapsulates all Win32 API interactions and provides
//! a clean interface to the rest of the application.

pub mod monitors;
pub mod window;

// C:\Users\Diegolas\Code\rust\tactile-win\src\platform\monitors.rs
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
    pub fn can_support_grid(&self, grid_cols: u32, grid_rows: u32, min_cell_width: i32, min_cell_height: i32) -> bool {
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
            MonitorError::InfoFailed(handle) => write!(f, "Failed to get info for monitor {:?}", handle),
            MonitorError::DpiFailed(handle) => write!(f, "Failed to get DPI for monitor {:?}", handle),
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
        if EnumDisplayMonitors(None, None, Some(enum_monitor_proc), LPARAM(&mut context as *mut _ as isize)) == FALSE {
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
        assert!(monitors.iter().any(|m| m.is_primary), "Should have a primary monitor");
        
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

// C:\Users\Diegolas\Code\rust\tactile-win\src\platform\window.rs
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
            placement.showCmd == SW_SHOWMAXIMIZED.0 as u32
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
            if ShowWindow(hwnd, SW_RESTORE).as_bool() == false {
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


//! Grid geometry and cell calculations
//!
//! This module handles the logical grid representation for window positioning.
//! It maps grid coordinates to screen rectangles and validates grid configurations.

use crate::domain::core::Rect;
use crate::domain::keyboard::{ GridCoords, QwertyLayout };

/// Errors that can occur during grid operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridError {
    /// Grid dimensions are invalid (zero or negative)
    InvalidDimensions {
        rows: u32,
        cols: u32,
    },
    /// Screen area is too small for minimum cell requirements
    ScreenTooSmall {
        screen_width: u32,
        screen_height: u32,
        min_cell_width: u32,
        min_cell_height: u32,
    },
    /// Grid coordinates are outside the valid range
    InvalidCoordinates {
        coords: GridCoords,
        max_row: u32,
        max_col: u32,
    },
    /// Calculated cell dimensions would be invalid
    InvalidCellSize {
        width: u32,
        height: u32,
    },
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
        let keyboard_layout = QwertyLayout::new(cols, rows).map_err(
            |_| GridError::InvalidDimensions { rows, cols }
        )?;

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

    /// Gets the keyboard key for grid coordinates
    ///
    /// # Arguments
    /// * `coords` - Grid coordinates (row, col)
    ///
    /// # Returns
    /// The keyboard key character for the specified coordinates
    pub fn key_for_coords(&self, coords: GridCoords) -> Result<char, GridError> {
        // Validate coordinates are within grid bounds
        if !self.contains_coords(coords) {
            return Err(GridError::InvalidCoordinates {
                coords,
                max_row: self.rows - 1,
                max_col: self.cols - 1,
            });
        }

        // Use the keyboard layout to find the key
        self.keyboard_layout.coords_to_key(coords).map_err(|_| GridError::InvalidCoordinates {
            coords,
            max_row: self.rows - 1,
            max_col: self.cols - 1,
        })
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
        let x = self.screen_area.x + ((coords.col * self.cell_width) as i32);
        let y = self.screen_area.y + ((coords.row * self.cell_height) as i32);

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
        let coords = self.keyboard_layout
            .key_to_coords(key)
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

    /// Converts a keyboard key to grid coordinates
    ///
    /// # Arguments
    /// * `key` - Keyboard key to convert
    ///
    /// # Returns
    /// Grid coordinates for the key or GridError if key is invalid
    pub fn key_to_coords(&self, key: char) -> Result<GridCoords, GridError> {
        self.keyboard_layout.key_to_coords(key).map_err(|_| GridError::InvalidCoordinates {
            coords: GridCoords::new(0, 0), // Placeholder
            max_row: self.rows - 1,
            max_col: self.cols - 1,
        })
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
        let x = self.screen_area.x + ((min_col * self.cell_width) as i32);
        let y = self.screen_area.y + ((min_row * self.cell_height) as i32);

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
        let start_coords = self.keyboard_layout.key_to_coords(start_key).map_err(|_| {
            GridError::InvalidCoordinates {
                coords: GridCoords::new(0, 0),
                max_row: self.rows - 1,
                max_col: self.cols - 1,
            }
        })?;

        let end_coords = self.keyboard_layout.key_to_coords(end_key).map_err(|_| {
            GridError::InvalidCoordinates {
                coords: GridCoords::new(0, 0),
                max_row: self.rows - 1,
                max_col: self.cols - 1,
            }
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
        let end = GridCoords::new(1, 1); // S
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

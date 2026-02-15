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
/// 
/// The layout uses a "bottom-up" fill strategy: keys are always assigned
/// starting from the bottom keyboard row (Z row) and working upward as
/// grid height increases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwertyLayout {
    cols: u32,
    rows: u32,
}

impl QwertyLayout {
    /// Maximum keyboard rows available (numbers, Q, A, Z rows)
    const MAX_KEYBOARD_ROWS: u32 = 4;

    /// Creates a new QWERTY layout for the specified grid dimensions
    ///
    /// # Arguments
    /// * `cols` - Number of columns in the grid (must be ≤ 10)
    /// * `rows` - Number of rows in the grid (must be ≤ 4)
    ///
    /// # Examples
    /// ```rust
    /// let layout = QwertyLayout::new(3, 2); // 2-row grid (A and Z rows)
    /// let layout = QwertyLayout::new(4, 4); // Full 4-row grid (numbers, Q, A, Z)
    /// ```
    pub fn new(cols: u32, rows: u32) -> Result<Self, KeyboardError> {
        // Validate supported grid sizes
        if cols == 0 || rows == 0 || cols > 10 || rows > 4 {
            return Err(KeyboardError::UnsupportedGridSize { cols, rows });
        }

        Ok(Self { cols, rows })
    }

    /// Returns the keyboard layout definition
    /// 
    /// Row 0: 1 2 3 4 5 6 7 8 9 0 (number row)
    /// Row 1: Q W E R T Y U I O P
    /// Row 2: A S D F G H J K L
    /// Row 3: Z X C V B N M
    fn keyboard_rows() -> [&'static [char]; 4] {
        [
            &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
            &['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
            &['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'],
            &['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
        ]
    }

    /// Maps a screen row to a keyboard row using bottom-up filling
    /// 
    /// Grid always fills from bottom keyboard row (row 3) upward:
    /// - 2-row grid: uses keyboard rows 2, 3 (A, Z)
    /// - 3-row grid: uses keyboard rows 1, 2, 3 (Q, A, Z)
    /// - 4-row grid: uses keyboard rows 0, 1, 2, 3 (numbers, Q, A, Z)
    fn screen_row_to_keyboard_row(screen_row: u32, grid_rows: u32) -> u32 {
        let rows_to_skip = Self::MAX_KEYBOARD_ROWS - grid_rows;
        rows_to_skip + screen_row
    }

    /// Inverse mapping: keyboard row to screen row
    /// Returns None if the keyboard row is not displayed in the current grid
    fn keyboard_row_to_screen_row(keyboard_row: u32, grid_rows: u32) -> Option<u32> {
        let rows_to_skip = Self::MAX_KEYBOARD_ROWS - grid_rows;
        if keyboard_row < rows_to_skip {
            None
        } else {
            Some(keyboard_row - rows_to_skip)
        }
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
    /// // 2-row grid shows A and Z rows (bottom-up)
    /// assert_eq!(layout.key_to_coords('A')?, GridCoords::new(0, 0));
    /// assert_eq!(layout.key_to_coords('z')?, GridCoords::new(1, 0)); // Case insensitive
    /// ```
    pub fn key_to_coords(&self, key: char) -> Result<GridCoords, KeyboardError> {
        let key = key.to_ascii_uppercase();
        let keyboard_rows = Self::keyboard_rows();

        // Find the key in the keyboard layout
        for (keyboard_row_idx, keyboard_row) in keyboard_rows.iter().enumerate() {
            if let Some(col) = keyboard_row.iter().position(|&k| k == key) {
                // Check if this column is within grid bounds
                if col >= self.cols as usize {
                    return Err(KeyboardError::InvalidKey(key));
                }

                // Map keyboard row to screen row using bottom-up strategy
                let keyboard_row = keyboard_row_idx as u32;
                if let Some(screen_row) = Self::keyboard_row_to_screen_row(keyboard_row, self.rows) {
                    return Ok(GridCoords::new(screen_row, col as u32));
                } else {
                    // Key exists but is not visible in current grid height
                    return Err(KeyboardError::InvalidKey(key));
                }
            }
        }

        Err(KeyboardError::InvalidKey(key))
    }

    /// Gets all valid keys for the current grid layout
    ///
    /// Returns keys in row-major order from top to bottom of the screen
    pub fn valid_keys(&self) -> Vec<char> {
        let keyboard_rows = Self::keyboard_rows();
        let mut valid = Vec::new();

        for screen_row in 0..self.rows {
            let keyboard_row = Self::screen_row_to_keyboard_row(screen_row, self.rows);
            let keys = keyboard_rows[keyboard_row as usize];
            
            for col in 0..self.cols {
                if let Some(&key) = keys.get(col as usize) {
                    valid.push(key);
                }
            }
        }

        valid
    }

    /// Gets the grid dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.cols, self.rows)
    }

    /// Converts grid coordinates to the corresponding keyboard key
    ///
    /// # Arguments
    /// * `coords` - Grid coordinates (row, col)
    ///
    /// # Returns
    /// The keyboard key for the specified coordinates
    pub fn coords_to_key(&self, coords: GridCoords) -> Result<char, KeyboardError> {
        // Validate coordinates are within layout bounds
        if coords.row >= self.rows || coords.col >= self.cols {
            return Err(KeyboardError::InvalidKey('\0'));
        }

        // Map screen row to keyboard row
        let keyboard_row = Self::screen_row_to_keyboard_row(coords.row, self.rows);
        let keyboard_rows = Self::keyboard_rows();

        keyboard_rows
            .get(keyboard_row as usize)
            .and_then(|row| row.get(coords.col as usize))
            .copied()
            .ok_or(KeyboardError::InvalidKey('\0'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_creation() {
        // Valid layouts
        assert!(QwertyLayout::new(3, 2).is_ok());
        assert!(QwertyLayout::new(10, 4).is_ok()); // Maximum size
        assert!(QwertyLayout::new(2, 1).is_ok());

        // Invalid layouts
        assert!(QwertyLayout::new(0, 2).is_err());
        assert!(QwertyLayout::new(3, 0).is_err());
        assert!(QwertyLayout::new(11, 2).is_err()); // Too wide
        assert!(QwertyLayout::new(3, 5).is_err()); // Too tall
    }

    #[test]
    fn bottom_up_2_rows() {
        // 2-row grid should show A and Z rows (keyboard rows 2 and 3)
        let layout = QwertyLayout::new(3, 2).unwrap();

        // Screen row 0 = keyboard row 2 (A row)
        assert_eq!(layout.key_to_coords('A').unwrap(), GridCoords::new(0, 0));
        assert_eq!(layout.key_to_coords('S').unwrap(), GridCoords::new(0, 1));
        assert_eq!(layout.key_to_coords('D').unwrap(), GridCoords::new(0, 2));

        // Screen row 1 = keyboard row 3 (Z row)
        assert_eq!(layout.key_to_coords('Z').unwrap(), GridCoords::new(1, 0));
        assert_eq!(layout.key_to_coords('X').unwrap(), GridCoords::new(1, 1));
        assert_eq!(layout.key_to_coords('C').unwrap(), GridCoords::new(1, 2));

        // Q row should not be visible
        assert!(layout.key_to_coords('Q').is_err());
        // Number row should not be visible
        assert!(layout.key_to_coords('1').is_err());
    }

    #[test]
    fn bottom_up_3_rows() {
        // 3-row grid should show Q, A, and Z rows (keyboard rows 1, 2, 3)
        let layout = QwertyLayout::new(3, 3).unwrap();

        // Screen row 0 = keyboard row 1 (Q row)
        assert_eq!(layout.key_to_coords('Q').unwrap(), GridCoords::new(0, 0));
        assert_eq!(layout.key_to_coords('W').unwrap(), GridCoords::new(0, 1));
        assert_eq!(layout.key_to_coords('E').unwrap(), GridCoords::new(0, 2));

        // Screen row 1 = keyboard row 2 (A row)
        assert_eq!(layout.key_to_coords('A').unwrap(), GridCoords::new(1, 0));
        assert_eq!(layout.key_to_coords('S').unwrap(), GridCoords::new(1, 1));
        assert_eq!(layout.key_to_coords('D').unwrap(), GridCoords::new(1, 2));

        // Screen row 2 = keyboard row 3 (Z row)
        assert_eq!(layout.key_to_coords('Z').unwrap(), GridCoords::new(2, 0));
        assert_eq!(layout.key_to_coords('X').unwrap(), GridCoords::new(2, 1));
        assert_eq!(layout.key_to_coords('C').unwrap(), GridCoords::new(2, 2));

        // Number row should not be visible
        assert!(layout.key_to_coords('1').is_err());
    }

    #[test]
    fn bottom_up_4_rows() {
        // 4-row grid should show all rows (keyboard rows 0, 1, 2, 3)
        let layout = QwertyLayout::new(3, 4).unwrap();

        // Screen row 0 = keyboard row 0 (number row)
        assert_eq!(layout.key_to_coords('1').unwrap(), GridCoords::new(0, 0));
        assert_eq!(layout.key_to_coords('2').unwrap(), GridCoords::new(0, 1));
        assert_eq!(layout.key_to_coords('3').unwrap(), GridCoords::new(0, 2));

        // Screen row 1 = keyboard row 1 (Q row)
        assert_eq!(layout.key_to_coords('Q').unwrap(), GridCoords::new(1, 0));
        assert_eq!(layout.key_to_coords('W').unwrap(), GridCoords::new(1, 1));
        assert_eq!(layout.key_to_coords('E').unwrap(), GridCoords::new(1, 2));

        // Screen row 2 = keyboard row 2 (A row)
        assert_eq!(layout.key_to_coords('A').unwrap(), GridCoords::new(2, 0));
        assert_eq!(layout.key_to_coords('S').unwrap(), GridCoords::new(2, 1));
        assert_eq!(layout.key_to_coords('D').unwrap(), GridCoords::new(2, 2));

        // Screen row 3 = keyboard row 3 (Z row)
        assert_eq!(layout.key_to_coords('Z').unwrap(), GridCoords::new(3, 0));
        assert_eq!(layout.key_to_coords('X').unwrap(), GridCoords::new(3, 1));
        assert_eq!(layout.key_to_coords('C').unwrap(), GridCoords::new(3, 2));
    }

    #[test]
    fn case_insensitive() {
        let layout = QwertyLayout::new(3, 2).unwrap();

        // Upper and lower case should map to same coordinates
        assert_eq!(
            layout.key_to_coords('A').unwrap(),
            layout.key_to_coords('a').unwrap()
        );
        assert_eq!(
            layout.key_to_coords('Z').unwrap(),
            layout.key_to_coords('z').unwrap()
        );
    }

    #[test]
    fn extended_columns() {
        // Test with more columns
        let layout = QwertyLayout::new(7, 2).unwrap();

        // 2-row grid, 7 cols shows A row and Z row
        // A row: A(0) S(1) D(2) F(3) G(4) H(5) J(6)
        assert_eq!(layout.key_to_coords('A').unwrap(), GridCoords::new(0, 0));
        assert_eq!(layout.key_to_coords('G').unwrap(), GridCoords::new(0, 4));
        assert_eq!(layout.key_to_coords('J').unwrap(), GridCoords::new(0, 6)); // 7th col
        // Z row: Z(0) X(1) C(2) V(3) B(4) N(5) M(6)
        assert_eq!(layout.key_to_coords('Z').unwrap(), GridCoords::new(1, 0));
        assert_eq!(layout.key_to_coords('B').unwrap(), GridCoords::new(1, 4));
        assert_eq!(layout.key_to_coords('M').unwrap(), GridCoords::new(1, 6));
    }

    #[test]
    fn number_row_full_width() {
        // Test all 10 numbers in a 4-row grid
        let layout = QwertyLayout::new(10, 4).unwrap();

        assert_eq!(layout.key_to_coords('1').unwrap(), GridCoords::new(0, 0));
        assert_eq!(layout.key_to_coords('5').unwrap(), GridCoords::new(0, 4));
        assert_eq!(layout.key_to_coords('0').unwrap(), GridCoords::new(0, 9));
    }

    #[test]
    fn invalid_keys() {
        let layout = QwertyLayout::new(3, 2).unwrap();

        // Invalid characters
        assert!(layout.key_to_coords('!').is_err());
        assert!(layout.key_to_coords(' ').is_err());

        // Valid keys but outside current grid bounds (col too large)
        assert!(layout.key_to_coords('F').is_err()); // Col 3, but grid is 3 cols

        // Valid keys but not visible in 2-row grid (Q and numbers not shown)
        assert!(layout.key_to_coords('Q').is_err());
        assert!(layout.key_to_coords('1').is_err());
    }

    #[test]
    fn valid_keys_generation() {
        // 2-row, 3-col grid: A,S,D and Z,X,C
        let layout_3x2 = QwertyLayout::new(3, 2).unwrap();
        let keys_3x2 = layout_3x2.valid_keys();
        assert_eq!(keys_3x2, vec!['A', 'S', 'D', 'Z', 'X', 'C']);

        // 3-row, 3-col grid: Q,W,E and A,S,D and Z,X,C
        let layout_3x3 = QwertyLayout::new(3, 3).unwrap();
        let keys_3x3 = layout_3x3.valid_keys();
        assert_eq!(keys_3x3, vec!['Q', 'W', 'E', 'A', 'S', 'D', 'Z', 'X', 'C']);

        // 4-row, 3-col grid: 1,2,3 and Q,W,E and A,S,D and Z,X,C
        let layout_3x4 = QwertyLayout::new(3, 4).unwrap();
        let keys_3x4 = layout_3x4.valid_keys();
        assert_eq!(
            keys_3x4,
            vec!['1', '2', '3', 'Q', 'W', 'E', 'A', 'S', 'D', 'Z', 'X', 'C']
        );
    }

    #[test]
    fn dimensions() {
        let layout = QwertyLayout::new(3, 2).unwrap();
        assert_eq!(layout.dimensions(), (3, 2));
    }

    #[test]
    fn coords_to_key_roundtrip() {
        let layout = QwertyLayout::new(3, 2).unwrap();

        // For 2-row grid: screen shows A and Z rows
        let coords_a = GridCoords::new(0, 0);
        assert_eq!(layout.coords_to_key(coords_a).unwrap(), 'A');

        let coords_z = GridCoords::new(1, 0);
        assert_eq!(layout.coords_to_key(coords_z).unwrap(), 'Z');

        // Round trip
        let key = 'S';
        let coords = layout.key_to_coords(key).unwrap();
        let key_back = layout.coords_to_key(coords).unwrap();
        assert_eq!(key, key_back);
    }

    #[test]
    fn row_major_ordering() {
        let layout = QwertyLayout::new(2, 2).unwrap();

        // 2-row grid shows A and Z rows
        // A=0, S=1 (screen row 0, keyboard row 2)
        // Z=2, X=3 (screen row 1, keyboard row 3)
        assert_eq!(layout.key_to_coords('A').unwrap(), GridCoords::new(0, 0));
        assert_eq!(layout.key_to_coords('S').unwrap(), GridCoords::new(0, 1));
        assert_eq!(layout.key_to_coords('Z').unwrap(), GridCoords::new(1, 0));
        assert_eq!(layout.key_to_coords('X').unwrap(), GridCoords::new(1, 1));

        // Verify we can reconstruct flat index: index = row * cols + col
        let a_coords = layout.key_to_coords('A').unwrap();
        let a_index = a_coords.row * layout.cols + a_coords.col;
        assert_eq!(a_index, 0);

        let x_coords = layout.key_to_coords('X').unwrap();
        let x_index = x_coords.row * layout.cols + x_coords.col;
        assert_eq!(x_index, 3);
    }
}

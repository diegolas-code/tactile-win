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
                if let Some(key) = all_keys
                    .get(row as usize)
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

        // Define the keyboard layout mapping - row-major order
        let layout = [
            // Row 0
            ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
            // Row 1
            ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', '\0'],
            // Row 2
            ['Z', 'X', 'C', 'V', 'B', 'N', 'M', '\0', '\0', '\0'],
        ];

        layout
            .get(coords.row as usize)
            .and_then(|row| row.get(coords.col as usize))
            .filter(|&&key| key != '\0')
            .map(|&key| key)
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
        assert_eq!(
            layout.key_to_coords('Q').unwrap(),
            layout.key_to_coords('q').unwrap()
        );
        assert_eq!(
            layout.key_to_coords('S').unwrap(),
            layout.key_to_coords('s').unwrap()
        );
    }

    #[test]
    fn invalid_keys() {
        let layout = QwertyLayout::new(3, 2).unwrap();

        // Invalid characters
        assert_eq!(
            layout.key_to_coords('1'),
            Err(KeyboardError::InvalidKey('1'))
        );
        assert_eq!(
            layout.key_to_coords('!'),
            Err(KeyboardError::InvalidKey('!'))
        );
        assert_eq!(
            layout.key_to_coords(' '),
            Err(KeyboardError::InvalidKey(' '))
        );

        // Valid keys but outside current grid bounds
        assert_eq!(
            layout.key_to_coords('R'),
            Err(KeyboardError::InvalidKey('R'))
        ); // Col 3, but grid is 3x2
        assert_eq!(
            layout.key_to_coords('Z'),
            Err(KeyboardError::InvalidKey('Z'))
        ); // Row 2, but grid is 3x2
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

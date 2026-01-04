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
        bottom_right: GridCoords,
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
            SelectionState::NotStarted => {
                return Err(SelectionError::NoSelectionStarted);
            }
            SelectionState::Complete { .. } => {
                return Err(SelectionError::SelectionAlreadyComplete);
            }
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
            SelectionState::Complete {
                top_left,
                bottom_right,
            } => Some((*top_left, *bottom_right)),
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
        self.get_normalized_coords()
            .map(|(top_left, bottom_right)| {
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
            },
        }
    }

    /// Adds a key to the selection process
    ///
    /// This method handles the two-step selection workflow:
    /// - First key starts the selection
    /// - Second key completes the selection
    ///
    /// # Arguments
    /// * `coords` - Grid coordinates for the pressed key
    ///
    /// # Returns
    /// Ok(()) if key was processed successfully, or SelectionError if invalid
    ///
    /// # Example
    /// ```rust
    /// use tactile_win::domain::{selection::Selection, keyboard::GridCoords};
    ///
    /// let mut selection = Selection::new();
    /// selection.add_coords(GridCoords::new(0, 0))?;  // Start selection
    /// assert!(selection.is_in_progress());
    ///
    /// selection.add_coords(GridCoords::new(1, 1))?;  // Complete selection
    /// assert!(selection.is_complete());
    /// ```
    pub fn add_coords(&mut self, coords: GridCoords) -> Result<(), SelectionError> {
        match &self.state {
            SelectionState::NotStarted => self.start(coords),
            SelectionState::InProgress { .. } => self.complete(coords),
            SelectionState::Complete { .. } => Err(SelectionError::SelectionAlreadyComplete),
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
        assert!(matches!(
            result,
            Err(SelectionError::SelectionAlreadyComplete)
        ));
    }

    #[test]
    fn normalize_coordinates_test() {
        // Normal order
        let (tl, br) = normalize_coordinates(GridCoords::new(0, 0), GridCoords::new(2, 3));
        assert_eq!(tl, GridCoords::new(0, 0));
        assert_eq!(br, GridCoords::new(2, 3));

        // Reverse order
        let (tl, br) = normalize_coordinates(GridCoords::new(2, 3), GridCoords::new(0, 0));
        assert_eq!(tl, GridCoords::new(0, 0));
        assert_eq!(br, GridCoords::new(2, 3));

        // Mixed order
        let (tl, br) = normalize_coordinates(GridCoords::new(1, 3), GridCoords::new(2, 1));
        assert_eq!(tl, GridCoords::new(1, 1));
        assert_eq!(br, GridCoords::new(2, 3));

        // Same coordinate
        let (tl, br) = normalize_coordinates(GridCoords::new(1, 1), GridCoords::new(1, 1));
        assert_eq!(tl, GridCoords::new(1, 1));
        assert_eq!(br, GridCoords::new(1, 1));
    }

    #[test]
    fn from_coords_creates_complete_selection() {
        let selection = Selection::from_coords(GridCoords::new(2, 1), GridCoords::new(0, 2));

        assert!(selection.is_complete());
        let (tl, br) = selection.get_normalized_coords().unwrap();
        assert_eq!(tl, GridCoords::new(0, 1));
        assert_eq!(br, GridCoords::new(2, 2));
    }

    #[test]
    fn dimensions_calculation() {
        let selection = Selection::from_coords(GridCoords::new(0, 0), GridCoords::new(2, 1));

        let (width, height) = selection.get_dimensions().unwrap();
        assert_eq!(width, 2); // Columns 0, 1
        assert_eq!(height, 3); // Rows 0, 1, 2

        assert_eq!(selection.get_cell_count(), Some(6));
        assert_eq!(selection.is_single_cell(), Some(false));
    }

    #[test]
    fn single_cell_selection() {
        let selection = Selection::from_coords(GridCoords::new(1, 1), GridCoords::new(1, 1));

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
        assert!(matches!(
            selection.state(),
            SelectionState::InProgress { .. }
        ));

        // Complete: InProgress -> Complete
        selection.complete(GridCoords::new(1, 1)).unwrap();
        assert!(matches!(selection.state(), SelectionState::Complete { .. }));

        // Reset: Complete -> NotStarted
        selection.reset();
        assert!(matches!(selection.state(), SelectionState::NotStarted));
    }
}

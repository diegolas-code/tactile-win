//! Application state management
//!
//! Defines the core application state machine and state transitions.
//! The state is kept simple with transient selection data only.

use crate::domain::selection::Selection;
use std::time::Instant;

/// Main application state - either idle or actively selecting
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// Application is idle, waiting for hotkey activation
    Idle,
    /// User is actively selecting grid cells
    Selecting(SelectingState),
}

/// State during active selection process
///
/// This contains only transient state data. Stable configuration
/// like grids and monitors lives in AppController.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectingState {
    /// Index of currently active monitor (where letters are shown)
    pub active_monitor_index: usize,
    /// Current selection progress (start key, completion, etc.)
    pub selection: Selection,
    /// Timestamp when selection started (for 30s timeout)
    pub selection_started: Instant,
}

impl SelectingState {
    /// Creates a new selecting state
    ///
    /// # Arguments
    /// * `active_monitor_index` - Index of monitor to start selection on
    pub fn new(active_monitor_index: usize) -> Self {
        Self {
            active_monitor_index,
            selection: Selection::new(),
            selection_started: Instant::now(),
        }
    }

    /// Checks if the selection has timed out (30 seconds)
    ///
    /// # Returns
    /// true if selection should be automatically cancelled
    pub fn is_timed_out(&self) -> bool {
        self.selection_started.elapsed().as_secs() >= 30
    }

    /// Gets the remaining time before timeout
    ///
    /// # Returns
    /// Seconds remaining before automatic timeout
    pub fn remaining_timeout(&self) -> u64 {
        30_u64.saturating_sub(self.selection_started.elapsed().as_secs())
    }

    /// Switches to a different monitor during selection
    ///
    /// # Arguments  
    /// * `monitor_index` - Index of monitor to switch to
    pub fn switch_monitor(&mut self, monitor_index: usize) {
        self.active_monitor_index = monitor_index;
        // Reset selection when switching monitors
        self.selection.reset();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Possible state transition events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateEvent {
    /// Hotkey was pressed
    HotkeyPressed,
    /// Valid grid key was pressed
    KeyPressed(char),
    /// Navigation key was pressed (arrow keys)
    Navigation(NavigationDirection),
    /// Escape key was pressed or selection cancelled
    SelectionCancelled,
    /// Selection completed successfully
    SelectionCompleted,
    /// Selection timed out (30 seconds)
    SelectionTimedOut,
}

/// Navigation directions for monitor switching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDirection {
    Left,
    Right,
    Up,
    Down,
}

/// State machine for application state transitions
pub struct StateMachine;

impl StateMachine {
    /// Create a new state machine instance
    pub fn new() -> Self {
        Self
    }

    /// Processes a state event and returns the new state
    ///
    /// # Arguments
    /// * `current_state` - Current application state
    /// * `event` - Event to process
    /// * `monitor_count` - Number of available monitors for bounds checking
    ///
    /// # Returns
    /// New application state after processing the event
    pub fn process_event(
        current_state: AppState,
        event: StateEvent,
        monitor_count: usize,
    ) -> AppState {
        match (current_state, event) {
            // From Idle state
            (AppState::Idle, StateEvent::HotkeyPressed) => {
                println!("STATE MACHINE: Idle -> Selecting (starting selection on monitor 0)");
                // Start selection on primary monitor (index 0)
                AppState::Selecting(SelectingState::new(0))
            }

            // From Selecting state
            (AppState::Selecting(selecting), StateEvent::KeyPressed(_key)) => {
                // Process key press in selection
                // Note: Grid validation will happen in controller
                AppState::Selecting(selecting)
            }

            (AppState::Selecting(mut selecting), StateEvent::Navigation(direction)) => {
                // Handle monitor navigation
                let new_monitor_index = match direction {
                    NavigationDirection::Left => {
                        if selecting.active_monitor_index > 0 {
                            selecting.active_monitor_index - 1
                        } else {
                            monitor_count.saturating_sub(1) // Wrap to last monitor
                        }
                    }
                    NavigationDirection::Right => {
                        if selecting.active_monitor_index + 1 < monitor_count {
                            selecting.active_monitor_index + 1
                        } else {
                            0 // Wrap to first monitor
                        }
                    }
                    // Up/Down navigation reserved for future multi-row monitor layouts
                    NavigationDirection::Up | NavigationDirection::Down => {
                        selecting.active_monitor_index // No change for now
                    }
                };

                selecting.switch_monitor(new_monitor_index);
                AppState::Selecting(selecting)
            }

            (AppState::Selecting(_), StateEvent::SelectionCompleted) => {
                // Selection successful, return to idle
                AppState::Idle
            }

            (AppState::Selecting(_), StateEvent::SelectionCancelled) => {
                // User cancelled, return to idle
                AppState::Idle
            }

            (AppState::Selecting(_), StateEvent::SelectionTimedOut) => {
                // Automatic timeout, return to idle
                AppState::Idle
            }

            (AppState::Selecting(_), StateEvent::HotkeyPressed) => {
                // Hotkey pressed during selection = toggle off
                AppState::Idle
            }

            // Invalid transitions - ignore event
            (state, _) => state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_idle() {
        let state = AppState::default();
        assert!(matches!(state, AppState::Idle));
    }

    #[test]
    fn selecting_state_creation() {
        let selecting = SelectingState::new(1);
        assert_eq!(selecting.active_monitor_index, 1);
        assert!(selecting.selection.is_empty());
        assert!(!selecting.is_timed_out()); // Should not timeout immediately
    }

    #[test]
    fn hotkey_activates_selection() {
        let state = StateMachine::process_event(
            AppState::Idle,
            StateEvent::HotkeyPressed,
            2, // 2 monitors
        );

        assert!(matches!(state, AppState::Selecting(_)));
        if let AppState::Selecting(selecting) = state {
            assert_eq!(selecting.active_monitor_index, 0); // Starts on primary monitor
        }
    }

    #[test]
    fn navigation_switches_monitors() {
        let initial_selecting = SelectingState::new(0);
        let state = AppState::Selecting(initial_selecting);

        // Navigate right from monitor 0 to monitor 1
        let new_state = StateMachine::process_event(
            state,
            StateEvent::Navigation(NavigationDirection::Right),
            3, // 3 monitors
        );

        if let AppState::Selecting(selecting) = new_state {
            assert_eq!(selecting.active_monitor_index, 1);
        } else {
            panic!("Expected selecting state");
        }
    }

    #[test]
    fn navigation_wraps_around() {
        let initial_selecting = SelectingState::new(2); // Last monitor
        let state = AppState::Selecting(initial_selecting);

        // Navigate right should wrap to monitor 0
        let new_state = StateMachine::process_event(
            state,
            StateEvent::Navigation(NavigationDirection::Right),
            3, // 3 monitors (indices 0, 1, 2)
        );

        if let AppState::Selecting(selecting) = new_state {
            assert_eq!(selecting.active_monitor_index, 0);
        } else {
            panic!("Expected selecting state");
        }
    }

    #[test]
    fn selection_completion_returns_to_idle() {
        let selecting = SelectingState::new(0);
        let state = AppState::Selecting(selecting);

        let new_state = StateMachine::process_event(state, StateEvent::SelectionCompleted, 1);

        assert!(matches!(new_state, AppState::Idle));
    }

    #[test]
    fn escape_cancels_selection() {
        let selecting = SelectingState::new(0);
        let state = AppState::Selecting(selecting);

        let new_state = StateMachine::process_event(state, StateEvent::SelectionCancelled, 1);

        assert!(matches!(new_state, AppState::Idle));
    }

    #[test]
    fn hotkey_during_selection_toggles_off() {
        let selecting = SelectingState::new(0);
        let state = AppState::Selecting(selecting);

        let new_state = StateMachine::process_event(state, StateEvent::HotkeyPressed, 1);

        assert!(matches!(new_state, AppState::Idle));
    }

    #[test]
    fn monitor_switching_resets_selection() {
        let mut selecting = SelectingState::new(0);

        // Simulate some selection progress (this would normally be set by controller)
        // For now just verify the monitor index changes and selection gets reset
        selecting.switch_monitor(1);

        assert_eq!(selecting.active_monitor_index, 1);
        assert!(selecting.selection.is_empty());
    }
}

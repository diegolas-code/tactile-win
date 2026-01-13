//! Configuration module for tactile-win
//!
//! Phase 4 introduces a user-facing configuration surface. This module
//! concentrates the data structures shared between the UI dialog and the
//! rest of the application when applying updated grid settings.

pub mod grid;

pub use grid::{GridConfigError, GridConfigStore, MonitorGridConfig};

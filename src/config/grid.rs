use crate::domain::core::Rect;
use crate::domain::grid::{Grid, GridError};
use crate::platform::monitors::Monitor;
use thiserror::Error;

/// Orientation of a monitor computed from its work area
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenOrientation {
    Landscape,
    Portrait,
}

impl ScreenOrientation {
    pub fn from_rect(rect: &Rect) -> Self {
        if rect.w >= rect.h {
            ScreenOrientation::Landscape
        } else {
            ScreenOrientation::Portrait
        }
    }
}

/// User-facing configuration for a monitor's grid
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorGridConfig {
    pub monitor_index: usize,
    pub cols: u32,
    pub rows: u32,
    pub min_cell_width: u32,
    pub min_cell_height: u32,
}

impl MonitorGridConfig {
    pub const MIN_COLS: u32 = 2;
    pub const MIN_ROWS: u32 = 2;
    pub const MAX_ROWS: u32 = 3;
    pub const MAX_COLS: u32 = 4;
    pub const DEFAULT_MIN_CELL: u32 = 300;
    pub const MIN_CELL_LIMIT: u32 = 200;
    pub const MAX_CELL_LIMIT: u32 = 1200;

    pub fn default_for_monitor(monitor: &Monitor) -> Self {
        let orientation = ScreenOrientation::from_rect(&monitor.work_area);
        let (cols, rows) = Self::orientation_defaults(orientation);
        Self {
            monitor_index: monitor.index,
            cols,
            rows,
            min_cell_width: Self::DEFAULT_MIN_CELL,
            min_cell_height: Self::DEFAULT_MIN_CELL,
        }
    }

    pub fn orientation_defaults(orientation: ScreenOrientation) -> (u32, u32) {
        match orientation {
            ScreenOrientation::Landscape => (3, 2),
            ScreenOrientation::Portrait => (2, 3),
        }
    }

    pub fn sanitize_cell_dimension(value: u32) -> u32 {
        value.clamp(Self::MIN_CELL_LIMIT, Self::MAX_CELL_LIMIT)
    }

    pub fn reset_to_defaults(&mut self, monitor: &Monitor) {
        let (cols, rows) = Self::orientation_defaults(ScreenOrientation::from_rect(&monitor.work_area));
        self.cols = cols;
        self.rows = rows;
        self.min_cell_width = Self::DEFAULT_MIN_CELL;
        self.min_cell_height = Self::DEFAULT_MIN_CELL;
    }

    pub fn apply_bounds_from_monitor(&mut self, monitor: &Monitor) -> Result<(), GridConfigError> {
        let bounds = GridBounds::for_monitor(
            monitor,
            self.min_cell_width,
            self.min_cell_height,
        )?;
        self.cols = bounds.clamp_cols(self.cols);
        self.rows = bounds.clamp_rows(self.rows);
        Ok(())
    }

    pub fn build_grid(&self, monitor: &Monitor) -> Result<Grid, GridConfigError> {
        let sanitized_width = Self::sanitize_cell_dimension(self.min_cell_width);
        let sanitized_height = Self::sanitize_cell_dimension(self.min_cell_height);
        let bounds = GridBounds::for_monitor(monitor, sanitized_width, sanitized_height)?;
        let cols = bounds.clamp_cols(self.cols);
        let rows = bounds.clamp_rows(self.rows);

        Grid::with_min_cell_size(rows, cols, monitor.work_area, sanitized_width, sanitized_height)
            .map_err(|source| GridConfigError::GridCreationFailed {
                monitor_index: monitor.index,
                source,
            })
    }

    pub fn bounds_for_monitor(
        monitor: &Monitor,
        min_cell_width: u32,
        min_cell_height: u32,
    ) -> Result<GridBounds, GridConfigError> {
        GridBounds::for_monitor(monitor, min_cell_width, min_cell_height)
    }
}

/// Bounding information for a monitor with a specific minimum cell size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridBounds {
    pub min_cols: u32,
    pub max_cols: u32,
    pub min_rows: u32,
    pub max_rows: u32,
}

impl GridBounds {
    pub fn for_monitor(
        monitor: &Monitor,
        min_cell_width: u32,
        min_cell_height: u32,
    ) -> Result<Self, GridConfigError> {
        let width_req = MonitorGridConfig::sanitize_cell_dimension(min_cell_width);
        let height_req = MonitorGridConfig::sanitize_cell_dimension(min_cell_height);

        let max_cols_by_size = capacity_for(monitor.work_area.w, width_req);
        if max_cols_by_size < MonitorGridConfig::MIN_COLS {
            return Err(GridConfigError::MonitorTooSmall {
                monitor_index: monitor.index,
                reason: format!(
                    "needs at least {}px width to fit {} columns",
                    (MonitorGridConfig::MIN_COLS * width_req),
                    MonitorGridConfig::MIN_COLS
                ),
            });
        }

        let max_rows_by_size = capacity_for(monitor.work_area.h, height_req);
        if max_rows_by_size < MonitorGridConfig::MIN_ROWS {
            return Err(GridConfigError::MonitorTooSmall {
                monitor_index: monitor.index,
                reason: format!(
                    "needs at least {}px height to fit {} rows",
                    (MonitorGridConfig::MIN_ROWS * height_req),
                    MonitorGridConfig::MIN_ROWS
                ),
            });
        }

        Ok(Self {
            min_cols: MonitorGridConfig::MIN_COLS,
            max_cols: max_cols_by_size.min(MonitorGridConfig::MAX_COLS),
            min_rows: MonitorGridConfig::MIN_ROWS,
            max_rows: max_rows_by_size.min(MonitorGridConfig::MAX_ROWS),
        })
    }

    pub fn clamp_cols(&self, value: u32) -> u32 {
        value.clamp(self.min_cols, self.max_cols)
    }

    pub fn clamp_rows(&self, value: u32) -> u32 {
        value.clamp(self.min_rows, self.max_rows)
    }
}

#[derive(Debug, Error)]
pub enum GridConfigError {
    #[error("Configuration mismatch between monitors and stored grid settings")]
    MonitorMismatch,
    #[error("Monitor {monitor_index} cannot satisfy minimum cell size requirements: {reason}")]
    MonitorTooSmall { monitor_index: usize, reason: String },
    #[error("Grid creation failed for monitor {monitor_index}: {source}")]
    GridCreationFailed { monitor_index: usize, source: GridError },
}

/// Store that keeps per-monitor grid configuration aligned with runtime monitors
#[derive(Debug, Clone)]
pub struct GridConfigStore {
    configs: Vec<MonitorGridConfig>,
}

impl GridConfigStore {
    pub fn new(monitors: &[Monitor]) -> Result<Self, GridConfigError> {
        if monitors.is_empty() {
            return Err(GridConfigError::MonitorMismatch);
        }

        let mut configs = Vec::with_capacity(monitors.len());
        for monitor in monitors {
            let mut config = MonitorGridConfig::default_for_monitor(monitor);
            config.apply_bounds_from_monitor(monitor)?;
            configs.push(config);
        }

        Ok(Self { configs })
    }

    pub fn configs(&self) -> &[MonitorGridConfig] {
        &self.configs
    }

    pub fn config_for(&self, monitor_index: usize) -> Option<&MonitorGridConfig> {
        self.configs.get(monitor_index)
    }

    pub fn update_configs(
        &mut self,
        monitors: &[Monitor],
        updated: Vec<MonitorGridConfig>,
    ) -> Result<(), GridConfigError> {
        if updated.len() != monitors.len() || updated.len() != self.configs.len() {
            return Err(GridConfigError::MonitorMismatch);
        }

        for cfg in updated {
            let monitor_index = cfg.monitor_index;
            let monitor = monitors
                .get(monitor_index)
                .ok_or(GridConfigError::MonitorMismatch)?;
            if monitor.index != monitor_index {
                return Err(GridConfigError::MonitorMismatch);
            }

            let mut sanitized = cfg;
            sanitized.min_cell_width = MonitorGridConfig::sanitize_cell_dimension(sanitized.min_cell_width);
            sanitized.min_cell_height = MonitorGridConfig::sanitize_cell_dimension(sanitized.min_cell_height);
            sanitized.cols = sanitized.cols.max(MonitorGridConfig::MIN_COLS);
            sanitized.rows = sanitized.rows.max(MonitorGridConfig::MIN_ROWS);
            sanitized.apply_bounds_from_monitor(monitor)?;
            if let Some(slot) = self.configs.get_mut(monitor_index) {
                *slot = sanitized;
            } else {
                return Err(GridConfigError::MonitorMismatch);
            }
        }

        Ok(())
    }

    pub fn build_grids(&self, monitors: &[Monitor]) -> Result<Vec<Grid>, GridConfigError> {
        if self.configs.len() != monitors.len() {
            return Err(GridConfigError::MonitorMismatch);
        }

        let mut grids = Vec::with_capacity(monitors.len());
        for monitor in monitors {
            let config = self
                .configs
                .get(monitor.index)
                .ok_or(GridConfigError::MonitorMismatch)?;
            grids.push(config.build_grid(monitor)?);
        }

        Ok(grids)
    }
}

fn capacity_for(length: i32, min_size: u32) -> u32 {
    if length <= 0 {
        return 0;
    }

    let min_size = min_size.max(1);
    (length as u32) / min_size
}

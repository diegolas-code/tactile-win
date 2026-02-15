use crate::domain::core::Rect;
use crate::domain::grid::{ Grid, GridError };
use crate::platform::monitors::Monitor;
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

const GRID_CONFIG_FILENAME: &str = "grid_config.json";
const GRID_CAPABILITIES_FILENAME: &str = "grid_capabilities.txt";
const GRID_CAPABILITIES_VERSION: u32 = 1;
const GRID_CONFIG_VERSION: u32 = 2;
const DEFAULT_SELECTION_TIMEOUT_SECS: u64 = 30;
const MIN_SELECTION_TIMEOUT_SECS: u64 = 5;
const MAX_SELECTION_TIMEOUT_SECS: u64 = 300;

#[derive(Debug, Serialize, Deserialize)]
struct StoredGridSettings {
    rows: u32,
    cols: u32,
    min_cell_width: u32,
    min_cell_height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredMonitorEntry {
    monitor_id: String,
    grid: StoredGridSettings,
}

#[derive(Debug, Serialize, Deserialize)]
struct GridConfigFile {
    #[serde(default = "default_config_version")]
    version: u32,
    #[serde(default = "default_selection_timeout_secs")]
    selection_timeout_secs: u64,
    monitors: Vec<StoredMonitorEntry>,
}

fn default_config_version() -> u32 {
    GRID_CONFIG_VERSION
}

fn default_selection_timeout_secs() -> u64 {
    DEFAULT_SELECTION_TIMEOUT_SECS
}

fn sanitize_selection_timeout_secs(value: u64) -> u64 {
    value.clamp(MIN_SELECTION_TIMEOUT_SECS, MAX_SELECTION_TIMEOUT_SECS)
}

/// Orientation of a monitor computed from its work area
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenOrientation {
    Landscape,
    Portrait,
}

impl ScreenOrientation {
    pub fn from_rect(rect: &Rect) -> Self {
        if rect.w >= rect.h { ScreenOrientation::Landscape } else { ScreenOrientation::Portrait }
    }
}

/// User-facing configuration for a monitor's grid
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub const MAX_ROWS: u32 = 4;
    pub const MAX_COLS: u32 = 8;
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

    pub fn apply_bounds_from_monitor(&mut self, monitor: &Monitor) -> Result<(), GridConfigError> {
        let bounds = GridBounds::for_monitor(monitor, self.min_cell_width, self.min_cell_height)?;
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

        Grid::with_min_cell_size(
            rows,
            cols,
            monitor.work_area,
            sanitized_width,
            sanitized_height
        ).map_err(|source| GridConfigError::GridCreationFailed {
            monitor_index: monitor.index,
            source,
        })
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
        min_cell_height: u32
    ) -> Result<Self, GridConfigError> {
        let width_req = MonitorGridConfig::sanitize_cell_dimension(min_cell_width);
        let height_req = MonitorGridConfig::sanitize_cell_dimension(min_cell_height);

        let max_cols_by_size = capacity_for(monitor.work_area.w, width_req);
        if max_cols_by_size < MonitorGridConfig::MIN_COLS {
            return Err(GridConfigError::MonitorTooSmall {
                monitor_index: monitor.index,
                reason: format!(
                    "needs at least {}px width to fit {} columns",
                    MonitorGridConfig::MIN_COLS * width_req,
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
                    MonitorGridConfig::MIN_ROWS * height_req,
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
    #[error(
        "Monitor {monitor_index} cannot satisfy minimum cell size requirements: {reason}"
    )] MonitorTooSmall {
        monitor_index: usize,
        reason: String,
    },
    #[error("Grid creation failed for monitor {monitor_index}: {source}")] GridCreationFailed {
        monitor_index: usize,
        source: GridError,
    },
    #[error("Failed to read configuration file {path}: {source}")] ConfigReadError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to write configuration file {path}: {source}")] ConfigWriteError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to write capability file {path}: {source}")] CapabilityWriteError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to parse configuration file {path}: {source}")] ConfigParseError {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("Failed to serialize configuration: {source}")] ConfigSerializeError {
        source: serde_json::Error,
    },
}

/// Store that keeps per-monitor grid configuration aligned with runtime monitors
#[derive(Debug, Clone)]
pub struct GridConfigStore {
    configs: Vec<MonitorGridConfig>,
    config_path: PathBuf,
    monitor_ids: Vec<String>,
    selection_timeout_secs: u64,
}

impl GridConfigStore {
    pub fn new(monitors: &[Monitor]) -> Result<Self, GridConfigError> {
        if monitors.is_empty() {
            return Err(GridConfigError::MonitorMismatch);
        }

        let config_path = Self::config_file_path();
        let monitor_ids = Self::monitor_ids(monitors);

        let store = if
            let Some((configs, selection_timeout_secs, needs_save)) = Self::load_from_disk(
                monitors,
                &config_path
            )?
        {
            let store = Self {
                configs,
                config_path,
                monitor_ids,
                selection_timeout_secs,
            };
            if needs_save {
                store.save_to_disk()?;
            }
            store
        } else {
            let configs = Self::default_configs(monitors)?;
            let store = Self {
                configs,
                config_path,
                monitor_ids,
                selection_timeout_secs: DEFAULT_SELECTION_TIMEOUT_SECS,
            };
            // Persist defaults so the file exists for subsequent runs
            store.save_to_disk()?;
            store
        };

        store.ensure_capabilities_file(monitors)?;

        Ok(store)
    }

    pub fn configs(&self) -> &[MonitorGridConfig] {
        &self.configs
    }

    pub fn selection_timeout_secs(&self) -> u64 {
        self.selection_timeout_secs
    }

    pub fn build_grids(&self, monitors: &[Monitor]) -> Result<Vec<Grid>, GridConfigError> {
        if self.configs.len() != monitors.len() {
            return Err(GridConfigError::MonitorMismatch);
        }

        let mut grids = Vec::with_capacity(monitors.len());
        for monitor in monitors {
            let config = self.configs.get(monitor.index).ok_or(GridConfigError::MonitorMismatch)?;
            grids.push(config.build_grid(monitor)?);
        }

        Ok(grids)
    }

    fn default_configs(monitors: &[Monitor]) -> Result<Vec<MonitorGridConfig>, GridConfigError> {
        monitors
            .iter()
            .map(|monitor| Self::default_config_for_monitor(monitor))
            .collect()
    }

    fn config_file_path() -> PathBuf {
        PathBuf::from(GRID_CONFIG_FILENAME)
    }

    fn capabilities_file_path(config_path: &PathBuf) -> PathBuf {
        match config_path.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.join(GRID_CAPABILITIES_FILENAME),
            _ => PathBuf::from(GRID_CAPABILITIES_FILENAME),
        }
    }

    fn load_from_disk(
        monitors: &[Monitor],
        path: &PathBuf
    ) -> Result<Option<(Vec<MonitorGridConfig>, u64, bool)>, GridConfigError> {
        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(path).map_err(|source| GridConfigError::ConfigReadError {
            path: path.clone(),
            source,
        })?;

        match serde_json::from_str::<GridConfigFile>(&data) {
            Ok(file) => {
                if file.version != GRID_CONFIG_VERSION {
                    println!(
                        "Grid configuration version {} detected (expected {}). Using best-effort load.",
                        file.version,
                        GRID_CONFIG_VERSION
                    );
                }
                let (configs, sanitized) = Self::align_stored_entries(monitors, file.monitors)?;
                let timeout = sanitize_selection_timeout_secs(file.selection_timeout_secs);
                let timeout_sanitized = timeout != file.selection_timeout_secs;
                // Rewrite file if version differs or if invalid values were corrected
                let needs_save =
                    sanitized || timeout_sanitized || file.version != GRID_CONFIG_VERSION;
                Ok(Some((configs, timeout, needs_save)))
            }
            Err(primary_err) => {
                match serde_json::from_str::<Vec<MonitorGridConfig>>(&data) {
                    Ok(mut legacy_configs) => {
                        println!("Detected legacy grid_config.json format. Migrating to version {}.", GRID_CONFIG_VERSION);
                        let (configs, _) = Self::align_legacy_configs(
                            monitors,
                            &mut legacy_configs
                        )?;
                        Ok(Some((configs, DEFAULT_SELECTION_TIMEOUT_SECS, true)))
                    }
                    Err(_) =>
                        Err(GridConfigError::ConfigParseError {
                            path: path.clone(),
                            source: primary_err,
                        }),
                }
            }
        }
    }

    fn align_stored_entries(
        monitors: &[Monitor],
        entries: Vec<StoredMonitorEntry>
    ) -> Result<(Vec<MonitorGridConfig>, bool), GridConfigError> {
        let mut entry_map: HashMap<String, StoredGridSettings> = HashMap::new();
        for entry in entries {
            entry_map.insert(entry.monitor_id, entry.grid);
        }

        let mut configs = Vec::with_capacity(monitors.len());
        let mut sanitized = false;
        for monitor in monitors {
            let monitor_id = monitor_identifier(monitor);
            if let Some(settings) = entry_map.remove(&monitor_id) {
                let (config, changed) = Self::config_from_settings(monitor, settings)?;
                sanitized |= changed;
                configs.push(config);
            } else {
                sanitized = true;
                println!("Grid config for monitor {} missing in file; using defaults.", monitor_id);
                configs.push(Self::default_config_for_monitor(monitor)?);
            }
        }

        if !entry_map.is_empty() {
            sanitized = true;
            println!("Ignoring {} monitor entries not present in current system", entry_map.len());
        }

        Ok((configs, sanitized))
    }

    fn align_legacy_configs(
        monitors: &[Monitor],
        configs: &mut [MonitorGridConfig]
    ) -> Result<(Vec<MonitorGridConfig>, bool), GridConfigError> {
        let mut by_index: HashMap<usize, MonitorGridConfig> = HashMap::new();
        for cfg in configs.iter_mut() {
            let idx = cfg.monitor_index;
            if monitors.get(idx).is_some() {
                by_index.insert(idx, cfg.clone());
            }
        }

        let mut result = Vec::with_capacity(monitors.len());
        let mut sanitized = false;
        for monitor in monitors {
            if let Some(cfg) = by_index.remove(&monitor.index) {
                let settings = StoredGridSettings {
                    rows: cfg.rows,
                    cols: cfg.cols,
                    min_cell_width: cfg.min_cell_width,
                    min_cell_height: cfg.min_cell_height,
                };
                let (config, changed) = Self::config_from_settings(monitor, settings)?;
                sanitized |= changed;
                result.push(config);
            } else {
                sanitized = true;
                println!(
                    "Legacy config missing entry for monitor {}; using defaults.",
                    monitor_identifier(monitor)
                );
                result.push(Self::default_config_for_monitor(monitor)?);
            }
        }

        if !by_index.is_empty() {
            sanitized = true;
            println!(
                "Ignoring {} legacy monitor entries not present in current system",
                by_index.len()
            );
        }

        Ok((result, sanitized))
    }

    fn save_to_disk(&self) -> Result<(), GridConfigError> {
        if self.monitor_ids.len() != self.configs.len() {
            return Err(GridConfigError::MonitorMismatch);
        }

        let entries: Vec<StoredMonitorEntry> = self.monitor_ids
            .iter()
            .zip(self.configs.iter())
            .map(|(monitor_id, config)| StoredMonitorEntry {
                monitor_id: monitor_id.clone(),
                grid: StoredGridSettings {
                    rows: config.rows,
                    cols: config.cols,
                    min_cell_width: config.min_cell_width,
                    min_cell_height: config.min_cell_height,
                },
            })
            .collect();

        let file = GridConfigFile {
            version: GRID_CONFIG_VERSION,
            selection_timeout_secs: self.selection_timeout_secs,
            monitors: entries,
        };

        let data = serde_json
            ::to_string_pretty(&file)
            .map_err(|source| GridConfigError::ConfigSerializeError { source })?;

        fs::write(&self.config_path, data).map_err(|source| GridConfigError::ConfigWriteError {
            path: self.config_path.clone(),
            source,
        })
    }

    fn ensure_capabilities_file(&self, monitors: &[Monitor]) -> Result<(), GridConfigError> {
        if self.configs.len() != monitors.len() {
            return Err(GridConfigError::MonitorMismatch);
        }

        let path = Self::capabilities_file_path(&self.config_path);
        if path.exists() {
            return Ok(());
        }

        let mut buffer = String::new();
        buffer.push_str("# Tactile-Win Monitor Capabilities\n");
        buffer.push_str(&format!("capabilities_version={}\n", GRID_CAPABILITIES_VERSION));
        buffer.push_str(&format!("monitor_count={}\n\n", monitors.len()));

        for monitor in monitors {
            let config = self.configs.get(monitor.index).ok_or(GridConfigError::MonitorMismatch)?;
            let min_cell_width = MonitorGridConfig::sanitize_cell_dimension(config.min_cell_width);
            let min_cell_height = MonitorGridConfig::sanitize_cell_dimension(
                config.min_cell_height
            );
            let bounds = GridBounds::for_monitor(monitor, min_cell_width, min_cell_height)?;

            buffer.push_str(&format!("[{}]\n", monitor_identifier(monitor)));
            buffer.push_str(&format!("index={}\n", monitor.index));
            buffer.push_str(&format!("is_primary={}\n", monitor.is_primary));
            buffer.push_str(&format!("physical_width={}\n", monitor.physical_rect.w));
            buffer.push_str(&format!("physical_height={}\n", monitor.physical_rect.h));
            buffer.push_str(&format!("work_area_width={}\n", monitor.work_area.w));
            buffer.push_str(&format!("work_area_height={}\n", monitor.work_area.h));
            buffer.push_str(&format!("work_area_left={}\n", monitor.work_area.x));
            buffer.push_str(&format!("work_area_top={}\n", monitor.work_area.y));
            buffer.push_str(&format!("min_cell_width={}\n", min_cell_width));
            buffer.push_str(&format!("min_cell_height={}\n", min_cell_height));
            buffer.push_str(&format!("min_cols={}\n", bounds.min_cols));
            buffer.push_str(&format!("max_cols={}\n", bounds.max_cols));
            buffer.push_str(&format!("min_rows={}\n", bounds.min_rows));
            buffer.push_str(&format!("max_rows={}\n\n", bounds.max_rows));
        }

        fs::write(&path, buffer).map_err(|source| GridConfigError::CapabilityWriteError {
            path: path.clone(),
            source,
        })
    }

    fn config_from_settings(
        monitor: &Monitor,
        settings: StoredGridSettings
    ) -> Result<(MonitorGridConfig, bool), GridConfigError> {
        let mut config = MonitorGridConfig {
            monitor_index: monitor.index,
            cols: settings.cols,
            rows: settings.rows,
            min_cell_width: settings.min_cell_width,
            min_cell_height: settings.min_cell_height,
        };

        let mut changed = false;

        let sanitized_width = MonitorGridConfig::sanitize_cell_dimension(config.min_cell_width);
        if sanitized_width != config.min_cell_width {
            config.min_cell_width = sanitized_width;
            changed = true;
        }

        let sanitized_height = MonitorGridConfig::sanitize_cell_dimension(config.min_cell_height);
        if sanitized_height != config.min_cell_height {
            config.min_cell_height = sanitized_height;
            changed = true;
        }

        let bounds = GridBounds::for_monitor(
            monitor,
            config.min_cell_width,
            config.min_cell_height
        )?;

        let original_cols = config.cols;
        let original_rows = config.rows;

        config.cols = config.cols.max(MonitorGridConfig::MIN_COLS);
        config.rows = config.rows.max(MonitorGridConfig::MIN_ROWS);

        config.cols = bounds.clamp_cols(config.cols);
        config.rows = bounds.clamp_rows(config.rows);

        if
            let Some(message) = dimension_adjustment_message(
                "Column count",
                original_cols,
                config.cols,
                MonitorGridConfig::MIN_COLS,
                MonitorGridConfig::MAX_COLS,
                bounds.max_cols
            )
        {
            changed = true;
            println!("Grid config for monitor {}: {}", monitor_identifier(monitor), message);
        }

        if
            let Some(message) = dimension_adjustment_message(
                "Row count",
                original_rows,
                config.rows,
                MonitorGridConfig::MIN_ROWS,
                MonitorGridConfig::MAX_ROWS,
                bounds.max_rows
            )
        {
            changed = true;
            println!("Grid config for monitor {}: {}", monitor_identifier(monitor), message);
        }

        Ok((config, changed))
    }

    fn default_config_for_monitor(monitor: &Monitor) -> Result<MonitorGridConfig, GridConfigError> {
        let mut config = MonitorGridConfig::default_for_monitor(monitor);
        config.apply_bounds_from_monitor(monitor)?;
        Ok(config)
    }

    fn monitor_ids(monitors: &[Monitor]) -> Vec<String> {
        monitors.iter().map(monitor_identifier).collect()
    }
}

fn capacity_for(length: i32, min_size: u32) -> u32 {
    if length <= 0 {
        return 0;
    }

    let min_size = min_size.max(1);
    (length as u32) / min_size
}

fn dimension_adjustment_message(
    label: &str,
    original: u32,
    adjusted: u32,
    min_allowed: u32,
    global_max: u32,
    monitor_max: u32
) -> Option<String> {
    if original == adjusted {
        return None;
    }

    let reason = if original < min_allowed {
        format!("was below minimum ({})", min_allowed)
    } else if original > global_max {
        format!("exceeded the global maximum ({})", global_max)
    } else if original > monitor_max {
        format!("exceeded this monitor's capacity ({})", monitor_max)
    } else if adjusted < original {
        "was reduced to satisfy constraints".to_string()
    } else {
        "was increased to satisfy constraints".to_string()
    };

    Some(format!("{} {} {} -> {}", label, original, reason, adjusted))
}

fn monitor_identifier(monitor: &Monitor) -> String {
    // Future multi-monitor support should keep these monitor_id values
    // ("primary" and "monitor-{index}") so the JSON parser stays stable.
    if monitor.is_primary {
        "primary".to_string()
    } else {
        format!("monitor-{}", monitor.index)
    }
}

//! Grid configuration dialog implemented with raw Win32 controls.
//!
//! The dialog is modal with respect to the application controller: when the
//! user opens it (currently via the off-grid `P` key), we pause the standard
//! message pump and run a local loop until the user applies or cancels the
//! configuration changes.

#![allow(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use std::sync::Once;
use std::thread;
use std::time::Duration;

use crate::config::grid::{GridBounds, GridConfigError, MonitorGridConfig, ScreenOrientation};
use crate::platform::monitors::Monitor;
use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM, WIN32_ERROR};
use windows::Win32::Graphics::Gdi::{GetStockObject, HFONT, DEFAULT_GUI_FONT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const DIALOG_WIDTH: i32 = 520;
const DIALOG_HEIGHT: i32 = 420;
const TAB_LEFT: i32 = 16;
const TAB_TOP: i32 = 16;
const TAB_WIDTH: i32 = DIALOG_WIDTH - (TAB_LEFT * 2);
const TAB_HEIGHT: i32 = 260;

const PANEL_LEFT: i32 = TAB_LEFT + 8;
const PANEL_TOP: i32 = TAB_TOP + 40;
const PANEL_WIDTH: i32 = TAB_WIDTH - 16;
const PANEL_HEIGHT: i32 = TAB_HEIGHT - 52;

const ID_TAB_CONTROL: i32 = 1001;
const ID_BTN_APPLY: i32 = 1002;
const ID_BTN_CANCEL: i32 = 1003;
const ID_BTN_RESET_BASE: i32 = 2000;
const ID_CHK_ADVANCED_BASE: i32 = 3000;
// Matches the Win32 ERROR_CLASS_ALREADY_EXISTS (1410) code.
const CLASS_ALREADY_EXISTS_ERR: WIN32_ERROR = WIN32_ERROR(1410);

/// Public entry point for opening the dialog
pub struct GridConfigurationDialog;

impl GridConfigurationDialog {
    pub fn open(
        monitors: &[Monitor],
        configs: &[MonitorGridConfig],
    ) -> Result<Option<Vec<MonitorGridConfig>>, ConfigDialogError> {
        ensure_common_controls();

        if monitors.is_empty() {
            return Err(ConfigDialogError::NoMonitors);
        }

        if configs.len() != monitors.len() {
            return Err(ConfigDialogError::DataMismatch);
        }

        let state = DialogState::new(monitors, configs)?;
        let state_ptr = Box::into_raw(Box::new(state));

        unsafe {
            if let Err(err) = create_dialog_window(state_ptr) {
                let _ = Box::from_raw(state_ptr);
                return Err(err);
            }
            ShowWindow((*state_ptr).hwnd, SW_SHOW);
            let _ = SetForegroundWindow((*state_ptr).hwnd);
            let _ = BringWindowToTop((*state_ptr).hwnd);
        }

        run_modal_loop(state_ptr);

        let boxed_state = unsafe { Box::from_raw(state_ptr) };
        match boxed_state.close_reason {
            DialogCloseReason::Applied => Ok(Some(boxed_state.configs)),
            DialogCloseReason::Cancelled | DialogCloseReason::Pending => Ok(None),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigDialogError {
    #[error("No monitors available for configuration")]
    NoMonitors,
    #[error("Monitor list does not match stored configuration")]
    DataMismatch,
    #[error("Failed to register configuration dialog window class")]
    ClassRegistrationFailed,
    #[error("Failed to create configuration dialog window")]
    WindowCreationFailed,
    #[error("Windows API error: {0}")]
    Win32Error(String),
    #[error(transparent)]
    Grid(#[from] GridConfigError),
}

struct DialogState {
    hwnd: HWND,
    tab_hwnd: HWND,
    monitors: Vec<Monitor>,
    configs: Vec<MonitorGridConfig>,
    panels: Vec<TabControls>,
    spinner_map: HashMap<isize, SpinnerDescriptor>,
    active_tab: usize,
    advanced_visible: Vec<bool>,
    close_reason: DialogCloseReason,
    font: HFONT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DialogCloseReason {
    Pending,
    Applied,
    Cancelled,
}

#[derive(Debug)]
struct TabControls {
    panel: HWND,
    orientation_value: HWND,
    summary_label: HWND,
    cols_edit: HWND,
    cols_spinner: HWND,
    rows_edit: HWND,
    rows_spinner: HWND,
    min_width_edit: HWND,
    min_width_spinner: HWND,
    min_height_edit: HWND,
    min_height_spinner: HWND,
    advanced_group: HWND,
    advanced_toggle: HWND,
    reset_button: HWND,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpinnerField {
    Columns,
    Rows,
    MinWidth,
    MinHeight,
}

#[derive(Debug, Clone, Copy)]
struct SpinnerDescriptor {
    tab_index: usize,
    field: SpinnerField,
}

impl DialogState {
    fn new(
        monitors: &[Monitor],
        configs: &[MonitorGridConfig],
    ) -> Result<Self, ConfigDialogError> {
        if monitors.len() != configs.len() {
            return Err(ConfigDialogError::DataMismatch);
        }
        let configs = configs.to_vec();
        let font = unsafe { HFONT(GetStockObject(DEFAULT_GUI_FONT).0) };

        Ok(Self {
            hwnd: HWND(0),
            tab_hwnd: HWND(0),
            monitors: monitors.to_vec(),
            configs,
            panels: Vec::new(),
            spinner_map: HashMap::new(),
            active_tab: 0,
            advanced_visible: vec![false; monitors.len()],
            close_reason: DialogCloseReason::Pending,
            font,
        })
    }
}

fn ensure_common_controls() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_TAB_CLASSES | ICC_UPDOWN_CLASS | ICC_STANDARD_CLASSES,
        };
        unsafe {
            InitCommonControlsEx(&icc);
        }
    });
}

fn create_dialog_window(state_ptr: *mut DialogState) -> Result<(), ConfigDialogError> {
    unsafe {
        let module = GetModuleHandleW(PCWSTR::null()).map_err(|e| {
            ConfigDialogError::Win32Error(format!("{:?}", e))
        })?;
        let instance: HINSTANCE = module.into();

        register_dialog_class(instance)?;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_CONTROLPARENT.0),
            w!("TactileGridConfigDialog"),
            w!("Grid Configuration"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            DIALOG_WIDTH,
            DIALOG_HEIGHT,
            None,
            None,
            instance,
            Some(state_ptr as *const _ as *mut _),
        );

        if hwnd.0 == 0 {
            return Err(ConfigDialogError::WindowCreationFailed);
        }

        (*state_ptr).hwnd = hwnd;
        Ok(())
    }
}

fn register_dialog_class(instance: HINSTANCE) -> Result<(), ConfigDialogError> {
    unsafe {
        let class_name = w!("TactileGridConfigDialog");
        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(dialog_wnd_proc),
            hInstance: instance,
            lpszClassName: class_name,
            ..Default::default()
        };

        if RegisterClassW(&wnd_class) == 0 {
            match GetLastError() {
                Err(err) if err.code() == CLASS_ALREADY_EXISTS_ERR.to_hresult() => {}
                _ => return Err(ConfigDialogError::ClassRegistrationFailed),
            }
        }
    }

    Ok(())
}

unsafe extern "system" fn dialog_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let createstruct = &*(lparam.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, createstruct.lpCreateParams as isize);
            if let Some(state) = dialog_state_mut(hwnd) {
                state.hwnd = hwnd;
                if let Err(err) = state.build_controls() {
                    state.show_error_dialog(&format!("{}", err));
                    let _ = DestroyWindow(hwnd);
                }
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            if let Some(state) = dialog_state_mut(hwnd) {
                state.handle_command(wparam, lparam);
            }
            LRESULT(0)
        }
        WM_NOTIFY => {
            if let Some(state) = dialog_state_mut(hwnd) {
                if state.handle_notify(lparam) {
                    return LRESULT(0);
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_CLOSE => {
            if let Some(state) = dialog_state_mut(hwnd) {
                state.cancel_and_close();
            } else {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            if let Some(state) = dialog_state_mut(hwnd) {
                if state.close_reason == DialogCloseReason::Pending {
                    state.close_reason = DialogCloseReason::Cancelled;
                }
                state.hwnd = HWND(0);
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn run_modal_loop(state_ptr: *mut DialogState) {
    unsafe {
        let mut msg = MSG::default();
        while (*state_ptr).hwnd.0 != 0 && IsWindow((*state_ptr).hwnd).as_bool() {
            if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    let _ = PostMessageW(HWND(0), WM_QUIT, msg.wParam, msg.lParam);
                    break;
                }

                if !IsDialogMessageW((*state_ptr).hwnd, &mut msg).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

unsafe fn dialog_state_mut(hwnd: HWND) -> Option<&'static mut DialogState> {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut DialogState;
    if ptr.is_null() {
        None
    } else {
        Some(&mut *ptr)
    }
}

impl DialogState {
    fn build_controls(&mut self) -> Result<(), ConfigDialogError> {
        unsafe {
            self.tab_hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                WC_TABCONTROL,
                PCWSTR::null(),
                WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_VISIBLE,
                TAB_LEFT,
                TAB_TOP,
                TAB_WIDTH,
                TAB_HEIGHT,
                self.hwnd,
                HMENU(ID_TAB_CONTROL as isize),
                None,
                None,
            );
            apply_font(self.tab_hwnd, self.font);
        }

        self.panels.reserve(self.monitors.len());

        let monitor_count = self.monitors.len();
        for index in 0..monitor_count {
            self.add_tab(index);
            let controls = self.create_tab_panel(index)?;
            self.panels.push(controls);
            self.refresh_tab(index)?;
            self.toggle_advanced_group(index, false);
        }

        self.show_tab(0);
        self.create_dialog_buttons();

        Ok(())
    }

    fn add_tab(&self, index: usize) {
        unsafe {
            let label = format!("Monitor {}", index + 1);
            let text = to_wstring(&label);
            let mut item = TCITEMW {
                mask: TCIF_TEXT,
                pszText: PWSTR(text.as_ptr() as *mut _),
                ..Default::default()
            };
            SendMessageW(
                self.tab_hwnd,
                TCM_INSERTITEMW,
                WPARAM(index),
                LPARAM(&mut item as *mut _ as isize),
            );
        }
    }

    fn create_tab_panel(&mut self, index: usize) -> Result<TabControls, ConfigDialogError> {
        let visible_style_bits = if index == 0 { WS_VISIBLE.0 } else { 0 };
        let panel_style = WINDOW_STYLE(WS_CHILD.0 | WS_CLIPCHILDREN.0 | visible_style_bits);

        let panel = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(WS_EX_CONTROLPARENT.0),
                w!("STATIC"),
                PCWSTR::null(),
                panel_style,
                PANEL_LEFT,
                PANEL_TOP,
                PANEL_WIDTH,
                PANEL_HEIGHT,
                self.hwnd,
                None,
                None,
                None,
            )
        };
        apply_font(panel, self.font);

        let _orientation_label = create_static(panel, self.font, "Orientation", 12, 10, 100, 20);
        let orientation_value = create_static(
            panel,
            self.font,
            &format_orientation_text(&self.monitors[index]),
            120,
            10,
            PANEL_WIDTH - 160,
            20,
        );

        let summary_label = create_static(panel, self.font, "", 12, 36, PANEL_WIDTH - 24, 24);

        let reset_button = unsafe {
            let style_bits =
                WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | (BS_PUSHBUTTON as u32);
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Reset to Default"),
                WINDOW_STYLE(style_bits),
                PANEL_WIDTH - 180,
                8,
                160,
                26,
                panel,
                HMENU((ID_BTN_RESET_BASE + index as i32) as isize),
                None,
                None,
            )
        };
        apply_font(reset_button, self.font);

        let _cols_label = create_static(panel, self.font, "Columns", 12, 76, 100, 22);
        let cols_edit = create_readonly_edit(panel, self.font, 120, 72, 64, 26);
        let cols_spinner = create_spinner(panel, 190, 72, 26);
        self.spinner_map.insert(
            cols_spinner.0,
            SpinnerDescriptor {
                tab_index: index,
                field: SpinnerField::Columns,
            },
        );

        let _rows_label = create_static(panel, self.font, "Rows", 12, 110, 100, 22);
        let rows_edit = create_readonly_edit(panel, self.font, 120, 106, 64, 26);
        let rows_spinner = create_spinner(panel, 190, 106, 26);
        self.spinner_map.insert(
            rows_spinner.0,
            SpinnerDescriptor {
                tab_index: index,
                field: SpinnerField::Rows,
            },
        );

        let advanced_toggle = unsafe {
            let style_bits =
                WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | (BS_AUTOCHECKBOX as u32);
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Advanced (minimum cell size)"),
                WINDOW_STYLE(style_bits),
                12,
                148,
                PANEL_WIDTH - 24,
                24,
                panel,
                HMENU((ID_CHK_ADVANCED_BASE + index as i32) as isize),
                None,
                None,
            )
        };
        apply_font(advanced_toggle, self.font);

        let advanced_group = unsafe {
            let style_bits = WS_CHILD.0 | (BS_GROUPBOX as u32);
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Minimum cell size (px)"),
                WINDOW_STYLE(style_bits),
                12,
                180,
                PANEL_WIDTH - 24,
                150,
                panel,
                None,
                None,
                None,
            )
        };
        apply_font(advanced_group, self.font);
        unsafe {
            ShowWindow(advanced_group, SW_HIDE);
        }

        let _min_width_label = create_static(advanced_group, self.font, "Minimum width", 12, 32, 140, 22);
        let min_width_edit = create_readonly_edit(advanced_group, self.font, 160, 28, 64, 26);
        let min_width_spinner = create_spinner(advanced_group, 230, 28, 26);
        self.spinner_map.insert(
            min_width_spinner.0,
            SpinnerDescriptor {
                tab_index: index,
                field: SpinnerField::MinWidth,
            },
        );

        let _min_height_label = create_static(advanced_group, self.font, "Minimum height", 12, 68, 140, 22);
        let min_height_edit = create_readonly_edit(advanced_group, self.font, 160, 64, 64, 26);
        let min_height_spinner = create_spinner(advanced_group, 230, 64, 26);
        self.spinner_map.insert(
            min_height_spinner.0,
            SpinnerDescriptor {
                tab_index: index,
                field: SpinnerField::MinHeight,
            },
        );

        Ok(TabControls {
            panel,
            orientation_value,
            summary_label,
            cols_edit,
            cols_spinner,
            rows_edit,
            rows_spinner,
            min_width_edit,
            min_width_spinner,
            min_height_edit,
            min_height_spinner,
            advanced_group,
            advanced_toggle,
            reset_button,
        })
    }

    fn create_dialog_buttons(&self) {
        unsafe {
            let button_style = WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0);
            let apply = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Apply"),
                button_style,
                DIALOG_WIDTH - 190,
                DIALOG_HEIGHT - 70,
                80,
                26,
                self.hwnd,
                HMENU(ID_BTN_APPLY as isize),
                None,
                None,
            );
            apply_font(apply, self.font);

            let cancel = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Cancel"),
                button_style,
                DIALOG_WIDTH - 100,
                DIALOG_HEIGHT - 70,
                80,
                26,
                self.hwnd,
                HMENU(ID_BTN_CANCEL as isize),
                None,
                None,
            );
            apply_font(cancel, self.font);
        }
    }

    fn show_tab(&mut self, index: usize) {
        for (tab_index, panel) in self.panels.iter().enumerate() {
            unsafe {
                ShowWindow(panel.panel, if tab_index == index { SW_SHOW } else { SW_HIDE });
            }
        }
        self.active_tab = index;
    }

    fn refresh_tab(&mut self, index: usize) -> Result<(), ConfigDialogError> {
        if index >= self.panels.len() {
            return Err(ConfigDialogError::DataMismatch);
        }

        let config = self
            .configs
            .get(index)
            .ok_or(ConfigDialogError::DataMismatch)?;
        let panel = &self.panels[index];

        set_numeric_field(panel.cols_edit, panel.cols_spinner, config.cols);
        set_numeric_field(panel.rows_edit, panel.rows_spinner, config.rows);
        set_numeric_field(panel.min_width_edit, panel.min_width_spinner, config.min_cell_width);
        set_numeric_field(panel.min_height_edit, panel.min_height_spinner, config.min_cell_height);

        self.update_summary_label(index);
        self.update_spinner_ranges(index)?;

        Ok(())
    }

    fn update_summary_label(&self, index: usize) {
        if let (Some(panel), Some(config)) = (self.panels.get(index), self.configs.get(index)) {
            set_control_text(
                panel.summary_label,
                &format_summary_text(config.cols, config.rows),
            );
        }
    }

    fn update_spinner_ranges(&mut self, index: usize) -> Result<(), ConfigDialogError> {
        let monitor = self
            .monitors
            .get(index)
            .ok_or(ConfigDialogError::DataMismatch)?;
        let config = self
            .configs
            .get_mut(index)
            .ok_or(ConfigDialogError::DataMismatch)?;

        let bounds = GridBounds::for_monitor(
            monitor,
            config.min_cell_width,
            config.min_cell_height,
        )?;

        config.cols = bounds.clamp_cols(config.cols);
        config.rows = bounds.clamp_rows(config.rows);

        let panel = self
            .panels
            .get(index)
            .ok_or(ConfigDialogError::DataMismatch)?;

        unsafe {
            SendMessageW(
                panel.cols_spinner,
                UDM_SETRANGE32,
                WPARAM(bounds.min_cols as usize),
                LPARAM(bounds.max_cols as isize),
            );
            SendMessageW(
                panel.rows_spinner,
                UDM_SETRANGE32,
                WPARAM(bounds.min_rows as usize),
                LPARAM(bounds.max_rows as isize),
            );
            SendMessageW(
                panel.min_width_spinner,
                UDM_SETRANGE32,
                WPARAM(MonitorGridConfig::MIN_CELL_LIMIT as usize),
                LPARAM(MonitorGridConfig::MAX_CELL_LIMIT as isize),
            );
            SendMessageW(
                panel.min_height_spinner,
                UDM_SETRANGE32,
                WPARAM(MonitorGridConfig::MIN_CELL_LIMIT as usize),
                LPARAM(MonitorGridConfig::MAX_CELL_LIMIT as isize),
            );
        }

        Ok(())
    }

    fn toggle_advanced_group(&mut self, index: usize, visible: bool) {
        if let Some(panel) = self.panels.get(index) {
            unsafe {
                ShowWindow(panel.advanced_group, if visible { SW_SHOW } else { SW_HIDE });
                SendMessageW(
                    panel.advanced_toggle,
                    BM_SETCHECK,
                    WPARAM(if visible { BST_CHECKED.0 as usize } else { BST_UNCHECKED.0 as usize }),
                    LPARAM(0),
                );
            }
        }
        if let Some(flag) = self.advanced_visible.get_mut(index) {
            *flag = visible;
        }
    }

    fn handle_command(&mut self, wparam: WPARAM, _lparam: LPARAM) {
        let command_id = (wparam.0 & 0xFFFF) as i32;
        let notify_code = ((wparam.0 >> 16) & 0xFFFF) as u16;

        if command_id == ID_BTN_APPLY {
            self.handle_apply();
            return;
        }

        if command_id == ID_BTN_CANCEL {
            self.cancel_and_close();
            return;
        }

        if command_id >= ID_BTN_RESET_BASE && command_id < ID_BTN_RESET_BASE + (self.panels.len() as i32)
        {
            let tab_index = (command_id - ID_BTN_RESET_BASE) as usize;
            self.reset_tab(tab_index);
            return;
        }

        if command_id >= ID_CHK_ADVANCED_BASE
            && command_id < ID_CHK_ADVANCED_BASE + (self.panels.len() as i32)
            && notify_code == BN_CLICKED as u16
        {
            let tab_index = (command_id - ID_CHK_ADVANCED_BASE) as usize;
            let new_state = !self.advanced_visible.get(tab_index).copied().unwrap_or(false);
            self.toggle_advanced_group(tab_index, new_state);
            return;
        }
    }

    fn handle_notify(&mut self, lparam: LPARAM) -> bool {
        unsafe {
            let header = &*(lparam.0 as *const NMHDR);

            if header.hwndFrom == self.tab_hwnd && header.code == TCN_SELCHANGE as u32 {
                let new_index = SendMessageW(self.tab_hwnd, TCM_GETCURSEL, WPARAM(0), LPARAM(0)).0 as usize;
                self.show_tab(new_index);
                return false;
            }

            if header.code == UDN_DELTAPOS as u32 {
                let data = &mut *(lparam.0 as *mut NMUPDOWN);
                return self.handle_spinner_delta(header.hwndFrom, data);
            }
        }

        false
    }

    fn handle_spinner_delta(&mut self, hwnd_from: HWND, delta: &mut NMUPDOWN) -> bool {
        let descriptor = match self.spinner_map.get(&hwnd_from.0) {
            Some(desc) => *desc,
            None => return false,
        };

        match descriptor.field {
            SpinnerField::Columns => self.adjust_grid_dimension(descriptor.tab_index, delta, true),
            SpinnerField::Rows => self.adjust_grid_dimension(descriptor.tab_index, delta, false),
            SpinnerField::MinWidth => self.adjust_minimum_size(descriptor.tab_index, delta, true),
            SpinnerField::MinHeight => self.adjust_minimum_size(descriptor.tab_index, delta, false),
        }
    }

    fn adjust_grid_dimension(&mut self, tab_index: usize, delta: &mut NMUPDOWN, is_columns: bool) -> bool {
        if tab_index >= self.configs.len() {
            return false;
        }

        let bounds = match self.compute_bounds(tab_index) {
            Ok(b) => b,
            Err(err) => {
                self.show_error_dialog(&format!("{}", err));
                return true;
            }
        };

        let (min, max, current_value) = if is_columns {
            (
                bounds.min_cols as i32,
                bounds.max_cols as i32,
                self.configs[tab_index].cols as i32,
            )
        } else {
            (
                bounds.min_rows as i32,
                bounds.max_rows as i32,
                self.configs[tab_index].rows as i32,
            )
        };

        let mut new_value = current_value + delta.iDelta;
        if new_value < min {
            new_value = min;
        }
        if new_value > max {
            new_value = max;
        }

        if is_columns {
            self.configs[tab_index].cols = new_value as u32;
        } else {
            self.configs[tab_index].rows = new_value as u32;
        }

        let _ = self.refresh_tab(tab_index);
        delta.iDelta = 0;
        true
    }

    fn adjust_minimum_size(&mut self, tab_index: usize, delta: &mut NMUPDOWN, is_width: bool) -> bool {
        if tab_index >= self.configs.len() {
            return false;
        }

        let config = &mut self.configs[tab_index];
        let mut new_value = (if is_width {
            config.min_cell_width
        } else {
            config.min_cell_height
        }) as i32;

        new_value += delta.iDelta;

        let min_limit = MonitorGridConfig::MIN_CELL_LIMIT as i32;
        let max_limit = MonitorGridConfig::MAX_CELL_LIMIT as i32;
        if new_value < min_limit {
            new_value = min_limit;
        }
        if new_value > max_limit {
            new_value = max_limit;
        }

        if is_width {
            config.min_cell_width = new_value as u32;
        } else {
            config.min_cell_height = new_value as u32;
        }

        if let Err(err) = self.refresh_tab(tab_index) {
            self.show_error_dialog(&format!("{}", err));
        }

        delta.iDelta = 0;
        true
    }

    fn reset_tab(&mut self, tab_index: usize) {
        if let (Some(monitor), Some(config)) = (
            self.monitors.get(tab_index),
            self.configs.get_mut(tab_index),
        ) {
            config.reset_to_defaults(monitor);
            if let Err(err) = config.apply_bounds_from_monitor(monitor) {
                self.show_error_dialog(&format!("{}", err));
            }
            let _ = self.refresh_tab(tab_index);
            self.toggle_advanced_group(tab_index, false);
        }
    }

    fn handle_apply(&mut self) {
        for index in 0..self.monitors.len() {
            if let Err(err) = self.compute_bounds(index) {
                self.show_error_dialog(&format!("{}", err));
                unsafe {
                    SendMessageW(self.tab_hwnd, TCM_SETCURSEL, WPARAM(index), LPARAM(0));
                }
                self.show_tab(index);
                return;
            }
        }

        self.close_reason = DialogCloseReason::Applied;
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }

    fn cancel_and_close(&mut self) {
        self.close_reason = DialogCloseReason::Cancelled;
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }

    fn show_error_dialog(&self, message: &str) {
        let wide = to_wstring(message);
        unsafe {
            MessageBoxW(
                self.hwnd,
                PCWSTR(wide.as_ptr()),
                w!("Grid Configuration"),
                MB_OK | MB_ICONWARNING,
            );
        }
    }

    fn compute_bounds(&self, index: usize) -> Result<GridBounds, ConfigDialogError> {
        let monitor = self
            .monitors
            .get(index)
            .ok_or(ConfigDialogError::DataMismatch)?;
        let config = self
            .configs
            .get(index)
            .ok_or(ConfigDialogError::DataMismatch)?;

        let bounds = GridBounds::for_monitor(
            monitor,
            config.min_cell_width,
            config.min_cell_height,
        )?;

        Ok(bounds)
    }
}

fn apply_font(hwnd: HWND, font: HFONT) {
    unsafe {
        SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    }
}

fn create_static(
    parent: HWND,
    font: HFONT,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> HWND {
    unsafe {
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE,
            x,
            y,
            width,
            height,
            parent,
            None,
            None,
            None,
        );
        apply_font(hwnd, font);
        set_control_text(hwnd, text);
        hwnd
    }
}

fn create_readonly_edit(
    parent: HWND,
    font: HFONT,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> HWND {
    unsafe {
        let style_bits = WS_CHILD.0
            | WS_VISIBLE.0
            | WS_BORDER.0
            | WS_TABSTOP.0
            | (ES_CENTER as u32)
            | (ES_READONLY as u32);
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            PCWSTR::null(),
            WINDOW_STYLE(style_bits),
            x,
            y,
            width,
            height,
            parent,
            None,
            None,
            None,
        );
        apply_font(hwnd, font);
        hwnd
    }
}

fn create_spinner(
    parent: HWND,
    x: i32,
    y: i32,
    height: i32,
) -> HWND {
    unsafe {
        let style_bits = WS_CHILD.0
            | WS_VISIBLE.0
            | (UDS_SETBUDDYINT as u32)
            | (UDS_ARROWKEYS as u32);
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            UPDOWN_CLASS,
            PCWSTR::null(),
            WINDOW_STYLE(style_bits),
            x,
            y,
            22,
            height,
            parent,
            None,
            None,
            None,
        )
    }
}

fn set_control_text(hwnd: HWND, text: &str) {
    let wide = to_wstring(text);
    unsafe {
        let _ = SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()));
    }
}

fn set_numeric_field(edit: HWND, spinner: HWND, value: u32) {
    set_control_text(edit, &value.to_string());
    unsafe {
        SendMessageW(spinner, UDM_SETPOS32, WPARAM(0), LPARAM(value as isize));
    }
}

fn format_orientation_text(monitor: &Monitor) -> String {
    let orientation = ScreenOrientation::from_rect(&monitor.work_area);
    let label = match orientation {
        ScreenOrientation::Landscape => "Landscape",
        ScreenOrientation::Portrait => "Portrait",
    };

    format!(
        "{} · {} × {} px",
        label,
        monitor.work_area.w.max(0),
        monitor.work_area.h.max(0)
    )
}

fn format_summary_text(cols: u32, rows: u32) -> String {
    format!("Current grid: {} columns × {} rows", cols, rows)
}

fn to_wstring(input: &str) -> Vec<u16> {
    input.encode_utf16().chain(std::iter::once(0)).collect()
}

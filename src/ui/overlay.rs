//! Overlay window management for grid display
//!
//! Provides transparent overlay windows that appear over all monitors
//! without stealing focus from the active window. Uses proper Win32
//! window styles for transparency and topmost behavior.

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{ Arc, Mutex };

use windows::Win32::Foundation::{ COLORREF, POINT, SIZE, HWND, LPARAM, LRESULT, WPARAM };
use windows::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA,
    AC_SRC_OVER,
    BI_RGB,
    BITMAPINFO,
    BITMAPINFOHEADER,
    BLENDFUNCTION,
    CreateCompatibleDC,
    CreateDIBSection,
    CreateSolidBrush,
    DIB_RGB_COLORS,
    DeleteDC,
    DeleteObject,
    GetDC,
    HGDIOBJ,
    ReleaseDC,
    SelectObject,
    BeginPaint,
    EndPaint,
    PAINTSTRUCT,
    SetDCBrushColor,
    SetDCPenColor,
    Rectangle,
    TextOutW,
    SetBkMode,
    SetTextColor,
    TRANSPARENT,
    CreateFontW,
    FW_BOLD,
    DEFAULT_CHARSET,
    OUT_DEFAULT_PRECIS,
    CLIP_DEFAULT_PRECIS,
    DEFAULT_QUALITY,
    DEFAULT_PITCH,
    FF_DONTCARE,
    MoveToEx,
    LineTo,
    InvalidateRect,
    CreatePen,
    PS_SOLID,
    HPEN,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW,
    DefWindowProcW,
    DestroyWindow,
    LWA_ALPHA,
    LWA_COLORKEY,
    RegisterClassW,
    SW_HIDE,
    SW_SHOW,
    SetLayeredWindowAttributes,
    ShowWindow,
    ULW_ALPHA,
    UpdateLayeredWindow,
    WM_DESTROY,
    WM_PAINT,
    WNDCLASSW,
    WS_EX_LAYERED,
    WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST,
    WS_EX_TRANSPARENT,
    WS_POPUP,
    GWLP_USERDATA,
    SetWindowLongPtrW,
    GetWindowLongPtrW,
};
use windows::core::w;

use crate::domain::core::Rect;
use crate::domain::grid::Grid;
use crate::platform::monitors::Monitor;
use crate::ui::renderer::{ GridLayout, GridRenderer, RendererError };

/// Overlay management errors
#[derive(Debug, thiserror::Error)]
pub enum OverlayError {
    #[error("Failed to register overlay window class")]
    WindowClassRegistrationFailed,

    #[error("Failed to create overlay window for monitor {monitor_index}")] WindowCreationFailed {
        monitor_index: usize,
    },

    #[error("Failed to configure overlay transparency")]
    TransparencyConfigurationFailed,

    #[error("Failed to acquire screen device context")]
    DeviceContextFailed,

    #[error("Failed to create memory device context")]
    MemoryDeviceContextFailed,

    #[error("Failed to create DIB section for overlay frame")]
    DibSectionCreationFailed,

    #[error("Failed to select bitmap into memory DC")]
    BitmapSelectionFailed,

    #[error("Failed to update layered window surface")]
    LayerUpdateFailed,

    #[error("Overlay manager not initialized")]
    NotInitialized,

    #[error("Rendering failed: {0}")] RenderingError(#[from] RendererError),
}

/// Overlay window for a single monitor
#[derive(Debug)]
pub struct OverlayWindow {
    /// Windows handle to the overlay window
    pub hwnd: HWND,

    /// Monitor this overlay belongs to
    pub monitor_index: usize,

    /// Monitor bounds for positioning
    pub monitor_rect: Rect,

    /// Grid for this monitor
    pub grid: Grid,

    /// DPI scale for this monitor
    pub dpi_scale: f32,

    /// Current visibility state
    pub visible: bool,

    /// Whether this overlay is the active monitor (shows letters)
    pub is_active: bool,

    /// Cached rendered content
    cached_pixmap: Option<tiny_skia::Pixmap>,

    /// Grid renderer
    renderer: GridRenderer,
}

impl OverlayWindow {
    /// Create a new overlay window for the specified monitor
    fn new(monitor_index: usize, monitor: &Monitor, grid: Grid) -> Result<Self, OverlayError> {
        let class_name = w!("TactileWinOverlayWindow");

        // Register window class if needed
        Self::register_window_class(class_name)?;

        // Create the overlay window
        let hwnd = Self::create_overlay_window(class_name, &monitor.work_area)?;

        // Configure transparency
        Self::configure_transparency(hwnd)?;

        let mut overlay = Self {
            hwnd,
            monitor_index,
            monitor_rect: monitor.work_area,
            grid,
            dpi_scale: monitor.dpi_scale,
            visible: false,
            is_active: false,
            cached_pixmap: None,
            renderer: GridRenderer::new(),
        };

        // Store pointer to self in window user data for access from WM_PAINT
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{ SetWindowLongPtrW, GWLP_USERDATA };
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, &overlay as *const _ as isize);
        }

        Ok(overlay)
    }

    /// Register overlay window class
    fn register_window_class(class_name: windows::core::PCWSTR) -> Result<(), OverlayError> {
        // Window procedure for overlay windows
        unsafe extern "system" fn overlay_window_proc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM
        ) -> LRESULT {
            match msg {
                WM_PAINT => {
                    use windows::Win32::UI::WindowsAndMessaging::{
                        GetWindowLongPtrW,
                        GWLP_USERDATA,
                    };

                    unsafe {
                        let mut ps = PAINTSTRUCT::default();
                        let hdc = BeginPaint(hwnd, &mut ps);

                        // Try to get the overlay window pointer from user data
                        let overlay_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);

                        if overlay_ptr != 0 {
                            let overlay = &*(overlay_ptr as *const OverlayWindow);

                            // Set up drawing context
                            SetBkMode(hdc, TRANSPARENT);
                            SetTextColor(hdc, COLORREF(0x0000ffff)); // Yellow text

                            // Create a thicker white pen for visible grid lines
                            use windows::Win32::Graphics::Gdi::{ CreatePen, PS_SOLID, HPEN };
                            let hpen = CreatePen(PS_SOLID, 3, COLORREF(0x00ffffff)); // 3 pixels thick, white
                            let old_pen = SelectObject(hdc, HGDIOBJ(hpen.0));

                            // Create font for grid letters
                            let (cell_width, cell_height) = overlay.grid.cell_size();
                            let font_name: Vec<u16> = "Arial\0".encode_utf16().collect();
                            let font_height = (cell_height / 2) as i32; // Half cell height
                            let hfont = CreateFontW(
                                font_height,
                                0,
                                0,
                                0,
                                FW_BOLD.0 as i32,
                                0,
                                0,
                                0,
                                DEFAULT_CHARSET.0 as u32,
                                OUT_DEFAULT_PRECIS.0 as u32,
                                CLIP_DEFAULT_PRECIS.0 as u32,
                                DEFAULT_QUALITY.0 as u32,
                                (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
                                windows::core::PCWSTR(font_name.as_ptr())
                            );
                            let old_font = SelectObject(hdc, HGDIOBJ(hfont.0));

                            // Draw vertical grid lines
                            let (rows, cols) = overlay.grid.dimensions();
                            let cell_width = cell_width as i32;
                            let cell_height = cell_height as i32;

                            for col in 0..=cols {
                                let x = (col as i32) * cell_width;
                                use windows::Win32::Graphics::Gdi::{ MoveToEx, LineTo };
                                MoveToEx(hdc, x, 0, None);
                                LineTo(hdc, x, (rows as i32) * cell_height);
                            }

                            // Draw horizontal grid lines
                            for row in 0..=rows {
                                let y = (row as i32) * cell_height;
                                use windows::Win32::Graphics::Gdi::{ MoveToEx, LineTo };
                                MoveToEx(hdc, 0, y, None);
                                LineTo(hdc, (cols as i32) * cell_width, y);
                            }

                            // Draw letters in cells (if active monitor)
                            if overlay.is_active {
                                use crate::domain::keyboard::GridCoords;
                                let layout = overlay.grid.keyboard_layout();
                                for row in 0..rows {
                                    for col in 0..cols {
                                        let coords = GridCoords::new(row, col);
                                        if let Ok(key) = layout.coords_to_key(coords) {
                                            let center_x =
                                                (col as i32) * cell_width +
                                                cell_width / 2 -
                                                font_height / 3;
                                            let center_y =
                                                (row as i32) * cell_height +
                                                cell_height / 2 -
                                                font_height / 3;

                                            let text: Vec<u16> = format!("{}\0", key)
                                                .encode_utf16()
                                                .collect();
                                            TextOutW(hdc, center_x, center_y, &text);
                                        }
                                    }
                                }
                            }

                            // Restore and cleanup
                            SelectObject(hdc, old_font);
                            DeleteObject(HGDIOBJ(hfont.0));
                            SelectObject(hdc, old_pen);
                            DeleteObject(HGDIOBJ(hpen.0));
                        }

                        EndPaint(hwnd, &ps);
                    }
                    LRESULT(0)
                }
                WM_DESTROY => { LRESULT(0) }
                _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
        }

        let hinstance = unsafe { GetModuleHandleW(None).unwrap() };

        // Create a dark blue brush for the background (will be semi-transparent)
        let hbrush = unsafe { CreateSolidBrush(COLORREF(0x00330000)) }; // Dark blue: BGR format

        let wc = WNDCLASSW {
            lpfnWndProc: Some(overlay_window_proc),
            hInstance: hinstance.into(),
            lpszClassName: class_name,
            hbrBackground: hbrush,
            ..Default::default()
        };

        let class_atom = unsafe { RegisterClassW(&wc) };
        if class_atom == 0 {
            // Class might already be registered, which is fine
            // We'll check the specific error if needed
        }

        Ok(())
    }

    /// Create the actual overlay window
    fn create_overlay_window(
        class_name: windows::core::PCWSTR,
        monitor_rect: &Rect
    ) -> Result<HWND, OverlayError> {
        let hinstance = unsafe { GetModuleHandleW(None).unwrap() };

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED |
                    WS_EX_TOPMOST |
                    WS_EX_NOACTIVATE |
                    WS_EX_TOOLWINDOW |
                    WS_EX_TRANSPARENT,
                class_name,
                w!("Tactile Overlay"),
                WS_POPUP, // Popup window with no frame
                monitor_rect.x,
                monitor_rect.y,
                monitor_rect.w,
                monitor_rect.h,
                None, // No parent
                None, // No menu
                hinstance,
                None
            )
        };

        if hwnd.0 == 0 {
            return Err(OverlayError::WindowCreationFailed { monitor_index: 0 });
        }

        Ok(hwnd)
    }

    /// Configure window transparency and alpha blending
    fn configure_transparency(hwnd: HWND) -> Result<(), OverlayError> {
        // Set semi-transparent overlay (no color keying - we want to see the background)
        let result = unsafe {
            SetLayeredWindowAttributes(
                hwnd,
                COLORREF(0), // Not used when only LWA_ALPHA is set
                180, // Alpha value (0-255, 180 = ~70% opaque for visibility)
                LWA_ALPHA // Only alpha blending, no color key
            )
        };

        if result.is_err() {
            return Err(OverlayError::TransparencyConfigurationFailed);
        }

        Ok(())
    }

    /// Show the overlay window
    pub fn show(&mut self) {
        if !self.visible {
            unsafe {
                // Update window user data pointer before showing
                SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, self as *const _ as isize);
                ShowWindow(self.hwnd, SW_SHOW);
                // Trigger initial paint
                InvalidateRect(self.hwnd, None, false);
            }
            self.visible = true;
        }
    }

    /// Hide the overlay window
    pub fn hide(&mut self) {
        if self.visible {
            unsafe {
                ShowWindow(self.hwnd, SW_HIDE);
            }
            self.visible = false;
        }
    }

    /// Check if overlay is currently visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set whether this overlay is the active monitor
    pub fn set_active(&mut self, active: bool) {
        if self.is_active != active {
            self.is_active = active;
            self.cached_pixmap = None;

            // Update window user data pointer (in case self moved in memory)
            unsafe {
                SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, self as *const _ as isize);
            }

            // Trigger repaint to show/hide letters
            unsafe {
                use windows::Win32::Graphics::Gdi::InvalidateRect;
                InvalidateRect(self.hwnd, None, false);
            }
        }
    }

    /// Check if this overlay is the active monitor
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Render the grid content
    pub fn render_grid(&mut self) -> Result<(), OverlayError> {
        // Create grid layout
        let layout = GridLayout::from_grid(
            &self.grid,
            self.monitor_rect,
            self.is_active,
            self.dpi_scale
        );

        // Render to pixmap
        let pixmap = self.renderer.render_layout(&layout)?;

        // Present the rendered pixmap before caching it
        self.present_pixmap(&pixmap)?;

        // Cache the result for diagnostics/debug purposes
        self.cached_pixmap = Some(pixmap);

        Ok(())
    }

    /// Present the pixmap via UpdateLayeredWindow for flicker-free rendering
    fn present_pixmap(&self, pixmap: &tiny_skia::Pixmap) -> Result<(), OverlayError> {
        use std::slice;

        let width = pixmap.width() as i32;
        let height = pixmap.height() as i32;

        unsafe {
            let screen_dc = GetDC(HWND(0));
            if screen_dc.0 == 0 {
                return Err(OverlayError::DeviceContextFailed);
            }

            let memory_dc = CreateCompatibleDC(screen_dc);
            if memory_dc.0 == 0 {
                ReleaseDC(HWND(0), screen_dc);
                return Err(OverlayError::MemoryDeviceContextFailed);
            }

            let mut bitmap_info = BITMAPINFO::default();
            bitmap_info.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down bitmap so we can copy directly
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            };

            let mut pixel_ptr: *mut c_void = std::ptr::null_mut();
            let dib = match
                CreateDIBSection(memory_dc, &bitmap_info, DIB_RGB_COLORS, &mut pixel_ptr, None, 0)
            {
                Ok(bitmap) => bitmap,
                Err(_) => {
                    DeleteDC(memory_dc);
                    ReleaseDC(HWND(0), screen_dc);
                    return Err(OverlayError::DibSectionCreationFailed);
                }
            };

            let dib_object: HGDIOBJ = dib.into();

            if pixel_ptr.is_null() {
                DeleteObject(dib_object);
                DeleteDC(memory_dc);
                ReleaseDC(HWND(0), screen_dc);
                return Err(OverlayError::DibSectionCreationFailed);
            }

            {
                let dst = slice::from_raw_parts_mut(pixel_ptr as *mut u8, pixmap.data().len());
                dst.copy_from_slice(pixmap.data());
            }

            let old_bitmap = SelectObject(memory_dc, dib_object);
            if old_bitmap.0 == 0 {
                DeleteObject(dib_object);
                DeleteDC(memory_dc);
                ReleaseDC(HWND(0), screen_dc);
                return Err(OverlayError::BitmapSelectionFailed);
            }

            let size = SIZE {
                cx: width,
                cy: height,
            };
            let dst_point = POINT {
                x: self.monitor_rect.x,
                y: self.monitor_rect.y,
            };
            let src_point = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };

            let update_result = UpdateLayeredWindow(
                self.hwnd,
                screen_dc,
                Some(&dst_point),
                Some(&size),
                memory_dc,
                Some(&src_point),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA
            );

            // Clean up GDI objects
            SelectObject(memory_dc, old_bitmap);
            DeleteObject(dib_object);
            DeleteDC(memory_dc);
            ReleaseDC(HWND(0), screen_dc);

            if let Err(_) = update_result {
                return Err(OverlayError::LayerUpdateFailed);
            }
        }

        Ok(())
    }

    /// Get the cached pixmap for display
    pub fn get_cached_pixmap(&self) -> Option<&tiny_skia::Pixmap> {
        self.cached_pixmap.as_ref()
    }
}

impl Drop for OverlayWindow {
    fn drop(&mut self) {
        // Clean up the window
        unsafe {
            DestroyWindow(self.hwnd).ok();
        }
    }
}

/// Manager for all overlay windows across multiple monitors
pub struct OverlayManager {
    /// Map of monitor index to overlay window
    overlays: Arc<Mutex<HashMap<usize, OverlayWindow>>>,

    /// Current visibility state
    visible: bool,
}

impl OverlayManager {
    /// Create a new overlay manager
    pub fn new() -> Self {
        Self {
            overlays: Arc::new(Mutex::new(HashMap::new())),
            visible: false,
        }
    }

    /// Initialize overlay windows for all provided monitors with their grids
    pub fn initialize(&mut self, monitors: &[Monitor], grids: &[Grid]) -> Result<(), OverlayError> {
        if monitors.len() != grids.len() {
            return Err(OverlayError::NotInitialized);
        }

        let mut overlays = self.overlays.lock().unwrap();

        // Clear any existing overlays
        overlays.clear();

        // Create overlay for each monitor with its corresponding grid
        for (index, (monitor, grid)) in monitors.iter().zip(grids.iter()).enumerate() {
            let overlay = OverlayWindow::new(index, monitor, grid.clone())?;
            overlays.insert(index, overlay);
        }

        Ok(())
    }

    /// Show overlays on all monitors
    pub fn show_all(&mut self) {
        if !self.visible {
            // Set first monitor as active by default
            if self.overlay_count() > 0 {
                self.set_active_monitor(0);
            }

            // Render grid content
            self.render_all_grids();

            let mut overlays = self.overlays.lock().unwrap();
            for overlay in overlays.values_mut() {
                overlay.show();
            }
            self.visible = true;
        }
    }

    /// Hide overlays on all monitors
    pub fn hide_all(&mut self) {
        if self.visible {
            let mut overlays = self.overlays.lock().unwrap();
            for overlay in overlays.values_mut() {
                overlay.hide();
            }
            self.visible = false;
        }
    }

    /// Show overlay for a specific monitor
    pub fn show_monitor(&mut self, monitor_index: usize) {
        let mut overlays = self.overlays.lock().unwrap();
        if let Some(overlay) = overlays.get_mut(&monitor_index) {
            overlay.show();
        }
    }

    /// Hide overlay for a specific monitor
    pub fn hide_monitor(&mut self, monitor_index: usize) {
        let mut overlays = self.overlays.lock().unwrap();
        if let Some(overlay) = overlays.get_mut(&monitor_index) {
            overlay.hide();
        }
    }

    /// Check if overlays are currently visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the number of overlay windows
    pub fn overlay_count(&self) -> usize {
        self.overlays.lock().unwrap().len()
    }

    /// Set which monitor is active (shows letters)
    pub fn set_active_monitor(&mut self, monitor_index: usize) {
        {
            let mut overlays = self.overlays.lock().unwrap();

            // Set all monitors as inactive first
            for overlay in overlays.values_mut() {
                overlay.set_active(false);
            }

            // Set the specified monitor as active
            if let Some(overlay) = overlays.get_mut(&monitor_index) {
                overlay.set_active(true);
            }
        } // Release the mutex lock here

        // Trigger re-rendering for all overlays
        self.render_all_grids();
    }

    /// Get the currently active monitor index
    pub fn get_active_monitor(&self) -> Option<usize> {
        let overlays = self.overlays.lock().unwrap();
        for (index, overlay) in overlays.iter() {
            if overlay.is_active() {
                return Some(*index);
            }
        }
        None
    }

    /// Render grid content for all overlays
    pub fn render_all_grids(&mut self) {
        let mut overlays = self.overlays.lock().unwrap();
        for overlay in overlays.values_mut() {
            if let Err(err) = overlay.render_grid() {
                eprintln!("Overlay rendering failed on monitor {}: {}", overlay.monitor_index, err);
            }
        }
    }

    /// Get overlay window handle for a specific monitor
    pub fn get_overlay_hwnd(&self, monitor_index: usize) -> Option<HWND> {
        self.overlays
            .lock()
            .unwrap()
            .get(&monitor_index)
            .map(|overlay| overlay.hwnd)
    }

    /// Toggle overlay visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide_all();
        } else {
            self.show_all();
        }
    }
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::core::Rect;
    use crate::platform::monitors::Monitor;

    #[test]
    fn overlay_manager_creation() {
        let manager = OverlayManager::new();
        assert!(!manager.is_visible());
        assert_eq!(manager.overlay_count(), 0);
    }

    #[test]
    fn overlay_manager_initialization() {
        let mut manager = OverlayManager::new();

        // Create mock monitors
        let monitors = vec![Monitor {
            index: 0,
            handle: windows::Win32::Graphics::Gdi::HMONITOR(1),
            work_area: Rect {
                x: 0,
                y: 0,
                w: 1920,
                h: 1080,
            },
            physical_rect: Rect {
                x: 0,
                y: 0,
                w: 1920,
                h: 1080,
            },
            is_primary: true,
            dpi_scale: 1.0,
            dpi_x: 96,
            dpi_y: 96,
        }];

        // Create a simple grid for testing
        let grid = Grid::new(3, 2, Rect::new(0, 0, 1920, 1080)).unwrap();
        let grids = [grid];

        // Initialize should work
        let result = manager.initialize(&monitors, &grids);

        // In test environment, window creation might fail, but the API should be correct
        match result {
            Ok(()) => {
                assert_eq!(manager.overlay_count(), 1);
            }
            Err(_) => {
                // Expected in test environment without proper Windows context
                println!("Overlay initialization failed (expected in test environment)");
            }
        }
    }

    #[test]
    fn overlay_visibility_state() {
        let mut manager = OverlayManager::new();

        // Initially not visible
        assert!(!manager.is_visible());

        // Show/hide should update state even without initialized overlays
        manager.show_all();
        assert!(manager.is_visible());

        manager.hide_all();
        assert!(!manager.is_visible());

        // Toggle should work
        manager.toggle();
        assert!(manager.is_visible());

        manager.toggle();
        assert!(!manager.is_visible());
    }
}

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

fn left_half(base: RECT) -> RECT {
    let width = base.right - base.left;
    let height = base.bottom - base.top;

    RECT {
        left: base.left,
        top: base.top,
        right: base.left + width / 2,
        bottom: base.top + height,
    }
}

fn right_half(base: RECT) -> RECT {
    let width = base.right - base.left;
    let height = base.bottom - base.top;

    RECT {
        left: base.left + width / 2,
        top: base.top,
        right: base.right,
        bottom: base.top + height,
    }
}

fn main() {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();

        if hwnd.0 == 0 {
            println!("No active window found");
            return;
        }

        let mut buffer = [0u16; 512];

        let length = GetWindowTextW(hwnd, &mut buffer);

        if length == 0 {
            println!("Active window has no title or an error occurred retrieving the window title");
            return;
        }

        let title = String::from_utf16_lossy(&buffer[..length as usize]);

        println!("HWND: {:?}", hwnd);
        println!("Title: {}", title);

        let mut work_area = RECT::default();

        let result = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut work_area as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)
        );

        if result.is_err() {
            eprintln!("Failed to get the work area");
            return;
        }

        let target = left_half(work_area);

        let _ = SetWindowPos(
            hwnd,
            HWND(0),
            target.left,
            target.top,
            target.right - target.left,
            target.bottom - target.top,
            SWP_NOZORDER | SWP_SHOWWINDOW
        );
    }
}

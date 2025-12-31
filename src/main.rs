use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

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

        let width = work_area.right - work_area.left;
        let height = work_area.bottom - work_area.top;

        let left = work_area.left;
        let top = work_area.top;
        let new_width = width / 2;
        let new_height = height;

        let _ =SetWindowPos(
            hwnd,
            HWND(0),
            left,
            top,
            new_width,
            new_height,
            SWP_NOZORDER | SWP_SHOWWINDOW
        );
    }
}

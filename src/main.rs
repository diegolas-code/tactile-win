use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();

        if hwnd.0 == 0 {
            println!("No hay ventana activa");
            return;
        }

        let mut buffer = [0u16; 512];

        let length = GetWindowTextW(hwnd, &mut buffer);

        if length == 0 {
            println!("Ventana activa sin título");
            return;
        }

        let title = String::from_utf16_lossy(&buffer[..length as usize]);

        println!("HWND: {:?}", hwnd);
        println!("Título: {}", title);

        let mut work_area = RECT::default();

        let result = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut work_area as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)
        );

        if result.is_err() {
            eprintln!("No se pudo obtener el work area");
            return;
        }

        // 3. Calcular mitad izquierda
        let width = work_area.right - work_area.left;
        let height = work_area.bottom - work_area.top;

        let left = work_area.left;
        let top = work_area.top;
        let new_width = width / 2;
        let new_height = height;

        // 4. Mover ventana
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

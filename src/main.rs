use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW,
};

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
    }
}

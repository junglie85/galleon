#![windows_subsystem = "windows"]

use windows_sys::Win32::System::Diagnostics::Debug::OutputDebugStringW;

#[macro_export]
macro_rules! wstr {
    ($($arg:tt)*) => {{
        let utf8 = std::fmt::format(format_args!($($arg)*));
        utf8.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>()
    }};
}

fn main() {
    let greeting = wstr!("{}", core::greet("shipmate"));
    unsafe { OutputDebugStringW(greeting.as_ptr()) };
}

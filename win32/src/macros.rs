#[macro_export]
macro_rules! wstr {
    ($($arg:tt)*) => {{
        let utf8 = std::fmt::format(format_args!($($arg)*));
        utf8.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>()
    }};
}

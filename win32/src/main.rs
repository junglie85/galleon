#![cfg_attr(not(test), windows_subsystem = "windows")]

#[allow(clippy::non_minimal_cfg)]
#[cfg(all(not(target_os = "windows")))]
compile_error!("only windows is supported");

use core::logger::{self, info, LevelFilter, Sink};
use std::sync::{Arc, Mutex};

use windows_sys::Win32::System::Diagnostics::Debug::OutputDebugStringW;

#[macro_export]
macro_rules! wstr {
    ($($arg:tt)*) => {{
        let utf8 = std::fmt::format(format_args!($($arg)*));
        utf8.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>()
    }};
}

fn main() {
    let log_sink = DebugConsoleSink::new(LevelFilter::Debug);
    if let Err(err) = logger::startup(LevelFilter::Debug) {
        let msg = wstr!("{err}\n");
        log_sink.output_debug_string(&msg);
        return;
    }

    logger::add_sink(&log_sink);

    let greeting = wstr!("{}\n", core::greet("shipmate"));
    log_sink.output_debug_string(&greeting);

    info!("Test message 1");

    logger::remove_sink(&log_sink);

    info!("Test message 2");

    logger::shutdown();
}

#[derive(Clone)]
struct DebugConsoleSink {
    max_level: Arc<Mutex<LevelFilter>>,
}

impl DebugConsoleSink {
    fn new(max_level: LevelFilter) -> Self {
        Self {
            max_level: Arc::new(Mutex::new(max_level)),
        }
    }

    fn output_debug_string(&self, s: &[u16]) {
        unsafe { OutputDebugStringW(s.as_ptr()) };
    }
}

impl Sink for DebugConsoleSink {
    fn enabled(&self, metadata: &core::logger::Metadata) -> bool {
        metadata.level().to_level_filter() <= *self.max_level.lock().unwrap()
    }

    fn log(&self, record: &core::logger::Record) {
        let msg = wstr!(
            "{} {} {:?} {:?}\n",
            record.level(),
            record.args(),
            record.file(),
            record.line()
        );
        self.output_debug_string(&msg);
    }

    fn flush(&self) {
        // Nothing to do.
    }
}

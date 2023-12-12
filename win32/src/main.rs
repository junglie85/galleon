#![cfg_attr(not(test), windows_subsystem = "windows")]

#[allow(clippy::non_minimal_cfg)]
#[cfg(all(not(target_os = "windows")))]
compile_error!("only windows is supported");

use common::logger::{self, Sink};
use std::sync::{Arc, Mutex};
use tracing::{error, info, info_span, level_filters::LevelFilter, Level};

use windows_sys::Win32::System::Diagnostics::Debug::OutputDebugStringW;

#[macro_export]
macro_rules! wstr {
    ($($arg:tt)*) => {{
        let utf8 = std::fmt::format(format_args!($($arg)*));
        utf8.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>()
    }};
}

fn main() {
    let log_sink = DebugConsoleSink::new(LevelFilter::TRACE);
    if let Err(err) = logger::startup(LevelFilter::TRACE) {
        let msg = wstr!("{err}\n");
        log_sink.output_debug_string(&msg);
        return;
    }

    logger::add_sink(&log_sink);

    let greeting = wstr!("{}\n", common::greet("shipmate"));
    log_sink.output_debug_string(&greeting);

    let outer_span = info_span!("outer", level = 0);
    let _outer_entered = outer_span.enter();

    let inner_span = info_span!("inner", level = 1);
    let _inner_entered = inner_span.enter();

    info!(milk = 3, "Test message 1");

    logger::set_max_level(LevelFilter::ERROR);
    log_sink.set_max_level(LevelFilter::INFO);

    info!(cheese = 7, "Test message 2");

    // logger::remove_sink(&log_sink);

    error!("Test message 3");

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

    fn set_max_level(&self, level: LevelFilter) {
        *self.max_level.lock().unwrap() = level;
    }

    fn output_debug_string(&self, s: &[u16]) {
        unsafe { OutputDebugStringW(s.as_ptr()) };
    }
}

impl Sink for DebugConsoleSink {
    fn enabled(&self, level: &Level) -> bool {
        matches!(self.max_level.lock().unwrap().into_level(), Some(ref max_level) if level <= max_level)
    }

    fn log(
        &self,
        level: &Level,
        msg: &str,
        args: Option<&str>,
        file: Option<&str>,
        line: Option<u32>,
    ) {
        let temp = match (args, file, line) {
            (Some(args), Some(file), Some(line)) => {
                wstr!("[{}][{}:{}] {} {}\n", level, file, line, msg, args)
            }
            (None, Some(file), Some(line)) => {
                wstr!("[{}][{}:{}] {}\n", level, file, line, msg)
            }
            (Some(args), None, None) => wstr!("[{}][unknown:unknown] {} {}\n", level, msg, args),
            _ => wstr!("[{}][unknown:unknown] {}\n", level, msg),
        };

        self.output_debug_string(&temp);
    }

    fn flush(&self) {
        // Nothing to do here.
    }
}

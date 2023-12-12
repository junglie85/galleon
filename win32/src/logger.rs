use common::log::Sink;
use std::sync::{Arc, Mutex};
use tracing::{level_filters::LevelFilter, Level};

use windows_sys::Win32::System::Diagnostics::Debug::OutputDebugStringW;

use crate::wstr;

#[derive(Clone)]
pub struct DebugConsoleSink {
    max_level: Arc<Mutex<LevelFilter>>,
}

impl DebugConsoleSink {
    pub fn new(max_level: LevelFilter) -> Self {
        Self {
            max_level: Arc::new(Mutex::new(max_level)),
        }
    }

    pub fn set_max_level(&self, level: LevelFilter) {
        *self.max_level.lock().unwrap() = level;
    }

    pub fn output_debug_string(&self, s: &[u16]) {
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

#![cfg_attr(not(test), windows_subsystem = "windows")]

#[allow(clippy::non_minimal_cfg)]
#[cfg(all(not(target_os = "windows")))]
compile_error!("only windows is supported");

use common::log::{self};
use tracing::{error, info, info_span, level_filters::LevelFilter};

use crate::logger::DebugConsoleSink;

mod logger;
mod macros;

fn main() {
    let log_sink = DebugConsoleSink::new(LevelFilter::TRACE);
    if let Err(err) = log::startup(LevelFilter::TRACE) {
        let msg = wstr!("{err}\n");
        log_sink.output_debug_string(&msg);
        return;
    }

    log::add_sink(&log_sink);

    let greeting = wstr!("{}\n", common::greet("shipmate"));
    log_sink.output_debug_string(&greeting);

    let outer_span = info_span!("outer", level = 0);
    let _outer_entered = outer_span.enter();

    let inner_span = info_span!("inner", level = 1);
    let _inner_entered = inner_span.enter();

    info!(milk = 3, "Test message 1");

    log::set_max_level(LevelFilter::ERROR);
    log_sink.set_max_level(LevelFilter::INFO);

    info!(cheese = 7, "Test message 2");

    // logger::remove_sink(&log_sink);

    error!("Test message 3");

    log::shutdown();
}

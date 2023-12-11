pub use log::{debug, error, info, trace, warn, Level, LevelFilter, Metadata, Record};
use log::{Log, SetLoggerError};
use std::{
    any::TypeId,
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex, OnceLock},
};

use crate::error::Error;

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[derive(Default, Clone)]
struct Logger {
    sinks: Arc<Mutex<HashMap<TypeId, Box<dyn Sink>>>>,
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl Logger {
    fn add_sink<S: Sink + Clone + 'static>(&self, sink: &S) {
        self.sinks
            .lock()
            .unwrap()
            .insert(TypeId::of::<S>(), Box::new(sink.clone()));
    }

    fn remove_sink<S: Sink + Clone + 'static>(&self, _sink: &S) {
        self.sinks.lock().unwrap().remove(&TypeId::of::<S>());
    }

    fn clear(&self) {
        let mut sinks = self.sinks.lock().unwrap();
        sinks.clear();
        sinks.shrink_to(0);
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level().to_level_filter() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            for sink in self.sinks.lock().unwrap().values() {
                if sink.enabled(record.metadata()) {
                    sink.log(record);
                }
            }
        }
    }

    fn flush(&self) {
        for sink in self.sinks.lock().unwrap().values() {
            sink.flush();
        }
    }
}

pub trait Sink {
    fn enabled(&self, metadata: &Metadata) -> bool;

    fn log(&self, record: &Record);

    fn flush(&self);
}

pub fn startup(max_level: LevelFilter) -> Result<(), LoggerError> {
    let logger = LOGGER.get_or_init(Logger::default);
    log::set_logger(logger)?;
    set_max_level(max_level);

    Ok(())
}

pub fn shutdown() {
    if let Some(logger) = LOGGER.get() {
        log::set_max_level(LevelFilter::Off);
        logger.flush();
        logger.clear();
    }
}

pub fn add_sink<S: Sink + Clone + 'static>(sink: &S) {
    if let Some(logger) = LOGGER.get() {
        logger.add_sink(sink);
    }
}

pub fn remove_sink<S: Sink + Clone + 'static>(sink: &S) {
    if let Some(logger) = LOGGER.get() {
        logger.remove_sink(sink);
    }
}

pub fn set_max_level(level: LevelFilter) {
    log::set_max_level(level);
}

#[derive(Debug)]
pub enum LoggerError {
    AlreadyInitialized,
}

impl Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggerError::AlreadyInitialized => write!(f, "logger is already initialized"),
        }
    }
}

impl std::error::Error for LoggerError {}

impl From<SetLoggerError> for LoggerError {
    fn from(_: SetLoggerError) -> Self {
        LoggerError::AlreadyInitialized
    }
}

impl From<LoggerError> for Error {
    fn from(err: LoggerError) -> Self {
        Error::new("failed to initialize logger").with_source(err)
    }
}

use std::{
    any::TypeId,
    collections::HashMap,
    fmt::{Display, Write},
    sync::{Arc, Mutex, OnceLock},
};

use tracing::{
    error, field::Visit, level_filters::LevelFilter, subscriber::SetGlobalDefaultError, Level,
    Subscriber,
};
use tracing_subscriber::{
    layer::SubscriberExt,
    reload::{self, Handle},
    Layer, Registry,
};

use crate::error::Error;

// note: this does not currently handle spans. see https://burgers.io/custom-logging-in-rust-using-tracing-part-2

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[derive(Clone)]
struct Logger {
    inner: Arc<Mutex<LoggerInner>>,
}

struct LoggerInner {
    reload_handle: Option<Handle<LevelFilter, Registry>>,
    sinks: HashMap<TypeId, Box<dyn Sink>>,
}

unsafe impl Send for LoggerInner {}
unsafe impl Sync for LoggerInner {}

impl Logger {
    fn new(reload_handle: Handle<LevelFilter, Registry>) -> Self {
        let inner = LoggerInner {
            reload_handle: Some(reload_handle),
            sinks: HashMap::new(),
        };

        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    fn set_max_level(&self, level: LevelFilter) {
        if let Some(reload_handle) = self.inner.lock().unwrap().reload_handle.as_mut() {
            if let Err(err) = reload_handle.modify(|max_level| *max_level = level) {
                error!("failed to set max logging level: {err}");
            }
        }
    }

    fn add_sink<S: Sink + Clone + 'static>(&self, sink: &S) {
        self.inner
            .lock()
            .unwrap()
            .sinks
            .insert(TypeId::of::<S>(), Box::new(sink.clone()));
    }

    fn remove_sink<S: Sink + Clone + 'static>(&self, _sink: &S) {
        self.inner.lock().unwrap().sinks.remove(&TypeId::of::<S>());
    }

    fn log(
        &self,
        level: &Level,
        msg: &str,
        args: Option<&str>,
        file: Option<&str>,
        line: Option<u32>,
    ) {
        for sink in self.inner.lock().unwrap().sinks.values() {
            if sink.enabled(level) {
                sink.log(level, msg, args, file, line);
            }
        }
    }

    fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.sinks.clear();
        inner.sinks.shrink_to(0);
    }

    fn flush(&self) {
        for sink in self.inner.lock().unwrap().sinks.values() {
            sink.flush();
        }
    }
}

impl<S> Layer<S> for Logger
where
    S: Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = StringVisitor::default(); // note: could use a fixed size buffer here.
        event.record(&mut visitor);

        let level = event.metadata().level();
        let file = event.metadata().file();
        let line = event.metadata().line();

        let msg = &visitor.msg;
        let args = if visitor.args.is_empty() {
            None
        } else {
            Some(visitor.args.as_str())
        };

        self.log(level, msg, args, file, line);
    }
}

#[derive(Default)]
struct StringVisitor {
    msg: String,
    args: String,
}

impl StringVisitor {
    fn record_display(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Display) {
        if field.name() == "message" {
            _ = write!(&mut self.msg, "{}", value);
        } else {
            if !self.args.is_empty() {
                _ = write!(&mut self.args, " ");
            }
            _ = write!(&mut self.args, "{}={}", field.name(), value);
        }
    }
}

impl Visit for StringVisitor {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.record_display(field, &value)
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.record_display(field, &value)
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.record_display(field, &value)
    }

    fn record_i128(&mut self, field: &tracing::field::Field, value: i128) {
        self.record_display(field, &value)
    }

    fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
        self.record_display(field, &value)
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.record_display(field, &value)
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.record_display(field, &value)
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.record_display(field, &tracing::field::display(value))
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            _ = write!(&mut self.msg, "{:?}", value);
        } else {
            if !self.args.is_empty() {
                _ = write!(&mut self.args, " ");
            }
            _ = write!(&mut self.args, "{}={:?}", field.name(), value);
        }
    }
}

pub trait Sink {
    fn enabled(&self, level: &Level) -> bool;

    fn log(
        &self,
        level: &Level,
        msg: &str,
        args: Option<&str>,
        file: Option<&str>,
        line: Option<u32>,
    );

    fn flush(&self);
}

pub fn startup(max_level: LevelFilter) -> Result<(), LoggerError> {
    let (max_level, reload_handle) = reload::Layer::new(max_level);
    let logger = LOGGER.get_or_init(|| Logger::new(reload_handle));
    let subscriber = tracing_subscriber::registry()
        .with(max_level)
        .with(logger.clone());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

pub fn shutdown() {
    if let Some(logger) = LOGGER.get() {
        logger.flush();
        logger.clear();
        logger.set_max_level(LevelFilter::OFF);
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
    if let Some(logger) = LOGGER.get() {
        logger.set_max_level(level);
    }
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

impl From<SetGlobalDefaultError> for LoggerError {
    fn from(_: SetGlobalDefaultError) -> Self {
        LoggerError::AlreadyInitialized
    }
}

impl From<LoggerError> for Error {
    fn from(err: LoggerError) -> Self {
        Error::new("failed to initialize logger").with_source(err)
    }
}

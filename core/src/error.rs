use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<Box<dyn std::error::Error>>,
}

impl Error {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source<E: std::error::Error + 'static>(self, source: E) -> Self {
        Self {
            source: Some(Box::new(source)),
            ..self
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref()
    }
}

use std::fmt;
use std::fmt::{Debug, Formatter};

pub type Result<T> = std::result::Result<T, PushMetricsError>;

#[derive(Debug)]
pub enum PushMetricsError {
    Prometheus(prometheus::Error),
    Url(url::ParseError),
    Generic(String),
    #[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
    Reqwest(reqwest::Error),
}

impl std::error::Error for PushMetricsError {}

impl fmt::Display for PushMetricsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PushMetricsError::Prometheus(e) => std::fmt::Display::fmt(e, f),
            PushMetricsError::Url(e) => std::fmt::Display::fmt(e, f),
            PushMetricsError::Generic(e) => std::fmt::Display::fmt(e, f),
            #[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
            PushMetricsError::Reqwest(e) => std::fmt::Display::fmt(e, f),
        }
    }
}

impl From<prometheus::Error> for PushMetricsError {
    fn from(prometheus_error: prometheus::Error) -> Self {
        PushMetricsError::Prometheus(prometheus_error)
    }
}

impl From<url::ParseError> for PushMetricsError {
    fn from(error: url::ParseError) -> Self {
        PushMetricsError::Url(error)
    }
}

#[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
impl From<reqwest::Error> for PushMetricsError {
    fn from(error: reqwest::Error) -> Self {
        PushMetricsError::Reqwest(error)
    }
}

use std::fmt;
use std::fmt::{Debug, Formatter};

pub type Result<T> = std::result::Result<T, PushMetricsError>;

#[derive(Debug)]
pub enum PushMetricsError {
    Prometheus(prometheus::Error),

    Reqwest(reqwest::Error),
    Generic(String),
}

impl std::error::Error for PushMetricsError {}

impl fmt::Display for PushMetricsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PushMetricsError::Prometheus(e) => std::fmt::Display::fmt(e, f),
            PushMetricsError::Reqwest(e) => std::fmt::Display::fmt(e, f),
            PushMetricsError::Generic(e) => std::fmt::Display::fmt(e, f),
        }
    }
}

impl From<prometheus::Error> for PushMetricsError {
    fn from(prometheus_error: prometheus::Error) -> Self {
        PushMetricsError::Prometheus(prometheus_error)
    }
}

impl From<reqwest::Error> for PushMetricsError {
    fn from(error: reqwest::Error) -> Self {
        PushMetricsError::Reqwest(error)
    }
}

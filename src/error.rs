use std::fmt;
use std::fmt::{Debug, Formatter};

#[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
use reqwest::StatusCode;
#[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
use url::Url;

pub type Result<T> = std::result::Result<T, PushMetricsError>;

/// PushMetricsError is the crates returned error type
#[derive(Debug)]
pub enum PushMetricsError {
    Url(url::ParseError),
    AlreadyContainsLabel(String),
    SlashInName(String),
    #[cfg(feature = "prometheus_crate")]
    Prometheus(prometheus::Error),
    #[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
    Response(String),
    #[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
    Reqwest(reqwest::Error),
}

impl std::error::Error for PushMetricsError {}

impl PushMetricsError {
    #[cfg(feature = "prometheus_crate")]
    pub(crate) fn contains_label(metric: &str, label_type: LabelType<'_>) -> Self {
        let message = format!(
            "pushed metric {metric} already contains {label}",
            label = label_type.message()
        );

        PushMetricsError::AlreadyContainsLabel(message)
    }

    #[cfg(any(feature = "prometheus_crate", feature = "prometheus_client_crate"))]
    pub(crate) fn slash_in_name(value: &str) -> Self {
        let message = format!("labels and job name must not contain '/': '{value}'");
        PushMetricsError::SlashInName(message)
    }

    #[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
    pub(crate) fn response(status_code: &StatusCode, url: &Url) -> Self {
        PushMetricsError::Response(format!(
            "unexpected status code {status_code} while pushing to {url}",
        ))
    }
}

#[cfg(feature = "prometheus_crate")]
#[derive(Debug)]
pub(crate) enum LabelType<'a> {
    Job,
    Grouping(&'a str),
}

#[cfg(feature = "prometheus_crate")]
impl<'a> LabelType<'a> {
    fn message(&self) -> String {
        match self {
            LabelType::Job => String::from("a job label"),
            LabelType::Grouping(label) => format!("grouping label with value '{label}'"),
        }
    }
}

impl fmt::Display for PushMetricsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PushMetricsError::Url(e) => std::fmt::Display::fmt(e, f),
            PushMetricsError::AlreadyContainsLabel(message) => std::fmt::Display::fmt(message, f),
            PushMetricsError::SlashInName(message) => std::fmt::Display::fmt(message, f),
            #[cfg(feature = "prometheus_crate")]
            PushMetricsError::Prometheus(e) => std::fmt::Display::fmt(e, f),
            #[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
            PushMetricsError::Response(e) => std::fmt::Display::fmt(e, f),
            #[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
            PushMetricsError::Reqwest(e) => std::fmt::Display::fmt(e, f),
        }
    }
}

impl From<url::ParseError> for PushMetricsError {
    fn from(error: url::ParseError) -> Self {
        PushMetricsError::Url(error)
    }
}

#[cfg(feature = "prometheus_crate")]
impl From<prometheus::Error> for PushMetricsError {
    fn from(prometheus_error: prometheus::Error) -> Self {
        PushMetricsError::Prometheus(prometheus_error)
    }
}

#[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
impl From<reqwest::Error> for PushMetricsError {
    fn from(error: reqwest::Error) -> Self {
        PushMetricsError::Reqwest(error)
    }
}

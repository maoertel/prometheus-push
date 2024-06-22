use std::fmt::Debug;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, PushMetricsError>;

/// `PushMetricsError` is the crates returned error type
#[derive(Error, Debug)]
pub enum PushMetricsError {
    #[error("error parsing url: {0}")]
    Url(#[from] url::ParseError),
    #[error("pushed metric already contains label '{0}'")]
    AlreadyContainsLabel(String),
    #[error("labels and job name must not contain '/': '{0}'")]
    SlashInName(String),
    #[cfg(feature = "prometheus_crate")]
    #[error("prometheus error: {0}")]
    Prometheus(#[from] prometheus::Error),
    #[cfg(feature = "prometheus_client_crate")]
    #[error("prometheus client error: {0}")]
    PrometheusClient(#[from] std::fmt::Error),
    #[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
    #[error("unexpected status code while pushing to url")]
    Response(String),
    #[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

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
    pub(crate) fn response(status_code: &reqwest::StatusCode, url: &url::Url) -> Self {
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

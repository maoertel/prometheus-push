#[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
use reqwest::StatusCode;
#[cfg(feature = "prometheus_crate")]
use std::collections::HashMap;
use url::Url;

#[cfg(feature = "prometheus_crate")]
use crate::error::PushMetricsError;
use crate::error::Result;

const METRICS_JOB_PATH: &str = "metrics/job/";

pub(crate) fn create_metrics_job_url(url: &Url) -> Result<Url> {
    Ok(url.join(METRICS_JOB_PATH)?)
}

#[cfg(feature = "prometheus_crate")]
pub(crate) fn build_url(url: &Url, job: &str, grouping: &HashMap<&str, &str>) -> Result<Url> {
    let mut url_params = vec![job];

    for (label_name, label_value) in grouping {
        url_params.push(validate(label_name)?);
        url_params.push(validate(label_value)?);
    }

    Ok(url.join(&url_params.join("/"))?)
}

#[cfg(feature = "prometheus_crate")]
pub(crate) fn validate(value: &str) -> Result<&str> {
    if value.contains('/') {
        return Err(PushMetricsError::slash_in_name(value));
    }

    Ok(value)
}

#[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
pub(crate) trait Respond {
    fn get_status_code(&self) -> StatusCode;
    fn get_url(&self) -> &Url;
}

#[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
pub(crate) fn handle_response<R: Respond>(response: &R) -> Result<()> {
    match response.get_status_code() {
        StatusCode::ACCEPTED | StatusCode::OK => {
            log::info!("Pushed metrics to the pushgateway.");
            Ok(())
        }
        status_code => Err(PushMetricsError::response(&status_code, response.get_url())),
    }
}

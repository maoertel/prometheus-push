use std::collections::HashMap;
use std::hash::BuildHasher;

use prometheus::core::Collector;
use prometheus::proto::MetricFamily;
use prometheus::Encoder;
use prometheus::ProtobufEncoder;
use prometheus::Registry;
#[cfg(any(feature = "with_reqwest", feature = "with_reqwest_blocking"))]
use reqwest::StatusCode;
use url::Url;

use crate::error::PushMetricsError;
use crate::error::Result;

const LABEL_NAME_JOB: &str = "job";
const METRICS_JOB_PATH: &str = "metrics/job/";

pub(crate) fn create_metrics_job_url(url: &Url) -> Result<Url> {
    Ok(url.join(METRICS_JOB_PATH)?)
}

pub(crate) fn create<'a, BH: BuildHasher>(
    job: &'a str,
    url: &'a Url,
    grouping: &HashMap<&str, &str, BH>,
    metric_families: Vec<MetricFamily>,
) -> Result<(Url, Vec<u8>, ProtobufEncoder)> {
    let job = validate_job(job)?;
    let url = build_url(url, job, grouping)?;
    let encoder = ProtobufEncoder::new();
    let encoded_metrics = encode_metrics(&encoder, grouping, metric_families)?;

    Ok((url, encoded_metrics, encoder))
}

pub(crate) fn metric_families_from(
    collectors: Vec<Box<dyn Collector>>,
) -> Result<Vec<MetricFamily>> {
    let registry = Registry::new();
    for collector in collectors {
        registry.register(collector)?;
    }

    Ok(registry.gather())
}

fn validate_job(job: &str) -> Result<&str> {
    if job.contains('/') {
        return Err(PushMetricsError::Generic(format!(
            "job name must not contain '/': {job}"
        )));
    }

    Ok(job)
}

fn build_url<'a, BH: BuildHasher>(
    url: &'a Url,
    job: &'a str,
    grouping: &'a HashMap<&'a str, &'a str, BH>,
) -> Result<Url> {
    let mut url_params = vec![job];

    for (label_name, label_value) in grouping {
        if label_value.contains('/') {
            return Err(PushMetricsError::Generic(format!(
                "value of grouping label {label_name} contains '/': {label_value}",
            )));
        }
        url_params.push(label_name);
        url_params.push(label_value);
    }

    Ok(url.join(&url_params.join("/"))?)
}

fn encode_metrics<'a, BH: BuildHasher>(
    encoder: &ProtobufEncoder,
    grouping: &'a HashMap<&'a str, &'a str, BH>,
    metric_families: Vec<MetricFamily>,
) -> Result<Vec<u8>> {
    let mut encoded_metrics = Vec::new();
    for metric_family in metric_families {
        for metric in metric_family.get_metric() {
            for label_pair in metric.get_label() {
                let label_name = label_pair.get_name();

                if LABEL_NAME_JOB == label_name {
                    return Err(PushMetricsError::Generic(format!(
                        "pushed metric {metric} already contains a job label",
                        metric = metric_family.get_name()
                    )));
                }

                if grouping.contains_key(label_name) {
                    return Err(PushMetricsError::Generic(format!(
                        "pushed metric {metric} already contains grouping label {label_name}",
                        metric = metric_family.get_name(),
                    )));
                }
            }
        }

        encoder.encode(&[metric_family], &mut encoded_metrics)?;
    }

    Ok(encoded_metrics)
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
            log::info!("Pushed metrics to the push gateway.");
            Ok(())
        }
        status_code => Err(PushMetricsError::Generic(format!(
            "unexpected status code {status_code} while pushing to {url}",
            url = response.get_url()
        ))),
    }
}

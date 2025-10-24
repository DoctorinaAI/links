use axum::http::HeaderValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::Span;

/// Performance timing tracker for Server-Timing header
///
/// This struct tracks multiple timing metrics during request processing.
/// It's designed to be stored in request extensions and accessed throughout
/// the request lifecycle.
///
/// Uses tracing spans under the hood for efficient, lock-free measurements.
///
/// # Example
/// ```ignore
/// let metrics = RequestMetrics::new();
///
/// metrics.measure_fn("database", || {
///     // Database operations...
/// });
///
/// let header = metrics.build_header();
/// ```
#[derive(Clone)]
pub struct RequestMetrics {
    metrics: Arc<Mutex<HashMap<String, Vec<MetricSample>>>>,
}

struct MetricSample {
    start: Instant,
    duration_us: Option<u128>,
}

impl RequestMetrics {
    /// Create a new metrics tracker
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start measuring a metric and return a guard
    /// When the guard is dropped, the measurement stops automatically
    ///
    /// # Example
    /// ```ignore
    /// let _guard = metrics.measure("json_decode");
    /// serde_json::from_str(&data)?;
    /// // Guard is dropped here, measurement stops
    /// ```
    pub fn measure(&self, name: impl Into<String>) -> MetricGuard {
        let name = name.into();
        let start = Instant::now();

        // Create tracing span for structured logging
        let span = tracing::debug_span!("metric", name = %name);

        // Add new sample to the metric
        {
            let _enter = span.enter();
            let mut metrics = self.metrics.lock().unwrap();
            metrics.entry(name.clone()).or_default().push(MetricSample {
                start,
                duration_us: None,
            });
        }

        MetricGuard {
            name,
            metrics: Arc::clone(&self.metrics),
            start,
            stopped: false,
            _span: span,
        }
    }

    /// Measure a closure execution time
    /// This is a convenience method that automatically handles the guard lifecycle
    ///
    /// # Example
    /// ```ignore
    /// let result = metrics.measure_fn("database", || {
    ///     database.query("SELECT * FROM users")
    /// })?;
    /// ```
    pub fn measure_fn<F, T>(&self, name: impl Into<String>, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _guard = self.measure(name);
        f()
    }

    /// Measure an async closure execution time
    /// This is a convenience method for async operations
    ///
    /// # Example
    /// ```ignore
    /// let result = metrics.measure_async("database", async {
    ///     database.query("SELECT * FROM users").await
    /// }).await?;
    /// ```
    pub async fn measure_async<F, Fut, T>(&self, name: impl Into<String>, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let _guard = self.measure(name);
        f().await
    }

    /// Manually record a metric duration (in microseconds)
    /// Use this when you can't use the guard pattern
    #[allow(dead_code)]
    pub fn record(&self, name: impl Into<String>, duration_us: u128) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.entry(name.into()).or_default().push(MetricSample {
            start: Instant::now(), // Dummy value
            duration_us: Some(duration_us),
        });
    }

    /// Build the Server-Timing header value from collected metrics
    ///
    /// Format: metric1;dur=1.23, metric2;dur=4.56;desc="description"
    /// Durations are in milliseconds with microsecond precision
    pub fn build_header(&self) -> Option<HeaderValue> {
        let metrics = self.metrics.lock().unwrap();

        if metrics.is_empty() {
            return None;
        }

        let mut timings = Vec::new();

        for (name, samples) in metrics.iter() {
            // Calculate total duration for this metric
            let total_us: u128 = samples.iter().map(|s| s.duration_us.unwrap_or(0)).sum();

            if total_us > 0 {
                // Convert microseconds to milliseconds with precision
                let duration_ms = total_us as f64 / 1000.0;

                // Add count if multiple samples
                if samples.len() > 1 {
                    timings.push(format!(
                        "{};dur={:.3};desc=\"{} calls\"",
                        sanitize_metric_name(name),
                        duration_ms,
                        samples.len()
                    ));
                } else {
                    timings.push(format!(
                        "{};dur={:.3}",
                        sanitize_metric_name(name),
                        duration_ms
                    ));
                }
            }
        }

        if timings.is_empty() {
            return None;
        }

        let header_value = timings.join(", ");
        HeaderValue::from_str(&header_value).ok()
    }

    /// Get summary of all metrics for logging
    #[allow(dead_code)]
    pub fn summary(&self) -> HashMap<String, (usize, u128)> {
        let metrics = self.metrics.lock().unwrap();
        let mut summary = HashMap::new();

        for (name, samples) in metrics.iter() {
            let total_us: u128 = samples.iter().map(|s| s.duration_us.unwrap_or(0)).sum();
            summary.insert(name.clone(), (samples.len(), total_us));
        }

        summary
    }
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for automatic metric measurement
///
/// When dropped, automatically stops the timer and records the duration
pub struct MetricGuard {
    name: String,
    metrics: Arc<Mutex<HashMap<String, Vec<MetricSample>>>>,
    start: Instant,
    stopped: bool,
    _span: Span,
}

impl MetricGuard {
    /// Manually stop the timer (optional, happens automatically on drop)
    #[allow(dead_code)]
    pub fn stop(mut self) {
        self.stop_internal();
    }

    fn stop_internal(&mut self) {
        if self.stopped {
            return;
        }

        let duration_us = self.start.elapsed().as_micros();

        let mut metrics = self.metrics.lock().unwrap();
        if let Some(samples) = metrics.get_mut(&self.name) {
            // Find the last sample without duration and update it
            if let Some(sample) = samples.iter_mut().rev().find(|s| s.duration_us.is_none()) {
                sample.duration_us = Some(duration_us);
            }
        }

        self.stopped = true;
    }
}

impl Drop for MetricGuard {
    fn drop(&mut self) {
        self.stop_internal();
    }
}

/// Sanitize metric name for Server-Timing header
/// Replaces invalid characters with underscores
fn sanitize_metric_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_measurement() {
        let metrics = RequestMetrics::new();

        {
            let _guard = metrics.measure("test");
            thread::sleep(Duration::from_millis(10));
        }

        let summary = metrics.summary();
        assert_eq!(summary.len(), 1);
        assert!(summary.get("test").unwrap().1 >= 10_000); // At least 10ms in microseconds
    }

    #[test]
    fn test_multiple_measurements() {
        let metrics = RequestMetrics::new();

        {
            let _guard1 = metrics.measure("db");
            thread::sleep(Duration::from_millis(5));
        }

        {
            let _guard2 = metrics.measure("cache");
            thread::sleep(Duration::from_millis(3));
        }

        let summary = metrics.summary();
        assert_eq!(summary.len(), 2);
        assert!(summary.contains_key("db"));
        assert!(summary.contains_key("cache"));
    }

    #[test]
    fn test_nested_measurements() {
        let metrics = RequestMetrics::new();

        let _guard1 = metrics.measure("outer");
        {
            let _guard2 = metrics.measure("inner");
            thread::sleep(Duration::from_millis(5));
        }
        thread::sleep(Duration::from_millis(5));
        drop(_guard1);

        let summary = metrics.summary();
        assert_eq!(summary.len(), 2);
    }

    #[test]
    fn test_same_metric_multiple_times() {
        let metrics = RequestMetrics::new();

        for _ in 0..3 {
            let _guard = metrics.measure("db_query");
            thread::sleep(Duration::from_millis(2));
        }

        let summary = metrics.summary();
        assert_eq!(summary.get("db_query").unwrap().0, 3); // 3 calls
    }

    #[test]
    fn test_header_format() {
        let metrics = RequestMetrics::new();

        {
            let _guard = metrics.measure("test");
            thread::sleep(Duration::from_millis(10));
        }

        let header = metrics.build_header();
        assert!(header.is_some());

        let header_value = header.unwrap();
        let header_str = header_value.to_str().unwrap();
        assert!(header_str.contains("test"));
        assert!(header_str.contains("dur="));
    }

    #[test]
    fn test_sanitize_metric_name() {
        assert_eq!(sanitize_metric_name("valid-name_123"), "valid-name_123");
        assert_eq!(sanitize_metric_name("invalid name!@#"), "invalid_name___");
        assert_eq!(sanitize_metric_name("test/path"), "test_path");
    }

    #[test]
    fn test_measure_fn() {
        let metrics = RequestMetrics::new();

        let result = metrics.measure_fn("computation", || {
            thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        let summary = metrics.summary();
        assert_eq!(summary.len(), 1);
        assert!(summary.get("computation").unwrap().1 >= 10_000);
    }

    #[tokio::test]
    async fn test_measure_async() {
        let metrics = RequestMetrics::new();

        let result = metrics
            .measure_async("async_operation", || async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                "completed"
            })
            .await;

        assert_eq!(result, "completed");
        let summary = metrics.summary();
        assert_eq!(summary.len(), 1);
        assert!(summary.get("async_operation").unwrap().1 >= 10_000);
    }

    #[test]
    fn test_measure_fn_with_result() {
        let metrics = RequestMetrics::new();

        let result: Result<i32, &str> = metrics.measure_fn("fallible_op", || Ok(100));

        assert_eq!(result, Ok(100));
        let summary = metrics.summary();
        assert!(summary.contains_key("fallible_op"));
    }
}

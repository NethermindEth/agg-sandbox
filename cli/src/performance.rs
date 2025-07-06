#![allow(dead_code)]

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Performance metrics for monitoring async operations
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub total_duration: Duration,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub avg_duration: Duration,
}

impl PerformanceMetrics {
    pub fn new(operation_name: String) -> Self {
        Self {
            operation_name,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_duration: Duration::ZERO,
            min_duration: None,
            max_duration: None,
            avg_duration: Duration::ZERO,
        }
    }

    pub fn record_success(&mut self, duration: Duration) {
        self.total_calls += 1;
        self.successful_calls += 1;
        self.total_duration += duration;

        self.min_duration = Some(
            self.min_duration
                .map(|min| min.min(duration))
                .unwrap_or(duration),
        );

        self.max_duration = Some(
            self.max_duration
                .map(|max| max.max(duration))
                .unwrap_or(duration),
        );

        self.avg_duration = self.total_duration / self.total_calls as u32;
    }

    pub fn record_failure(&mut self, duration: Duration) {
        self.total_calls += 1;
        self.failed_calls += 1;
        self.total_duration += duration;

        self.min_duration = Some(
            self.min_duration
                .map(|min| min.min(duration))
                .unwrap_or(duration),
        );

        self.max_duration = Some(
            self.max_duration
                .map(|max| max.max(duration))
                .unwrap_or(duration),
        );

        if self.total_calls > 0 {
            self.avg_duration = self.total_duration / self.total_calls as u32;
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.successful_calls as f64 / self.total_calls as f64
        }
    }

    pub fn failure_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.failed_calls as f64 / self.total_calls as f64
        }
    }
}

/// Performance monitor for tracking async operation performance
pub struct PerformanceMonitor {
    metrics: Arc<DashMap<String, Arc<RwLock<PerformanceMetrics>>>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
        }
    }

    /// Get or create metrics for an operation
    async fn get_metrics(&self, operation_name: &str) -> Arc<RwLock<PerformanceMetrics>> {
        if let Some(metrics) = self.metrics.get(operation_name) {
            metrics.clone()
        } else {
            let new_metrics = Arc::new(RwLock::new(PerformanceMetrics::new(
                operation_name.to_string(),
            )));
            self.metrics
                .insert(operation_name.to_string(), new_metrics.clone());
            new_metrics
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self, operation_name: &str, duration: Duration) {
        let metrics = self.get_metrics(operation_name).await;
        let mut metrics = metrics.write().await;
        metrics.record_success(duration);

        debug!(
            operation = operation_name,
            duration = ?duration,
            total_calls = metrics.total_calls,
            "Recorded successful operation"
        );
    }

    /// Record a failed operation
    pub async fn record_failure(&self, operation_name: &str, duration: Duration) {
        let metrics = self.get_metrics(operation_name).await;
        let mut metrics = metrics.write().await;
        metrics.record_failure(duration);

        warn!(
            operation = operation_name,
            duration = ?duration,
            total_calls = metrics.total_calls,
            failure_rate = metrics.failure_rate(),
            "Recorded failed operation"
        );
    }

    /// Get metrics for an operation
    pub async fn get_operation_metrics(&self, operation_name: &str) -> Option<PerformanceMetrics> {
        if let Some(metrics) = self.metrics.get(operation_name) {
            Some(metrics.read().await.clone())
        } else {
            None
        }
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> Vec<PerformanceMetrics> {
        let mut all_metrics = Vec::new();

        for entry in self.metrics.iter() {
            let metrics = entry.value().read().await;
            all_metrics.push(metrics.clone());
        }

        all_metrics
    }

    /// Clear all metrics
    pub async fn clear_metrics(&self) {
        self.metrics.clear();
        info!("Performance metrics cleared");
    }

    /// Print performance summary
    pub async fn print_summary(&self) {
        let all_metrics = self.get_all_metrics().await;

        if all_metrics.is_empty() {
            println!("No performance metrics available");
            return;
        }

        println!("\n📊 Performance Summary");
        println!("{}", "═".repeat(80));
        println!(
            "{:<20} {:>8} {:>8} {:>8} {:>10} {:>10} {:>10}",
            "Operation", "Calls", "Success", "Failed", "Avg (ms)", "Min (ms)", "Max (ms)"
        );
        println!("{}", "─".repeat(80));

        for metric in all_metrics {
            println!(
                "{:<20} {:>8} {:>8} {:>8} {:>10.1} {:>10.1} {:>10.1}",
                metric.operation_name,
                metric.total_calls,
                metric.successful_calls,
                metric.failed_calls,
                metric.avg_duration.as_millis() as f64,
                metric
                    .min_duration
                    .map(|d| d.as_millis() as f64)
                    .unwrap_or(0.0),
                metric
                    .max_duration
                    .map(|d| d.as_millis() as f64)
                    .unwrap_or(0.0),
            );
        }
        println!("{}", "═".repeat(80));
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for measuring async operation performance
pub trait AsyncTimer {
    #[allow(async_fn_in_trait)]
    async fn time_async<F, R, E>(&self, operation_name: &str, future: F) -> Result<R, E>
    where
        F: std::future::Future<Output = Result<R, E>>;
}

impl AsyncTimer for PerformanceMonitor {
    async fn time_async<F, R, E>(&self, operation_name: &str, future: F) -> Result<R, E>
    where
        F: std::future::Future<Output = Result<R, E>>,
    {
        let start = Instant::now();
        let result = future.await;
        let duration = start.elapsed();

        match result {
            Ok(ref _value) => {
                self.record_success(operation_name, duration).await;
            }
            Err(ref _error) => {
                self.record_failure(operation_name, duration).await;
            }
        }

        result
    }
}

/// Global performance monitor instance
static GLOBAL_MONITOR: once_cell::sync::Lazy<PerformanceMonitor> =
    once_cell::sync::Lazy::new(PerformanceMonitor::new);

/// Get the global performance monitor
pub fn global_performance_monitor() -> &'static PerformanceMonitor {
    &GLOBAL_MONITOR
}

/// Macro for easily timing async operations
#[macro_export]
macro_rules! time_async {
    ($operation:expr, $future:expr) => {{
        use $crate::performance::{global_performance_monitor, AsyncTimer};
        global_performance_monitor()
            .time_async($operation, $future)
            .await
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[test]
    fn test_performance_metrics_new() {
        let metrics = PerformanceMetrics::new("test_operation".to_string());
        assert_eq!(metrics.operation_name, "test_operation");
        assert_eq!(metrics.total_calls, 0);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_performance_metrics_record_success() {
        let mut metrics = PerformanceMetrics::new("test".to_string());
        let duration = Duration::from_millis(100);

        metrics.record_success(duration);

        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 1);
        assert_eq!(metrics.failed_calls, 0);
        assert_eq!(metrics.avg_duration, duration);
        assert_eq!(metrics.success_rate(), 1.0);
    }

    #[test]
    fn test_performance_metrics_record_failure() {
        let mut metrics = PerformanceMetrics::new("test".to_string());
        let duration = Duration::from_millis(50);

        metrics.record_failure(duration);

        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 0);
        assert_eq!(metrics.failed_calls, 1);
        assert_eq!(metrics.failure_rate(), 1.0);
    }

    #[test]
    fn test_performance_metrics_multiple_records() {
        let mut metrics = PerformanceMetrics::new("test".to_string());

        metrics.record_success(Duration::from_millis(100));
        metrics.record_success(Duration::from_millis(200));
        metrics.record_failure(Duration::from_millis(150));

        assert_eq!(metrics.total_calls, 3);
        assert_eq!(metrics.successful_calls, 2);
        assert_eq!(metrics.failed_calls, 1);
        assert_eq!(metrics.success_rate(), 2.0 / 3.0);
        assert_eq!(metrics.min_duration, Some(Duration::from_millis(100)));
        assert_eq!(metrics.max_duration, Some(Duration::from_millis(200)));
    }

    #[tokio::test]
    async fn test_performance_monitor_record() {
        let monitor = PerformanceMonitor::new();

        monitor
            .record_success("test_op", Duration::from_millis(100))
            .await;
        monitor
            .record_failure("test_op", Duration::from_millis(50))
            .await;

        let metrics = monitor.get_operation_metrics("test_op").await.unwrap();
        assert_eq!(metrics.total_calls, 2);
        assert_eq!(metrics.successful_calls, 1);
        assert_eq!(metrics.failed_calls, 1);
    }

    #[tokio::test]
    async fn test_async_timer() {
        let monitor = PerformanceMonitor::new();

        // Test successful operation
        let result: Result<i32, &str> = monitor
            .time_async("test_success", async {
                sleep(Duration::from_millis(1)).await;
                Ok(42)
            })
            .await;

        assert_eq!(result.unwrap(), 42);

        // Test failed operation
        let result: Result<i32, &str> = monitor
            .time_async("test_failure", async {
                sleep(Duration::from_millis(1)).await;
                Err("test error")
            })
            .await;

        assert!(result.is_err());

        let success_metrics = monitor.get_operation_metrics("test_success").await.unwrap();
        let failure_metrics = monitor.get_operation_metrics("test_failure").await.unwrap();

        assert_eq!(success_metrics.successful_calls, 1);
        assert_eq!(failure_metrics.failed_calls, 1);
    }

    #[tokio::test]
    async fn test_monitor_clear() {
        let monitor = PerformanceMonitor::new();

        monitor
            .record_success("test", Duration::from_millis(100))
            .await;

        let metrics_before = monitor.get_all_metrics().await;
        assert!(!metrics_before.is_empty());

        monitor.clear_metrics().await;

        let metrics_after = monitor.get_all_metrics().await;
        assert!(metrics_after.is_empty());
    }
}

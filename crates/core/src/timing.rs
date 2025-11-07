//! High-precision timing engine using quanta
//!
//! Provides nanosecond-precision timing with minimal overhead for
//! accurate latency measurement of LLM API requests.

use std::fmt;
use std::time::Duration;

/// High-precision clock using quanta for nanosecond timing
#[derive(Clone)]
pub struct Clock {
    clock: quanta::Clock,
}

impl Clock {
    /// Create a new high-precision clock
    pub fn new() -> Self {
        Self {
            clock: quanta::Clock::new(),
        }
    }

    /// Get the current timestamp
    #[inline]
    pub fn now(&self) -> Timestamp {
        Timestamp {
            instant: self.clock.now(),
        }
    }

    /// Measure the duration of a synchronous operation
    #[inline]
    pub fn measure<F, T>(&self, f: F) -> (T, Duration)
    where
        F: FnOnce() -> T,
    {
        let start = self.now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Measure the duration of an async operation
    #[inline]
    pub async fn measure_async<F, Fut, T>(&self, f: F) -> (T, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = self.now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

/// High-precision timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    instant: quanta::Instant,
}

impl Timestamp {
    /// Calculate duration since this timestamp
    #[inline]
    pub fn elapsed(&self) -> Duration {
        let now = quanta::Instant::now();
        now.duration_since(self.instant)
    }

    /// Calculate duration between two timestamps
    #[inline]
    pub fn duration_since(&self, earlier: Timestamp) -> Duration {
        self.instant.duration_since(earlier.instant)
    }

    /// Get raw nanosecond value (approximate, for display only)
    #[inline]
    pub fn as_nanos(&self) -> u64 {
        // Since we can't get raw value, use a reference point
        // This is mainly for display purposes
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Timestamp")
    }
}

/// Timing engine for measuring LLM request latency
///
/// Provides high-precision timing with minimal overhead:
/// - Nanosecond resolution using quanta
/// - Inline functions for zero-cost abstractions
/// - Low overhead (<5μs per measurement)
pub struct TimingEngine {
    clock: Clock,
}

impl TimingEngine {
    /// Create a new timing engine
    pub fn new() -> Self {
        Self {
            clock: Clock::new(),
        }
    }

    /// Start a new timing measurement
    #[inline]
    pub fn start(&self) -> TimingMeasurement {
        TimingMeasurement::new(&self.clock)
    }

    /// Get the underlying clock
    #[inline]
    pub fn clock(&self) -> &Clock {
        &self.clock
    }
}

impl Default for TimingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// A single timing measurement tracking multiple checkpoints
pub struct TimingMeasurement {
    clock: Clock,
    start: Timestamp,
    checkpoints: Vec<(String, Timestamp)>,
}

impl TimingMeasurement {
    /// Create a new timing measurement
    #[inline]
    pub fn new(clock: &Clock) -> Self {
        Self {
            clock: clock.clone(),
            start: clock.now(),
            checkpoints: Vec::new(),
        }
    }

    /// Record a checkpoint with a label
    #[inline]
    pub fn checkpoint<S: Into<String>>(&mut self, label: S) {
        self.checkpoints.push((label.into(), self.clock.now()));
    }

    /// Get the start timestamp
    #[inline]
    pub fn start_time(&self) -> Timestamp {
        self.start
    }

    /// Get all checkpoints
    pub fn checkpoints(&self) -> &[(String, Timestamp)] {
        &self.checkpoints
    }

    /// Calculate total duration since start
    #[inline]
    pub fn total_duration(&self) -> Duration {
        self.start.elapsed()
    }

    /// Calculate duration between consecutive checkpoints
    pub fn checkpoint_durations(&self) -> Vec<(String, Duration)> {
        let mut durations = Vec::new();
        let mut prev = self.start;

        for (label, timestamp) in &self.checkpoints {
            let duration = timestamp.duration_since(prev);
            durations.push((label.clone(), duration));
            prev = *timestamp;
        }

        durations
    }

    /// Convert to a timing result
    pub fn finish(self) -> TimingResult {
        let total = self.total_duration();
        let checkpoints = self.checkpoint_durations();

        TimingResult {
            total_duration: total,
            checkpoints,
        }
    }
}

/// Result of a timing measurement
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimingResult {
    /// Total duration of the measurement
    pub total_duration: Duration,
    /// Duration between consecutive checkpoints
    pub checkpoints: Vec<(String, Duration)>,
}

impl TimingResult {
    /// Get a specific checkpoint duration by label
    pub fn get_checkpoint(&self, label: &str) -> Option<Duration> {
        self.checkpoints
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, d)| *d)
    }

    /// Get total duration in nanoseconds
    #[inline]
    pub fn total_nanos(&self) -> u64 {
        self.total_duration.as_nanos() as u64
    }

    /// Get total duration in microseconds
    #[inline]
    pub fn total_micros(&self) -> u64 {
        self.total_duration.as_micros() as u64
    }

    /// Get total duration in milliseconds
    #[inline]
    pub fn total_millis(&self) -> u64 {
        self.total_duration.as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_clock_now() {
        let clock = Clock::new();
        let ts1 = clock.now();
        thread::sleep(Duration::from_micros(100));
        let ts2 = clock.now();

        assert!(ts2 > ts1);
    }

    #[test]
    fn test_timestamp_elapsed() {
        let clock = Clock::new();
        let ts = clock.now();
        thread::sleep(Duration::from_micros(100));
        let elapsed = ts.elapsed();

        assert!(elapsed.as_micros() >= 100);
    }

    #[test]
    fn test_clock_measure() {
        let clock = Clock::new();
        let (result, duration) = clock.measure(|| {
            thread::sleep(Duration::from_micros(100));
            42
        });

        assert_eq!(result, 42);
        assert!(duration.as_micros() >= 100);
    }

    #[tokio::test]
    async fn test_clock_measure_async() {
        let clock = Clock::new();
        let (result, duration) = clock
            .measure_async(|| async {
                tokio::time::sleep(Duration::from_micros(100)).await;
                42
            })
            .await;

        assert_eq!(result, 42);
        assert!(duration.as_micros() >= 100);
    }

    #[test]
    fn test_timing_measurement() {
        let engine = TimingEngine::new();
        let mut measurement = engine.start();

        thread::sleep(Duration::from_micros(100));
        measurement.checkpoint("checkpoint1");

        thread::sleep(Duration::from_micros(100));
        measurement.checkpoint("checkpoint2");

        let result = measurement.finish();

        assert!(result.total_duration.as_micros() >= 200);
        assert_eq!(result.checkpoints.len(), 2);
        assert!(result.get_checkpoint("checkpoint1").is_some());
        assert!(result.get_checkpoint("checkpoint2").is_some());
    }

    #[test]
    fn test_timing_precision() {
        let clock = Clock::new();
        let measurements: Vec<Duration> = (0..1000)
            .map(|_| {
                let start = clock.now();
                start.elapsed()
            })
            .collect();

        // Verify overhead is low (<5μs)
        let avg_overhead = measurements.iter().sum::<Duration>() / measurements.len() as u32;
        assert!(avg_overhead.as_micros() < 5);
    }
}

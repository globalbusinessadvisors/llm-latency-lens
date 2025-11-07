//! Statistical calculations for metrics

use serde::{Deserialize, Serialize};

/// Statistical summary of a set of values
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Statistics {
    /// Number of samples
    pub count: u64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Mean (average) value
    pub mean: f64,
    /// Median value (50th percentile)
    pub median: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
    /// Sum of all values
    pub sum: f64,
}

impl Statistics {
    /// Calculate statistics from a slice of values
    pub fn from_values(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self::default();
        }

        let count = values.len() as u64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let median = percentile(&sorted, 50.0);
        let p95 = percentile(&sorted, 95.0);
        let p99 = percentile(&sorted, 99.0);

        // Calculate standard deviation
        let variance: f64 = values.iter().map(|v| {
            let diff = v - mean;
            diff * diff
        }).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Self {
            count,
            min,
            max,
            mean,
            median,
            std_dev,
            p95,
            p99,
            sum,
        }
    }

    /// Check if statistics are empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Calculate a percentile from sorted values
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }

    let index = (p / 100.0) * (sorted_values.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    let weight = index - lower as f64;

    sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = Statistics::from_values(&values);

        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.sum, 15.0);
    }

    #[test]
    fn test_statistics_empty() {
        let values: Vec<f64> = vec![];
        let stats = Statistics::from_values(&values);

        assert_eq!(stats.count, 0);
        assert!(stats.is_empty());
    }

    #[test]
    fn test_percentiles() {
        let values: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        let stats = Statistics::from_values(&values);

        assert!((stats.p95 - 95.0).abs() < 1.0);
        assert!((stats.p99 - 99.0).abs() < 1.0);
    }

    #[test]
    fn test_standard_deviation() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let stats = Statistics::from_values(&values);

        // Known std dev for this dataset is approximately 2.0
        assert!((stats.std_dev - 2.0).abs() < 0.1);
    }
}

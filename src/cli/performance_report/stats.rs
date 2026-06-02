//! Runtime sample summary helpers.

use super::RuntimeDistribution;
use std::cmp::Ordering;

pub(super) fn distribution(samples: Vec<f64>) -> RuntimeDistribution {
    let min_ms = samples
        .iter()
        .copied()
        .min_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let max_ms = samples
        .iter()
        .copied()
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    RuntimeDistribution {
        samples: samples.len(),
        p95_ms: percentile(&samples, 0.95),
        samples_ms: samples,
        min_ms,
        max_ms,
    }
}

pub(super) fn median(samples: &[f64]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let midpoint = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[midpoint - 1] + sorted[midpoint]) / 2.0)
    } else {
        Some(sorted[midpoint])
    }
}

pub(super) fn percentile(samples: &[f64], percentile: f64) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let percentile = percentile.clamp(0.0, 1.0);
    let rank = ((sorted.len() as f64 * percentile).ceil() as usize).saturating_sub(1);
    sorted.get(rank).copied()
}

#[cfg(test)]
mod tests {
    use super::{distribution, median, percentile};
    use crate::cli::performance_report::RuntimeDistribution;

    #[test]
    fn median_handles_odd_and_even_samples() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median(&[4.0, 1.0, 2.0, 3.0]), Some(2.5));
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn distribution_records_bounds() {
        let RuntimeDistribution {
            samples,
            p95_ms,
            min_ms,
            max_ms,
            ..
        } = distribution(vec![3.0, 1.5, 4.0]);
        assert_eq!(samples, 3);
        assert_eq!(p95_ms, Some(4.0));
        assert_eq!(min_ms, Some(1.5));
        assert_eq!(max_ms, Some(4.0));
    }

    #[test]
    fn percentile_uses_nearest_rank() {
        assert_eq!(percentile(&[], 0.95), None);
        assert_eq!(percentile(&[1.0], 0.95), Some(1.0));
        assert_eq!(percentile(&[1.0, 2.0, 3.0, 4.0], 0.5), Some(2.0));
        assert_eq!(
            percentile(&(1..=30).map(f64::from).collect::<Vec<_>>(), 0.95),
            Some(29.0)
        );
    }
}

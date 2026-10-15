//! Scalar correctness oracle for oreslang-hpc-finance.
//!
//! The public Oreslang library should match these semantics, but it does not
//! need to mirror this Rust API.

#[derive(Debug, Clone, PartialEq)]
pub enum KernelError {
    OutputLength {
        expected: usize,
        actual: usize,
    },
    ColumnLength {
        column: &'static str,
        expected: usize,
        actual: usize,
    },
    InvalidWindow {
        window: usize,
        len: usize,
    },
    NonFinite {
        index: usize,
    },
    NonPositivePrice {
        index: usize,
    },
    NegativePrice {
        index: usize,
    },
    ZeroPreviousPrice {
        index: usize,
    },
    InsufficientSamples {
        required: usize,
        actual: usize,
    },
    InvalidAnnualization,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drawdown {
    pub peak_index: usize,
    pub trough_index: usize,
    /// Fractional loss from peak to trough, in [0, 1] for non-negative prices.
    pub fraction: f64,
}

/// Struct-of-arrays view for OHLCV bars.
///
/// Keeping columns independent lets kernels touch only the data they need and
/// maps cleanly to vectorized loads or device transfer later.
#[derive(Debug, Clone, Copy)]
pub struct BarColumns<'a> {
    pub ts_ns: &'a [i64],
    pub open: &'a [f64],
    pub high: &'a [f64],
    pub low: &'a [f64],
    pub close: &'a [f64],
    pub volume: &'a [f64],
}

impl<'a> BarColumns<'a> {
    pub fn len(&self) -> usize {
        self.ts_ns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ts_ns.is_empty()
    }

    pub fn validate_lengths(&self) -> Result<(), KernelError> {
        let expected = self.ts_ns.len();
        for (column, actual) in [
            ("open", self.open.len()),
            ("high", self.high.len()),
            ("low", self.low.len()),
            ("close", self.close.len()),
            ("volume", self.volume.len()),
        ] {
            if actual != expected {
                return Err(KernelError::ColumnLength {
                    column,
                    expected,
                    actual,
                });
            }
        }
        Ok(())
    }
}

fn expect_output(actual: usize, expected: usize) -> Result<(), KernelError> {
    if actual != expected {
        Err(KernelError::OutputLength { expected, actual })
    } else {
        Ok(())
    }
}

fn require_finite(values: &[f64]) -> Result<(), KernelError> {
    for (index, value) in values.iter().copied().enumerate() {
        if !value.is_finite() {
            return Err(KernelError::NonFinite { index });
        }
    }
    Ok(())
}

/// Writes N-1 simple returns for N prices into caller-provided storage.
pub fn simple_returns_into(prices: &[f64], out: &mut [f64]) -> Result<(), KernelError> {
    let expected = prices.len().saturating_sub(1);
    expect_output(out.len(), expected)?;
    require_finite(prices)?;

    // Validate every denominator before the first write so an error never
    // exposes a partially-updated caller buffer.
    for i in 1..prices.len() {
        if prices[i - 1] == 0.0 {
            return Err(KernelError::ZeroPreviousPrice { index: i - 1 });
        }
    }

    for i in 1..prices.len() {
        out[i - 1] = prices[i] / prices[i - 1] - 1.0;
    }
    Ok(())
}

/// Writes N-1 log returns for N strictly-positive prices.
pub fn log_returns_into(prices: &[f64], out: &mut [f64]) -> Result<(), KernelError> {
    let expected = prices.len().saturating_sub(1);
    expect_output(out.len(), expected)?;
    require_finite(prices)?;

    for (index, price) in prices.iter().copied().enumerate() {
        if price <= 0.0 {
            return Err(KernelError::NonPositivePrice { index });
        }
    }

    for i in 1..prices.len() {
        let ratio = prices[i] / prices[i - 1];
        // Finite positive prices can still overflow/underflow their ratio.
        // Keep direct ln near 1 for precision; log differences cover extremes.
        out[i - 1] = if ratio.is_finite() && ratio > 0.0 {
            ratio.ln()
        } else {
            prices[i].ln() - prices[i - 1].ln()
        };
    }
    Ok(())
}

/// Compact rolling mean: N values and window W produce N-W+1 outputs.
pub fn rolling_mean_into(
    values: &[f64],
    window: usize,
    out: &mut [f64],
) -> Result<(), KernelError> {
    if window == 0 || window > values.len() {
        return Err(KernelError::InvalidWindow {
            window,
            len: values.len(),
        });
    }
    expect_output(out.len(), values.len() - window + 1)?;
    require_finite(values)?;

    let mut sum = values[..window].iter().sum::<f64>();
    out[0] = sum / window as f64;

    for end in window..values.len() {
        sum += values[end];
        sum -= values[end - window];
        out[end - window + 1] = sum / window as f64;
    }
    Ok(())
}

/// Stable scalar mean using an online update.
pub fn mean(values: &[f64]) -> Result<f64, KernelError> {
    if values.is_empty() {
        return Err(KernelError::InsufficientSamples {
            required: 1,
            actual: 0,
        });
    }
    require_finite(values)?;

    let mut m = 0.0;
    for (i, x) in values.iter().copied().enumerate() {
        let n = (i + 1) as f64;
        m += (x - m) / n;
    }
    Ok(m)
}

/// Sample variance (denominator N-1) via Welford's algorithm.
pub fn sample_variance(values: &[f64]) -> Result<f64, KernelError> {
    if values.len() < 2 {
        return Err(KernelError::InsufficientSamples {
            required: 2,
            actual: values.len(),
        });
    }
    require_finite(values)?;

    let mut count = 0usize;
    let mut mean = 0.0;
    let mut m2 = 0.0;

    for x in values.iter().copied() {
        count += 1;
        let delta = x - mean;
        mean += delta / count as f64;
        let delta2 = x - mean;
        m2 += delta * delta2;
    }

    Ok(m2 / (count - 1) as f64)
}

/// Annualized arithmetic Sharpe ratio.
///
/// `risk_free_per_period` must already use the same period as `returns`.
/// Returns `Ok(None)` when excess-return volatility is exactly zero.
pub fn sharpe_ratio(
    returns: &[f64],
    risk_free_per_period: f64,
    periods_per_year: f64,
) -> Result<Option<f64>, KernelError> {
    if returns.len() < 2 {
        return Err(KernelError::InsufficientSamples {
            required: 2,
            actual: returns.len(),
        });
    }
    if !risk_free_per_period.is_finite() || !periods_per_year.is_finite() || periods_per_year <= 0.0
    {
        return Err(KernelError::InvalidAnnualization);
    }
    require_finite(returns)?;

    let mut mean_excess = 0.0;
    let mut m2 = 0.0;

    for (i, r) in returns.iter().copied().enumerate() {
        let x = r - risk_free_per_period;
        let n = (i + 1) as f64;
        let delta = x - mean_excess;
        mean_excess += delta / n;
        let delta2 = x - mean_excess;
        m2 += delta * delta2;
    }

    let variance = m2 / (returns.len() - 1) as f64;
    if variance == 0.0 {
        return Ok(None);
    }

    Ok(Some(
        mean_excess / variance.sqrt() * periods_per_year.sqrt(),
    ))
}

/// Maximum drawdown over non-negative finite prices.
///
/// Empty input returns `Ok(None)`.
pub fn max_drawdown(prices: &[f64]) -> Result<Option<Drawdown>, KernelError> {
    if prices.is_empty() {
        return Ok(None);
    }
    require_finite(prices)?;

    for (index, price) in prices.iter().copied().enumerate() {
        if price < 0.0 {
            return Err(KernelError::NegativePrice { index });
        }
    }

    let mut peak_price = prices[0];
    let mut peak_index = 0usize;
    let mut best = Drawdown {
        peak_index: 0,
        trough_index: 0,
        fraction: 0.0,
    };

    for (index, price) in prices.iter().copied().enumerate().skip(1) {
        if price > peak_price {
            peak_price = price;
            peak_index = index;
            continue;
        }

        // A zero running peak has no meaningful percentage drawdown until a
        // positive price establishes a denominator.
        if peak_price == 0.0 {
            continue;
        }

        let fraction = 1.0 - price / peak_price;
        if fraction > best.fraction {
            best = Drawdown {
                peak_index,
                trough_index: index,
                fraction,
            };
        }
    }

    Ok(Some(best))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() <= eps, "{a} != {b}");
    }

    #[test]
    fn bar_columns_validate_shape() {
        let bars = BarColumns {
            ts_ns: &[1, 2],
            open: &[10.0, 11.0],
            high: &[12.0, 13.0],
            low: &[9.0, 10.0],
            close: &[11.0, 12.0],
            volume: &[100.0, 120.0],
        };
        assert_eq!(bars.validate_lengths(), Ok(()));
        assert_eq!(bars.len(), 2);
    }

    #[test]
    fn simple_returns_are_compact_and_allocation_free() {
        let prices = [100.0, 110.0, 99.0];
        let mut out = [0.0; 2];
        simple_returns_into(&prices, &mut out).unwrap();
        approx_eq(out[0], 0.10, 1e-12);
        approx_eq(out[1], -0.10, 1e-12);
    }

    #[test]
    fn log_returns_reject_non_positive_prices() {
        let prices = [100.0, 0.0];
        let mut out = [0.0; 1];
        assert_eq!(
            log_returns_into(&prices, &mut out),
            Err(KernelError::NonPositivePrice { index: 1 })
        );
    }

    #[test]
    fn log_returns_remain_finite_across_full_positive_price_range() {
        let smallest = f64::from_bits(1);
        let largest = f64::MAX;
        let mut out = [0.0; 2];
        log_returns_into(&[smallest, largest, smallest], &mut out).unwrap();
        let expected = largest.ln() - smallest.ln();
        assert!(out.iter().all(|x| x.is_finite()));
        approx_eq(out[0], expected, 1e-12);
        approx_eq(out[1], -expected, 1e-12);
        log_returns_into(&[1.0, 1.0 + f64::EPSILON, 1.0], &mut out).unwrap();
        assert_eq!(out[0], (1.0 + f64::EPSILON).ln());
    }

    #[test]
    fn rolling_mean_uses_compact_output() {
        let values = [1.0, 2.0, 3.0, 4.0];
        let mut out = [0.0; 3];
        rolling_mean_into(&values, 2, &mut out).unwrap();
        assert_eq!(out, [1.5, 2.5, 3.5]);
    }

    #[test]
    fn sample_variance_uses_n_minus_one() {
        let values = [1.0, 2.0, 3.0];
        approx_eq(sample_variance(&values).unwrap(), 1.0, 1e-12);
    }

    #[test]
    fn max_drawdown_tracks_peak_and_trough() {
        let prices = [100.0, 120.0, 90.0, 95.0, 80.0, 130.0];
        let dd = max_drawdown(&prices).unwrap().unwrap();
        assert_eq!(dd.peak_index, 1);
        assert_eq!(dd.trough_index, 4);
        approx_eq(dd.fraction, 1.0 / 3.0, 1e-12);
    }

    #[test]
    fn sharpe_zero_volatility_has_no_finite_value() {
        let returns = [0.01, 0.01, 0.01];
        assert_eq!(sharpe_ratio(&returns, 0.0, 252.0).unwrap(), None);
    }

    #[test]
    fn rejects_column_length_mismatch() {
        let bars = BarColumns {
            ts_ns: &[1, 2],
            open: &[10.0],
            high: &[12.0, 13.0],
            low: &[9.0, 10.0],
            close: &[11.0, 12.0],
            volume: &[100.0, 120.0],
        };
        assert_eq!(
            bars.validate_lengths(),
            Err(KernelError::ColumnLength {
                column: "open",
                expected: 2,
                actual: 1,
            })
        );
    }

    #[test]
    fn simple_returns_reject_zero_previous_price() {
        let prices = [0.0, 1.0];
        let mut out = [0.0; 1];
        assert_eq!(
            simple_returns_into(&prices, &mut out),
            Err(KernelError::ZeroPreviousPrice { index: 0 })
        );
    }

    #[test]
    fn simple_returns_validation_is_fail_before_write() {
        let prices = [2.0, 4.0, 0.0, 1.0];
        let mut out = [7.0, 8.0, 9.0];
        assert_eq!(
            simple_returns_into(&prices, &mut out),
            Err(KernelError::ZeroPreviousPrice { index: 2 })
        );
        assert_eq!(out, [7.0, 8.0, 9.0]);
    }

    #[test]
    fn rolling_mean_rejects_invalid_window() {
        let values = [1.0, 2.0];
        let mut out = [0.0; 1];
        assert_eq!(
            rolling_mean_into(&values, 0, &mut out),
            Err(KernelError::InvalidWindow { window: 0, len: 2 })
        );
    }

    #[test]
    fn kernels_reject_non_finite_inputs() {
        let values = [1.0, f64::NAN];
        assert_eq!(mean(&values), Err(KernelError::NonFinite { index: 1 }));
    }

    #[test]
    fn max_drawdown_rejects_negative_prices() {
        let prices = [1.0, -0.5];
        assert_eq!(
            max_drawdown(&prices),
            Err(KernelError::NegativePrice { index: 1 })
        );
    }

    #[test]
    fn rejects_wrong_output_length() {
        let prices = [100.0, 101.0, 102.0];
        let mut out = [0.0; 1];
        assert_eq!(
            simple_returns_into(&prices, &mut out),
            Err(KernelError::OutputLength {
                expected: 2,
                actual: 1,
            })
        );
    }
}

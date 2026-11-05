//! Scalar Rust reference checks only; these do not establish AttoLang guest parity.
use oreslang_hpc_finance_reference::{
    simple_returns_into, log_returns_into, rolling_mean_into, max_drawdown,
    BarColumns, KernelError,
};

#[test]
fn errors_never_partially_write_simple_return_output() {
    let mut out = [13.0, 17.0, 19.0];
    assert_eq!(simple_returns_into(&[2.0, 4.0, 0.0, 1.0], &mut out),
        Err(KernelError::ZeroPreviousPrice { index: 2 }));
    assert_eq!(out, [13.0, 17.0, 19.0]);

    assert_eq!(simple_returns_into(&[2.0, f64::NAN, 3.0, 4.0], &mut out),
        Err(KernelError::NonFinite { index: 1 }));
    assert_eq!(out, [13.0, 17.0, 19.0]);
}

#[test]
fn rolling_window_validation_and_nonfinite_input_are_fail_before_write() {
    let mut out = [13.0, 17.0];
    assert_eq!(rolling_mean_into(&[1.0, 2.0, 3.0], 0, &mut out),
        Err(KernelError::InvalidWindow { window: 0, len: 3 }));
    assert_eq!(rolling_mean_into(&[1.0, f64::INFINITY, 3.0], 2, &mut out),
        Err(KernelError::NonFinite { index: 1 }));
    assert_eq!(out, [13.0, 17.0]);
    rolling_mean_into(&[1.0, 2.0, 3.0], 2, &mut out).unwrap();
    assert_eq!(out, [1.5, 2.5]);
}

#[test]
fn extreme_positive_log_return_remains_finite_and_signed() {
    let p = [f64::MIN_POSITIVE, f64::MAX, f64::MIN_POSITIVE];
    let mut out = [f64::NAN; 2];
    log_returns_into(&p, &mut out).unwrap();
    assert!(out.iter().all(|x| x.is_finite()));
    let expected = f64::MAX.ln() - f64::MIN_POSITIVE.ln();
    assert!((out[0] - expected).abs() < 1e-9);
    assert!((out[1] + expected).abs() < 1e-9);
}

#[test]
fn zero_initial_peak_does_not_produce_invalid_fraction() {
    let drawdown = max_drawdown(&[0.0, 0.0, 4.0, 2.0]).unwrap().unwrap();
    assert_eq!(drawdown.peak_index, 2);
    assert_eq!(drawdown.trough_index, 3);
    assert_eq!(drawdown.fraction, 0.5);
}

#[test]
fn each_ohlcv_column_mismatch_names_the_first_invalid_column() {
    let ts_ns = [10, 20];
    let valid = [1.0, 2.0];
    let short = [1.0];
    for target in ["open", "high", "low", "close", "volume"] {
        let bars = BarColumns {
            ts_ns: &ts_ns,
            open: if target == "open" { &short } else { &valid },
            high: if target == "high" { &short } else { &valid },
            low: if target == "low" { &short } else { &valid },
            close: if target == "close" { &short } else { &valid },
            volume: if target == "volume" { &short } else { &valid },
        };
        assert_eq!(bars.validate_lengths(),
            Err(KernelError::ColumnLength {
                column: target, expected: 2, actual: 1
            }));
    }
}

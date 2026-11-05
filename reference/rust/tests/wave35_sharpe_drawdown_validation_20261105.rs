//! Host scalar-oracle edge tests; not compiler-backed AttoLang parity evidence.
use oreslang_hpc_finance_reference::{
    log_returns_into, rolling_mean_into, sample_variance, sharpe_ratio, simple_returns_into,
    KernelError,
};

#[test]
fn return_buffers_remain_unchanged_on_wrong_length_and_nonpositive_price() {
    let mut simple_out = [41.0, 43.0, 47.0];
    assert_eq!(
        simple_returns_into(&[1.0, 2.0, 3.0], &mut simple_out),
        Err(KernelError::OutputLength {
            expected: 2,
            actual: 3,
        })
    );
    assert_eq!(simple_out, [41.0, 43.0, 47.0]);

    let mut log_out = [41.0, 43.0];
    assert_eq!(
        log_returns_into(&[1.0, 0.0, 3.0], &mut log_out),
        Err(KernelError::NonPositivePrice { index: 1 })
    );
    assert_eq!(log_out, [41.0, 43.0]);
}

#[test]
fn rolling_window_output_length_is_checked_before_any_write() {
    let mut out = [73.0];
    assert_eq!(
        rolling_mean_into(&[1.0, 2.0, 3.0], 2, &mut out),
        Err(KernelError::OutputLength {
            expected: 2,
            actual: 1,
        })
    );
    assert_eq!(out, [73.0]);
}

#[test]
fn sharpe_annualization_and_nonfinite_returns_are_rejected() {
    let returns = [0.1, 0.2];
    assert_eq!(
        sharpe_ratio(&returns, f64::NAN, 252.0),
        Err(KernelError::InvalidAnnualization)
    );
    assert_eq!(
        sharpe_ratio(&returns, 0.0, 0.0),
        Err(KernelError::InvalidAnnualization)
    );
    assert_eq!(
        sharpe_ratio(&returns, 0.0, f64::INFINITY),
        Err(KernelError::InvalidAnnualization)
    );
    assert_eq!(
        sharpe_ratio(&[0.1, f64::NAN], 0.0, 252.0),
        Err(KernelError::NonFinite { index: 1 })
    );
    assert!(sharpe_ratio(&returns, 0.0, 252.0)
        .unwrap()
        .unwrap()
        .is_finite());
}

#[test]
fn sample_variance_refuses_insufficient_samples_and_uses_sample_denominator() {
    assert_eq!(
        sample_variance(&[7.0]),
        Err(KernelError::InsufficientSamples {
            required: 2,
            actual: 1,
        })
    );
    let variance = sample_variance(&[1.0, 2.0, 3.0, 4.0]).unwrap();
    assert!((variance - 5.0 / 3.0).abs() < 1e-12);
}

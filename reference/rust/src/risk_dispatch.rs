//! HOST-ONLY reference for the final pre-trade decision and mock venue boundary.
//! This is not an Oreslang Region kernel, actor or authorized live venue path.
//! Recompute the risk decision at dispatch; never trust a caller-supplied approval.

use crate::{pretrade_risk_gate, RiskDecision};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LimitOrder {
    pub side: i64,
    pub quantity: f64,
    pub limit_price: f64,
}

/// Narrow, test-only capability: intentionally not a network or live exchange API.
pub trait MockVenue {
    fn submit_limit(&mut self, order: LimitOrder);
}

/// Re-evaluates risk immediately before invoking the mock capability.
/// The caller cannot pass an approved flag, reason code or override.
pub fn authorize_mock_limit<V: MockVenue>(
    order: LimitOrder,
    max_quantity: f64,
    venue: &mut V,
) -> RiskDecision {
    let decision = pretrade_risk_gate(
        order.side,
        order.quantity,
        order.limit_price,
        max_quantity,
    );
    if decision.approved && decision.reason_code == 0 {
        venue.submit_limit(order);
    }
    decision
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingMockVenue {
        calls: Vec<LimitOrder>,
    }

    impl MockVenue for RecordingMockVenue {
        fn submit_limit(&mut self, order: LimitOrder) {
            self.calls.push(order);
        }
    }

    fn parse_numeric(text: &str) -> f64 {
        match text {
            "NaN" => f64::NAN,
            "+Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            _ => text.parse().expect("fixture numeric field"),
        }
    }

    #[test]
    fn shared_23_case_inventory_has_no_rejected_venue_calls() {
        let mut venue = RecordingMockVenue::default();
        let mut accepted = 0usize;
        let mut rejected = 0usize;
        for row in include_str!("../../../fixtures/risk-gate-v1.tsv").lines() {
            if row.is_empty() || row.starts_with('#') {
                continue;
            }
            let columns: Vec<_> = row.split('|').collect();
            assert_eq!(columns.len(), 7);
            let order = LimitOrder {
                side: columns[1].parse().expect("side"),
                quantity: parse_numeric(columns[2]),
                limit_price: parse_numeric(columns[3]),
            };
            let before = venue.calls.len();
            let decision =
                authorize_mock_limit(order, parse_numeric(columns[4]), &mut venue);
            let expected_approved = columns[5].parse::<bool>().expect("approved");
            let expected_reason = columns[6].parse::<u8>().expect("reason");
            assert_eq!(decision.approved, expected_approved, "{}", columns[0]);
            assert_eq!(decision.reason_code, expected_reason, "{}", columns[0]);
            if expected_approved {
                accepted += 1;
                assert_eq!(venue.calls.len(), before + 1, "{}", columns[0]);
                assert_eq!(venue.calls[before], order);
            } else {
                rejected += 1;
                assert_eq!(venue.calls.len(), before, "{}", columns[0]);
            }
        }
        assert_eq!((accepted, rejected), (3, 20));
        assert_eq!(venue.calls.len(), 3);
    }

    #[test]
    fn malformed_and_nonfinite_inputs_never_reach_mock_venue() {
        let mut venue = RecordingMockVenue::default();
        for side in [-2, -1, 0, 1, 2] {
            for quantity in [-1.0, -0.0, 0.0, 1.0, 3.0, f64::NAN] {
                for limit_price in [-1.0, 0.0, 10.0, f64::INFINITY] {
                    for maximum in [-1.0, 0.0, 2.0, f64::NAN] {
                        let order = LimitOrder {
                            side,
                            quantity,
                            limit_price,
                        };
                        let before = venue.calls.len();
                        let decision = authorize_mock_limit(order, maximum, &mut venue);
                        let submitted = venue.calls.len() != before;
                        assert_eq!(submitted, decision.approved && decision.reason_code == 0);
                        if submitted {
                            assert_eq!(venue.calls.len(), before + 1);
                        } else {
                            assert_eq!(venue.calls.len(), before);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn risk_is_recomputed_from_order_not_from_caller_approval() {
        let mut venue = RecordingMockVenue::default();
        let forged_approval = RiskDecision {
            approved: true,
            reason_code: 0,
        };
        assert!(forged_approval.approved);
        let rejected = authorize_mock_limit(
            LimitOrder {
                side: 2,
                quantity: 1.0,
                limit_price: 10.0,
            },
            2.0,
            &mut venue,
        );
        assert_eq!(rejected.reason_code, 5);
        assert!(!rejected.approved);
        assert!(venue.calls.is_empty());
    }
}

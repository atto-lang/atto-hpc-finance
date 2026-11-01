//! Source inventory checks only. These do NOT execute any Oreslang guest.
//! The separately pinned compiler must execute the exact Oreslang source files.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    const NEW_GUESTS: [(&str, &str, usize); 4] = [
        (
            "batch_shape_adversarial",
            include_str!("../../../tests/batch_shape_adversarial.ores"),
            37,
        ),
        (
            "scalar_exact_values",
            include_str!("../../../tests/scalar_exact_values.ores"),
            24,
        ),
        (
            "scalar_failure_atomicity_matrix",
            include_str!("../../../tests/scalar_failure_atomicity_matrix.ores"),
            19,
        ),
        (
            "risk_precedence_matrix",
            include_str!("../../../tests/risk_precedence_matrix.ores"),
            34,
        ),
    ];

    fn cases(source: &'static str) -> Vec<&'static str> {
        source
            .lines()
            .filter_map(|line| line.trim().strip_prefix("// case:"))
            .map(|name| name.split(';').next().expect("nonempty case marker"))
            .collect()
    }

    #[test]
    fn new_oreslang_test_case_inventories_are_complete_and_unique() {
        let mut global_names = BTreeSet::new();
        for (name, source, count) in NEW_GUESTS {
            let ids = cases(source);
            assert_eq!(ids.len(), count, "missing guest cases in {name}");
            for id in ids {
                assert!(!id.is_empty(), "empty case name in {name}");
                assert!(
                    global_names.insert(format!("{name}/{id}")),
                    "duplicate case name {name}/{id}"
                );
            }
            assert!(source.contains("pub routine main() -> void"));
            assert!(source.contains("stdio.stdout.write("));
        }
        assert_eq!(global_names.len(), 114);
    }

    #[test]
    fn exact_head_guest_gate_and_mock_agree_on_new_oreslang_programs() {
        let runner = include_str!("../../../scripts/run-scalar-guest-exact-head.sh");
        let mock_only = include_str!("../../../scripts/test-scalar-guest-gate.sh");
        for (name, _, assertions) in NEW_GUESTS {
            let expected = std::iter::repeat("true")
                .take(assertions)
                .collect::<Vec<_>>()
                .join(":");
            assert!(
                runner.contains(format!("{name}) expected='{expected}'").as_str()),
                "exact-head runner has wrong oracle: {name}"
            );
            assert!(
                mock_only.contains(format!("*/{name}.ores) printf '{expected}'").as_str()),
                "mock-only test has wrong expected output: {name}"
            );
            assert!(
                mock_only.contains(format!("HPC_GUEST_CASE=PASS {name}").as_str()),
                "mock-only test is missing receipt assertion: {name}"
            );
        }
    }

    #[test]
    fn new_guest_groups_cover_both_rejections_and_valid_numeric_values() {
        let batch = cases(NEW_GUESTS[0].1);
        let numeric = cases(NEW_GUESTS[1].1);
        let atomic = cases(NEW_GUESTS[2].1);
        let risk = cases(NEW_GUESTS[3].1);
        assert!(batch.contains(&"minimum_nonempty"));
        assert!(batch.contains(&"huge_valid_shape_no_buffers_allocated"));
        assert!(batch.contains(&"zero_decision_capacity"));
        assert!(numeric.contains(&"rolling_alternating"));
        assert!(numeric.contains(&"variance_signed"));
        assert!(atomic.contains(&"late_zero_denominator"));
        assert!(atomic.contains(&"rolling_late_nan"));
        assert!(risk.contains(&"negative_max_overrides_no_side"));
        assert!(risk.contains(&"nan_quantity_overrides_zero_side"));
    }
}

// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use pretty_assertions::assert_eq;
use quick_junit::{DeserializeErrorKind, PathElement, Report, TestCase, TestCaseStatus, TestSuite};

#[test]
fn skipped_counts_aggregate_at_suite_and_report_level() {
    let mut report = Report::new("skip-run");

    let mut suite1 = TestSuite::new("suite1");
    suite1.add_test_case(TestCase::new("passes", TestCaseStatus::success()));
    suite1.add_test_case(TestCase::new("skips1", TestCaseStatus::skipped()));
    report.add_test_suite(suite1);

    let mut suite2 = TestSuite::new("suite2");
    suite2.add_test_case(TestCase::new("skips2", TestCaseStatus::skipped()));
    suite2.add_test_case(TestCase::new("skips3", TestCaseStatus::skipped()));
    report.add_test_suite(suite2);

    assert_eq!(report.test_suites[0].skipped, 1, "suite1 skip count");
    assert_eq!(report.test_suites[1].skipped, 2, "suite2 skip count");
    assert_eq!(
        report.skipped, 3,
        "report aggregates skip counts across suites"
    );
    assert_eq!(
        report.tests, 4,
        "report aggregates test counts across suites"
    );

    // add_test_case alters skipped but not disabled, so disabled stays None
    // everywhere.
    assert_eq!(
        report.disabled, None,
        "disabled is never populated from test cases"
    );
    assert_eq!(
        report.test_suites[0].disabled, None,
        "suite1 disabled is unset"
    );
    assert_eq!(
        report.test_suites[1].disabled, None,
        "suite2 disabled is unset"
    );

    let xml = report.to_string().expect("serialization succeeds");

    assert!(
        xml.contains(r#"<testsuites name="skip-run" tests="4" skipped="3""#),
        "root element carries aggregated skip count, got:\n{xml}"
    );
    assert!(
        xml.contains(r#"<testsuite name="suite1" tests="2" skipped="1""#),
        "suite1 carries its own skip count, got:\n{xml}"
    );
    assert!(
        xml.contains(r#"<testsuite name="suite2" tests="2" skipped="2""#),
        "suite2 carries its own skip count, got:\n{xml}"
    );
    assert!(
        !xml.contains("disabled="),
        "the `disabled` attribute is omitted when the field is None, got:\n{xml}"
    );

    let deserialized = Report::deserialize_from_str(&xml).expect("deserialization succeeds");
    assert_eq!(
        deserialized, report,
        "report round-trips through serialization"
    );
    assert_eq!(
        deserialized.skipped, 3,
        "root skip count survives round-trip"
    );
    assert_eq!(
        deserialized.test_suites[1].skipped, 2,
        "suite skip count survives round-trip"
    );
}

// Older versions of quick-junit wrote `disabled` on `<testsuite>` as their
// skip count, and googletest writes `disabled` on both `<testsuites>` and
// `<testsuite>` for tests disabled by design. Both are now parsed into the
// separate `disabled` field, leaving `skipped` at zero.
#[test]
fn deserialize_disabled_preserved_as_separate_count() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="legacy" tests="2" failures="0" errors="0" disabled="2">
    <testsuite name="suite" tests="2" disabled="2" errors="0" failures="0">
        <testcase name="a"><skipped/></testcase>
        <testcase name="b"><skipped/></testcase>
    </testsuite>
</testsuites>
"#;

    let report = Report::deserialize_from_str(xml).expect("report with disabled counts parses");
    assert_eq!(report.skipped, 0, "root `disabled` does not feed `skipped`");
    assert_eq!(
        report.disabled,
        Some(2),
        "root `disabled` is preserved as its own count"
    );
    assert_eq!(
        report.test_suites[0].skipped, 0,
        "suite `disabled` does not feed `skipped`"
    );
    assert_eq!(
        report.test_suites[0].disabled,
        Some(2),
        "suite `disabled` is preserved as its own count"
    );
}

#[test]
fn deserialize_skipped_and_disabled_are_independent() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="both" tests="2" failures="0" errors="0" skipped="2" disabled="5">
    <testsuite name="suite" tests="2" skipped="2" disabled="5" errors="0" failures="0">
        <testcase name="a"><skipped/></testcase>
        <testcase name="b"><skipped/></testcase>
    </testsuite>
</testsuites>
"#;

    let report = Report::deserialize_from_str(xml).expect("report parses");
    assert_eq!(report.skipped, 2, "root skipped is its own count");
    assert_eq!(report.disabled, Some(5), "root disabled is its own count");
    assert_eq!(
        report.test_suites[0].skipped, 2,
        "suite skipped is its own count"
    );
    assert_eq!(
        report.test_suites[0].disabled,
        Some(5),
        "suite disabled is its own count"
    );

    let xml2 = report.to_string().expect("serialization succeeds");
    let report2 = Report::deserialize_from_str(&xml2).expect("re-parse succeeds");
    assert_eq!(
        report2, report,
        "both skipped and disabled survive a serialize/deserialize round-trip"
    );
    assert!(
        xml2.contains(r#"skipped="2""#) && xml2.contains(r#"disabled="5""#),
        "serialized output carries both counts, got:\n{xml2}"
    );
}

#[test]
fn deserialize_invalid_suite_skipped_count() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="test" tests="1" failures="0" errors="0">
    <testsuite name="suite" tests="1" skipped="invalid" errors="0" failures="0">
    </testsuite>
</testsuites>"#;

    match Report::deserialize_from_str(xml) {
        Ok(_) => panic!("expected a parse error for malformed skipped count"),
        Err(e) => {
            assert!(
                matches!(e.kind(), DeserializeErrorKind::ParseIntError(_)),
                "expected a ParseIntError, got: {:?}",
                e.kind()
            );
            assert_eq!(
                e.path(),
                &[
                    PathElement::TestSuites,
                    PathElement::TestSuite(0, None),
                    PathElement::Attribute("skipped".to_string()),
                ],
                "error path points at the suite's skipped attribute"
            );
        }
    }
}

#[test]
fn add_test_suite_aggregates_disabled_counts() {
    let mut report = Report::new("agg");

    let mut suite_a = TestSuite::new("a");
    suite_a.disabled = Some(2);
    report.add_test_suite(suite_a);
    assert_eq!(
        report.disabled,
        Some(2),
        "first disabled count initializes the aggregate"
    );

    report.add_test_suite(TestSuite::new("b"));
    assert_eq!(
        report.disabled,
        Some(2),
        "a suite without a disabled count leaves the aggregate untouched"
    );

    let mut suite_c = TestSuite::new("c");
    suite_c.disabled = Some(3);
    report.add_test_suite(suite_c);
    assert_eq!(
        report.disabled,
        Some(5),
        "present disabled counts sum across suites"
    );

    let mut all_none = Report::new("all-none");
    all_none.add_test_suite(TestSuite::new("x"));
    all_none.add_test_suite(TestSuite::new("y"));
    assert_eq!(
        all_none.disabled, None,
        "a report built from suites without disabled counts stays None"
    );
}

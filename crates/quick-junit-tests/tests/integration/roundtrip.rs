// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Property-based roundtrip tests for quick-junit serialization and deserialization.
//!
//! These tests verify that:
//! 1. Serialization always succeeds for valid structures
//! 2. Deserialization succeeds for serialized output
//! 3. Deserializing serialized data produces equivalent structures
//! 4. Summary counts (tests, failures, errors) are consistent with actual test cases

use quick_junit::*;
use test_strategy::proptest;

/// Tests that Report roundtrips correctly through serialization and deserialization.
///
/// This verifies:
/// - Report can be serialized to XML
/// - The XML can be deserialized back to a Report
/// - The deserialized Report matches the original (modulo known precision limitations)
#[proptest(cases = 32)]
fn report_roundtrip(report: Report) {
    let xml = report.to_string().expect("serialization should succeed");

    let deserialized = Report::deserialize_from_str(&xml).expect("deserialization should succeed");

    assert_eq!(report, deserialized);
}

/// Tests that TestSuite roundtrips correctly.
#[proptest(cases = 64)]
fn test_suite_roundtrip(test_suite: TestSuite) {
    // Create a minimal report to hold the test suite.
    let mut report = Report::new("test-report");
    report.add_test_suite(test_suite.clone());

    let xml = report.to_string().expect("serialization should succeed");
    let deserialized_report =
        Report::deserialize_from_str(&xml).expect("deserialization should succeed");

    let deserialized = &deserialized_report.test_suites[0];
    assert_eq!(&test_suite, deserialized, "test suite should match");
}

/// Tests that TestCase roundtrips correctly.
#[proptest]
fn test_case_roundtrip(test_case: TestCase) {
    // Create a minimal report to hold the test case.
    let mut report = Report::new("test-report");
    let mut suite = TestSuite::new("test-suite");
    suite.add_test_case(test_case.clone());
    report.add_test_suite(suite);

    let xml = report.to_string().expect("serialization should succeed");
    let deserialized_report =
        Report::deserialize_from_str(&xml).expect("deserialization should succeed");

    let deserialized = &deserialized_report.test_suites[0].test_cases[0];
    assert_eq!(&test_case, deserialized, "test case should match");
}

/// Tests that Property roundtrips correctly.
#[proptest]
fn property_roundtrip(property: Property) {
    // Create a minimal report with the property
    let mut report = Report::new("test-report");
    let mut suite = TestSuite::new("test-suite");
    suite.add_property(property.clone());
    report.add_test_suite(suite);

    // Serialize and deserialize
    let xml = report.to_string().expect("serialization should succeed");
    let deserialized_report =
        Report::deserialize_from_str(&xml).expect("deserialization should succeed");

    let deserialized = &deserialized_report.test_suites[0].properties[0];
    assert_eq!(&property, deserialized, "property should match");
}

/// Tests that XmlString filters invalid characters correctly.
#[proptest]
fn xml_string_filters_invalid_chars(s: String) {
    let xml_string = XmlString::new(&s);

    // The XmlString should not contain any control characters (except valid
    // ones).
    for ch in xml_string.as_str().chars() {
        assert!(
            !matches!(ch, '\x00'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x1f'),
            "XmlString should filter control characters, but found {:?}",
            ch
        );
    }

    // If the original string was valid, it should be preserved.
    let has_invalid = s.chars().any(|c| {
        matches!(c, '\x00'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x1f') || s.contains("\x1b[")
        // ANSI escape
    });

    if !has_invalid {
        assert_eq!(xml_string.as_str(), s, "Valid strings should be preserved");
    }
}

/// Tests that NonSuccessKind roundtrips correctly.
#[proptest]
fn non_success_kind_roundtrip(kind: NonSuccessKind) {
    // Create a test case with this kind.
    let status = TestCaseStatus::non_success(kind);
    let test_case = TestCase::new("test", status.clone());

    // Create a minimal report.
    let mut report = Report::new("test-report");
    let mut suite = TestSuite::new("test-suite");
    suite.add_test_case(test_case);
    report.add_test_suite(suite);

    let xml = report.to_string().expect("serialization should succeed");
    let deserialized_report =
        Report::deserialize_from_str(&xml).expect("deserialization should succeed");

    let deserialized_status = &deserialized_report.test_suites[0].test_cases[0].status;
    assert_eq!(&status, deserialized_status);
}

/// Tests that TestRerun roundtrips correctly.
#[proptest]
fn test_rerun_roundtrip(rerun: TestRerun) {
    // Create a test case with a flaky run
    let test_case = TestCase::new(
        "test",
        TestCaseStatus::Success {
            flaky_runs: vec![rerun.clone()],
        },
    );

    // Create a minimal report
    let mut report = Report::new("test-report");
    let mut suite = TestSuite::new("test-suite");
    suite.add_test_case(test_case);
    report.add_test_suite(suite);

    // Serialize and deserialize
    let xml = report.to_string().expect("serialization should succeed");
    let deserialized_report =
        Report::deserialize_from_str(&xml).expect("deserialization should succeed");

    let deserialized_status = &deserialized_report.test_suites[0].test_cases[0].status;

    if let TestCaseStatus::Success { flaky_runs } = deserialized_status {
        assert_eq!(flaky_runs.len(), 1, "should have one flaky run");
        let deserialized_rerun = &flaky_runs[0];
        assert_eq!(&rerun, deserialized_rerun);
    } else {
        panic!("Expected Success status with flaky runs");
    }
}

/// Tests the serialized form of timestamps: a numeric UTC offset (never `Z`), and fractional
/// seconds truncated to 3 decimal places. Deserializing keeps the offset.
#[test]
fn timestamp_wire_format() {
    use jiff::{
        tz::{self, TimeZone},
        Timestamp,
    };

    let instant = Timestamp::new(1_617_303_157, 999_999_999).unwrap();
    for (offset, expected) in [
        (tz::offset(0), "2021-04-01T18:52:37.999+00:00"),
        (tz::offset(-8), "2021-04-01T10:52:37.999-08:00"),
        (
            tz::Offset::from_seconds(5 * 3600 + 45 * 60).unwrap(),
            "2021-04-02T00:37:37.999+05:45",
        ),
    ] {
        let mut report = Report::new("timestamps");
        report.set_timestamp(instant.to_zoned_datetime(TimeZone::fixed(offset)));
        let xml = report.to_string().expect("serialization should succeed");
        assert!(
            xml.contains(&format!(r#"timestamp="{expected}""#)),
            "expected timestamp {expected} in:\n{xml}"
        );

        let deserialized =
            Report::deserialize_from_str(&xml).expect("deserialization should succeed");
        let timestamp = deserialized.timestamp.expect("timestamp is present");
        assert_eq!(timestamp.offset(), offset);
        assert_eq!(
            timestamp.timestamp(),
            Timestamp::new(1_617_303_157, 999_000_000).unwrap()
        );
    }
}

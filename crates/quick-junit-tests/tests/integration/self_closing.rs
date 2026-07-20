// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tests to ensure that testsuite and testcase tags are serialized in the
//! self-closing form if and only if they truly have no children.

use pretty_assertions::assert_eq;
use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestRerun, TestSuite};

#[test]
fn childless_test_suite_serializes_self_closing() {
    let mut suite = TestSuite::new("empty-suite");
    suite.tests = 3;
    suite.disabled = 1;

    let mut report = Report::new("run");
    report.add_test_suite(suite.clone());

    let xml = report.to_string().expect("report serializes");

    assert!(
        xml.contains(
            r#"<testsuite name="empty-suite" tests="3" disabled="1" errors="0" failures="0"/>"#
        ),
        "expected self-closing testsuite, got:\n{xml}"
    );
    assert!(
        !xml.contains("</testsuite>"),
        "childless suite should have no closing tag, got:\n{xml}"
    );

    let deserialized = Report::deserialize_from_str(&xml).expect("report round-trips");
    assert_eq!(
        deserialized.test_suites[0], suite,
        "suite round-trips to an equal value"
    );
}

#[test]
fn childless_success_test_case_serializes_self_closing() {
    let test_case = TestCase::new("solo", TestCaseStatus::success());

    let mut suite = TestSuite::new("suite");
    suite.add_test_case(test_case.clone());
    let mut report = Report::new("run");
    report.add_test_suite(suite);

    let xml = report.to_string().expect("report serializes");

    assert!(
        xml.contains(r#"<testcase name="solo"/>"#),
        "expected self-closing testcase, got:\n{xml}"
    );
    assert!(
        !xml.contains("</testcase>"),
        "childless case should have no closing tag, got:\n{xml}"
    );

    let deserialized = Report::deserialize_from_str(&xml).expect("report round-trips");
    assert_eq!(
        deserialized.test_suites[0].test_cases[0], test_case,
        "case round-trips to an equal value"
    );
}

#[test]
fn suite_with_only_system_out_stays_expanded() {
    // system-out is a child, so the suite must not collapse to self-closing.
    let mut suite = TestSuite::new("suite");
    suite.set_system_out("suite output");
    let mut report = Report::new("run");
    report.add_test_suite(suite);

    let xml = report.to_string().expect("report serializes");

    assert!(
        xml.contains("<system-out>suite output</system-out>"),
        "expected system-out child, got:\n{xml}"
    );
    assert!(
        xml.contains("</testsuite>"),
        "suite with system-out should stay expanded, got:\n{xml}"
    );
}

#[test]
fn skipped_test_case_stays_expanded() {
    // A skipped case always emits a <skipped/> child, so it should not be
    // serialized as a self-closing tag.
    let test_case = TestCase::new("skip", TestCaseStatus::skipped());

    let mut suite = TestSuite::new("suite");
    suite.add_test_case(test_case);
    let mut report = Report::new("run");
    report.add_test_suite(suite);

    let xml = report.to_string().expect("report serializes");

    assert!(
        xml.contains("<skipped/>"),
        "expected <skipped/> child, got:\n{xml}"
    );
    assert!(
        xml.contains("</testcase>"),
        "skipped case has a child element so stays expanded, got:\n{xml}"
    );
}

#[test]
fn success_test_case_with_flaky_run_stays_expanded() {
    // A flaky run is a child element, so a success case with a flaky run should
    // not be serialized as a self-closing tag.
    let mut status = TestCaseStatus::success();
    status.add_rerun(TestRerun::new(NonSuccessKind::Failure));
    let test_case = TestCase::new("flaky", status);

    let mut suite = TestSuite::new("suite");
    suite.add_test_case(test_case);
    let mut report = Report::new("run");
    report.add_test_suite(suite);

    let xml = report.to_string().expect("report serializes");

    assert!(
        xml.contains("<flakyFailure"),
        "expected <flakyFailure> child, got:\n{xml}"
    );
    assert!(
        xml.contains("</testcase>"),
        "success case with a flaky run stays expanded, got:\n{xml}"
    );
}

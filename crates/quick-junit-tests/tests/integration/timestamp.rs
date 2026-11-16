// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use chrono::{DateTime, FixedOffset, TimeZone};
use proptest::prelude::*;
use quick_junit::{DeserializeErrorKind, Report};
use test_strategy::proptest;

fn timestamp(secs: i64, nanos: u32, offset_secs: i32) -> DateTime<FixedOffset> {
    FixedOffset::east_opt(offset_secs)
        .expect("offset is in range")
        .timestamp_opt(secs, nanos)
        .single()
        .expect("timestamp is in range")
}

fn serialized_timestamp(report: &Report) -> String {
    let xml = report.to_string().expect("report serialized");
    let start = xml
        .find("timestamp=\"")
        .expect("timestamp attribute present")
        + "timestamp=\"".len();
    let len = xml[start..].find('"').expect("timestamp attribute closed");
    xml[start..start + len].to_owned()
}

fn reserialize(input: &str) -> Option<String> {
    let xml =
        format!(r#"<testsuites name="r" tests="0" failures="0" errors="0" timestamp="{input}"/>"#);
    match Report::deserialize_from_str(&xml) {
        Ok(report) => Some(serialized_timestamp(&report)),
        Err(error) => {
            let DeserializeErrorKind::ParseTimestampError(_) = error.kind() else {
                panic!("for input {input:?}, expected a timestamp parse error, got {error}");
            };
            None
        }
    }
}

#[test]
fn timestamp_serialize() {
    let cases: &[(i64, u32, i32, &str)] = &[
        (1617299557, 0, -8 * 3600, "2021-04-01T09:52:37.000-08:00"),
        (1617299557, 0, 0, "2021-04-01T17:52:37.000+00:00"),
        (1617299557, 123_999_999, 0, "2021-04-01T17:52:37.123+00:00"),
        (
            1617299557,
            999_999_999,
            3600,
            "2021-04-01T18:52:37.999+01:00",
        ),
        (1617299557, 1, 19800, "2021-04-01T23:22:37.000+05:30"),
        (1617299557, 0, 19815, "2021-04-01T23:22:52.000+05:30"),
        (1617299557, 0, -1, "2021-04-01T17:52:36.000-00:00"),
        (-62167219200, 0, 0, "0000-01-01T00:00:00.000+00:00"),
    ];

    for &(secs, nanos, offset_secs, expected) in cases {
        let mut report = Report::new("r");
        report.set_timestamp(timestamp(secs, nanos, offset_secs));
        assert_eq!(
            serialized_timestamp(&report),
            expected,
            "for secs={secs}, nanos={nanos}, offset_secs={offset_secs}"
        );
    }
}

#[test]
fn timestamp_deserialize() {
    let cases: &[(&str, Option<&str>)] = &[
        (
            "2021-04-01T10:52:37-08:00",
            Some("2021-04-01T10:52:37.000-08:00"),
        ),
        (
            "2021-04-01T10:52:37Z",
            Some("2021-04-01T10:52:37.000+00:00"),
        ),
        (
            "2021-04-01t10:52:37z",
            Some("2021-04-01T10:52:37.000+00:00"),
        ),
        (
            "2021-04-01 10:52:37Z",
            Some("2021-04-01T10:52:37.000+00:00"),
        ),
        (
            "2021-04-01T10:52:37+00:00",
            Some("2021-04-01T10:52:37.000+00:00"),
        ),
        (
            "2021-04-01T10:52:37-00:00",
            Some("2021-04-01T10:52:37.000+00:00"),
        ),
        (
            "2021-04-01T10:52:37+23:59",
            Some("2021-04-01T10:52:37.000+23:59"),
        ),
        (
            "2021-04-01T10:52:37.1Z",
            Some("2021-04-01T10:52:37.100+00:00"),
        ),
        (
            "2021-04-01T10:52:37.9999Z",
            Some("2021-04-01T10:52:37.999+00:00"),
        ),
        (
            "2021-04-01T10:52:37.123456789Z",
            Some("2021-04-01T10:52:37.123+00:00"),
        ),
        (
            "2021-04-01T10:52:37.123456789012Z",
            Some("2021-04-01T10:52:37.123+00:00"),
        ),
        (
            "2021-04-01T10:52:37.999999999999-08:00",
            Some("2021-04-01T10:52:37.999-08:00"),
        ),
        (
            "2016-12-31T23:59:60Z",
            Some("2016-12-31T23:59:60.000+00:00"),
        ),
        (
            "2016-12-31T23:59:60.5Z",
            Some("2016-12-31T23:59:60.500+00:00"),
        ),
        (
            "0000-01-01T00:00:00Z",
            Some("0000-01-01T00:00:00.000+00:00"),
        ),
        (
            "9999-12-31T23:59:59Z",
            Some("9999-12-31T23:59:59.000+00:00"),
        ),
        ("2021-04-01T10:52:37,123Z", None),
        ("2021-04-01T10:52:37-0800", None),
        ("2021-04-01T10:52:37-08", None),
        ("2021-04-01T10:52:37+24:00", None),
        ("2021-04-01T10:52:37+05:30:15", None),
        ("2021-04-01T10:52Z", None),
        ("2021-04-01T10Z", None),
        ("20210401T105237Z", None),
        ("+002021-04-01T10:52:37Z", None),
        ("2021-04-01T10:52:37Z[UTC]", None),
        ("2021-04-01T10:52:37-08:00[America/Los_Angeles]", None),
        ("2021-04-01", None),
        ("2021-04-01T10:52:37", None),
        ("2021-04-01T10:52:37.Z", None),
        (" 2021-04-01T10:52:37Z", None),
        ("2021-04-01T10:52:37Z ", None),
        ("2021-04-01T10:52:37 Z", None),
        ("2021-02-29T10:52:37Z", None),
        ("2021-04-01T24:00:00Z", None),
        ("", None),
    ];

    for &(input, expected) in cases {
        assert_eq!(
            reserialize(input).as_deref(),
            expected,
            "for input {input:?}"
        );
    }
}

fn canonical_timestamp_strategy() -> impl Strategy<Value = String> {
    (
        (1u32..=9998, 1u32..=12, 1u32..=28),
        (0u32..=23, 0u32..=59, 0u32..=59, 0u32..=999),
        (any::<bool>(), 0u32..=23, 0u32..=59),
    )
        .prop_map(
            |((year, month, day), (hour, minute, second, millis), (negative, oh, om))| {
                let sign = if negative && (oh, om) != (0, 0) { '-' } else { '+' };
                format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}{sign}{oh:02}:{om:02}"
                )
            },
        )
}

#[proptest]
fn timestamp_offset_roundtrip(#[strategy(canonical_timestamp_strategy())] input: String) {
    prop_assert_eq!(reserialize(&input), Some(input));
}

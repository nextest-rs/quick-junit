#![no_main]

use libfuzzer_sys::fuzz_target;
use quick_junit::Report;

fuzz_target!(|data: &[u8]| {
    // Try to interpret the data as a UTF-8 string.
    let input = match std::str::from_utf8(data) {
        Ok(data) => data,
        Err(_) => return,
    };

    // Check that the deserializer doesn't panic on any input; both success and
    // error paths should be covered.
    _ = Report::deserialize_from_str(input);
});

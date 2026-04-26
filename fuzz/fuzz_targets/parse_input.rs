#![no_main]

use libfuzzer_sys::fuzz_target;

// Smoke-test the parser: any UTF-8 input must return Ok(_) or a typed
// ParseError without panicking.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = rustycli::parser::parse_pipelines(s);
    }
});

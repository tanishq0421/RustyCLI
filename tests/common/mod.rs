//! Shared helpers for integration tests. Imported via `mod common;`.

use assert_cmd::Command;

/// Build a `Command` for the rustycli binary with `--norc` so tests don't
/// pick up the developer's real `~/.rustyrc`.
pub fn rusty() -> Command {
    let mut c = Command::cargo_bin("rustycli").unwrap();
    c.arg("--norc");
    c
}

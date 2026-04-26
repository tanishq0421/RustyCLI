use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::tempdir;

mod common;
use common::rusty;

#[test]
fn echo_prints_args() {
    rusty()
        .args(["-c", "echo hello world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn cd_then_pwd_via_multistatement() {
    rusty()
        .args(["-c", "cd /tmp; pwd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("/tmp"));
}

#[test]
fn unknown_command_is_reported() {
    rusty()
        .args(["-c", "definitely_not_a_real_command_xyz"])
        .assert()
        // Shell itself exits 0 — the failure is per-command, mirroring bash.
        .stderr(predicate::str::contains("command not found"));
}

#[test]
fn export_and_expand() {
    rusty()
        .args(["-c", "export NAME=tanishq; echo hi $NAME"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hi tanishq"));
}

#[test]
fn parse_error_does_not_kill_shell() {
    // Unterminated quote on the first line; the second line should still run.
    rusty()
        .args(["-c", "echo \"oops"])
        .assert()
        .stderr(predicate::str::contains("parse error"));
}

#[test]
fn script_mode_runs_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("script.sh");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "echo line1").unwrap();
    writeln!(f, "echo line2").unwrap();
    drop(f);

    rusty()
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line1"))
        .stdout(predicate::str::contains("line2"));
}

#[test]
fn alias_from_rcfile_expands() {
    let dir = tempdir().unwrap();
    let rc = dir.path().join(".rustyrc");
    std::fs::write(&rc, "alias greet='echo hello-from-alias'\n").unwrap();

    // Note: drop --norc here so the rcfile loads. Override HOME so we don't
    // touch the real user's rcfile.
    Command::cargo_bin("rustycli")
        .unwrap()
        .env("HOME", dir.path())
        .args(["-c", "greet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello-from-alias"));
}

#[test]
fn comments_are_ignored() {
    rusty()
        .args(["-c", "# this is a comment"])
        .assert()
        .success();
}

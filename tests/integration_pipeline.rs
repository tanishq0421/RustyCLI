use predicates::prelude::*;
use std::io::Write;
use tempfile::tempdir;

mod common;
use common::rusty;

#[test]
fn pipeline_two_stages() {
    rusty()
        .args(["-c", "echo hello world | wc -w"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn pipeline_three_stages() {
    rusty()
        .args(["-c", "printf 'a\\nb\\nc\\n' | grep b | wc -l"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn redirect_out_creates_file() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.txt");
    rusty()
        .args(["-c", &format!("echo first > {}", out.display())])
        .assert()
        .success();
    let body = std::fs::read_to_string(&out).unwrap();
    assert_eq!(body.trim(), "first");
}

#[test]
fn redirect_append_preserves_existing() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("log.txt");
    std::fs::write(&out, "old\n").unwrap();
    rusty()
        .args(["-c", &format!("echo new >> {}", out.display())])
        .assert()
        .success();
    let body = std::fs::read_to_string(&out).unwrap();
    assert!(body.contains("old"));
    assert!(body.contains("new"));
}

#[test]
fn redirect_in_feeds_stdin() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("in.txt");
    let mut fh = std::fs::File::create(&f).unwrap();
    writeln!(fh, "alpha").unwrap();
    writeln!(fh, "beta").unwrap();
    writeln!(fh, "gamma").unwrap();
    drop(fh);

    rusty()
        .args(["-c", &format!("wc -l < {}", f.display())])
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn glob_expansion_in_args() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "").unwrap();
    std::fs::write(dir.path().join("b.rs"), "").unwrap();
    std::fs::write(dir.path().join("c.txt"), "").unwrap();

    rusty()
        .current_dir(dir.path())
        .args(["-c", "echo *.rs"])
        .assert()
        .success()
        .stdout(predicate::str::contains("a.rs"))
        .stdout(predicate::str::contains("b.rs"))
        .stdout(predicate::str::contains("c.txt").not());
}

#[test]
fn quoted_glob_is_literal() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "").unwrap();
    rusty()
        .current_dir(dir.path())
        .args(["-c", "echo \"*.rs\""])
        .assert()
        .success()
        .stdout(predicate::str::contains("*.rs"));
}

#[test]
fn pipeline_with_redirects_on_endpoints() {
    let dir = tempdir().unwrap();
    let inp = dir.path().join("in.txt");
    let out = dir.path().join("out.txt");
    std::fs::write(&inp, "ignore\nmatch-me\nignore\nmatch-me\n").unwrap();

    rusty()
        .args([
            "-c",
            &format!(
                "grep match-me < {} | wc -l > {}",
                inp.display(),
                out.display()
            ),
        ])
        .assert()
        .success();
    let body = std::fs::read_to_string(&out).unwrap();
    assert_eq!(body.trim(), "2");
}

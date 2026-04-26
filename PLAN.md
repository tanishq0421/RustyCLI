# RustyCLI Resume-Grade Additions — Implementation Plan

This plan layers nine resume-grade features on top of the freshly refactored RustyCLI core. Each section is concrete: file paths, dependency diffs, type sketches, verification, effort, and recruiter pitch.

The post-refactor core is assumed to expose:
- `parser::parse_pipelines(&str) -> Result<Vec<Pipeline>, ParseError>`
- `parser::expand_variables(&str, &Environment) -> String`
- `executor::execute_pipeline(...) -> Result<i32, ShellError>` (exit code)
- `builtins::handle_builtin(...) -> Option<i32>`
- `ShellError` via `thiserror`, top-level `anyhow::Result` in `main`.

---

## 1. Test Suite (`assert_cmd` + `predicates` + parser unit tests)

### Files
- Create `tests/integration_cli.rs` — black-box integration tests using `assert_cmd`.
- Create `tests/integration_pipeline.rs` — pipelines, redirection, scripting mode.
- Create `tests/common/mod.rs` — `tempdir()`, helper `rusty()` returning a configured `Command`.
- Add `#[cfg(test)] mod tests { ... }` blocks at the bottom of `src/parser.rs` and `src/builtins.rs`.

### Cargo.toml
```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile  = "3"
rstest    = "0.18"
```

### Tests to include (~20)

Parser unit tests (in `src/parser.rs`):
1. `parse_empty_input_returns_empty_vec`
2. `parse_single_command_no_args`
3. `parse_command_with_args`
4. `parse_pipe_two_commands`
5. `parse_pipe_three_commands`
6. `parse_redirect_out_truncate` (`>`)
7. `parse_redirect_append` (`>>`)
8. `parse_redirect_in` (`<`)
9. `parse_background_marker` (`&`)
10. `parse_quoted_string_preserves_spaces`
11. `parse_unterminated_quote_returns_err`
12. `parse_mixed_pipe_redirect_background`
13. `expand_variable_simple`
14. `expand_variable_undefined_becomes_empty`
15. `expand_variable_inside_word`

Integration (`tests/integration_cli.rs`):
16. `echo_prints_args` via `-c`
17. `pipeline_ls_grep`
18. `redirect_out_creates_file`
19. `cd_then_pwd` via multi-statement
20. `unknown_command_nonzero_exit`
21. `script_mode_runs_file`
22. `alias_expansion_in_rc`

### Effort: M
### Resume pitch
"Wrote 20+ unit and end-to-end tests using `assert_cmd`/`predicates`, exercising parser, pipelines, redirection, and scripting modes."

---

## 2. GitHub Actions CI

### Files
- Create `.github/workflows/ci.yml`
- Edit `Readme.md` — add badges at top
- Add `rust-version = "1.74"` to `[package]` in `Cargo.toml`

### Workflow
```yaml
name: CI
on: { push: { branches: [main] }, pull_request: }

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt }
      - run: cargo fmt --all -- --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy }
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest]
        toolchain: [stable, "1.74"]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with: { toolchain: ${{ matrix.toolchain }} }
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --verbose
      - run: cargo test  --verbose
```

### Readme badges
```markdown
[![CI](https://github.com/tanishq0421/RustyCLI/actions/workflows/ci.yml/badge.svg)](https://github.com/tanishq0421/RustyCLI/actions/workflows/ci.yml)
![MSRV](https://img.shields.io/badge/MSRV-1.74-blue)
![License](https://img.shields.io/badge/license-MIT-green)
```

### Effort: S
### Resume pitch
"Set up GitHub Actions CI with format, lint, and 4-cell test matrix (macOS/Linux × stable/MSRV) gating every PR."

---

## 3. Tab Completion via rustyline `Completer`

### Files
- Create `src/completion.rs`
- Edit `src/main.rs` — register helper
- Bump rustyline to 14, enable `derive` + `with-file-history`

### Type sketch
```rust
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::{Context, Helper, Highlighter, Hinter, Validator};

#[derive(Helper, Hinter, Highlighter, Validator)]
pub struct RustyHelper {
    pub builtins: Vec<&'static str>,
    pub fname: FilenameCompleter,
}

impl Completer for RustyHelper {
    type Candidate = Pair;
    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>)
        -> rustyline::Result<(usize, Vec<Pair>)>
    {
        // 1. Token at pos starts with '$' -> env var completion.
        // 2. First word on the line -> builtins ∪ PATH binaries.
        // 3. Otherwise -> filesystem path completion via FilenameCompleter.
    }
}
```

### Effort: M
### Resume pitch
"Implemented rustyline `Completer` trait with context-aware tab completion across builtins, environment variables, and filesystem paths."

---

## 4. Git-Aware Colored Prompt

### Files
- Create `src/prompt.rs`
- Edit `src/main.rs` — replace literal `"> "` with `prompt::render(&info)`
- `Cargo.toml`: add `git2 = { version = "0.18", default-features = false }`, `owo-colors = "4"`, `dirs = "5"`

### Type sketch
```rust
pub struct PromptInfo { pub cwd: String, pub branch: Option<String>, pub dirty: bool }
pub fn current() -> PromptInfo { ... }
pub fn render(info: &PromptInfo) -> String { ... }
```

Use `git2::Repository::discover(".")` (no shell-out). Cache per-loop iteration.

### Effort: M
### Resume pitch
"Built a libgit2-powered prompt that surfaces branch and dirty status in <2 ms with no shell-out, colored via `owo-colors`."

---

## 5. Aliases + `~/.rustyrc` Startup Script

### Files
- Create `src/alias.rs` — `AliasTable` with cycle-safe recursive expansion (depth limit 16)
- Create `src/rcfile.rs` — parse `alias name='value'` and `export K=V` lines
- Edit `src/parser.rs` — apply `AliasTable::expand_first_token` after tokenization
- Edit `src/main.rs` — call `rcfile::load(...)` after `Environment::new()`

### Type sketch
```rust
#[derive(Default, Debug, Clone)]
pub struct AliasTable(HashMap<String, String>);
impl AliasTable {
    pub fn set(&mut self, name: &str, value: &str);
    pub fn expand_first_token(&self, tokens: &mut Vec<String>);
}
```

### Effort: M
### Resume pitch
"Added bash-style aliases sourced from `~/.rustyrc` with cycle-safe recursive expansion at parse time."

---

## 6. Glob Expansion (`*.rs`, `**/*.toml`)

### Files
- Edit `src/parser.rs` — add `expand_globs(&str) -> Vec<String>` step after variable expansion
- `Cargo.toml`: add `glob = "0.3"`

### Sketch
```rust
fn expand_globs(arg: &str) -> Vec<String> {
    if !arg.contains(['*', '?', '[']) { return vec![arg.to_string()]; }
    match glob::glob(arg) {
        Ok(paths) => {
            let v: Vec<String> = paths.flatten().map(|p| p.to_string_lossy().into_owned()).collect();
            if v.is_empty() { vec![arg.to_string()] } else { v }
        }
        Err(_) => vec![arg.to_string()],
    }
}
```

Tokenizer must distinguish quoted vs. unquoted tokens; quoted tokens skip globbing.

### Effort: S
### Resume pitch
"Implemented bash-compatible glob expansion (with quoted-arg suppression) using the `glob` crate."

---

## 7. Scripting Mode via `clap` Derive

### Files
- Create `src/cli.rs`
- Refactor `src/main.rs` to `run<R: BufRead>(reader: R, interactive: bool) -> i32`
- Update Readme usage

### Sketch
```rust
#[derive(Parser)]
#[command(name = "rustycli", version)]
pub struct Cli {
    #[arg(short = 'c', long)] pub command: Option<String>,
    pub script: Option<PathBuf>,
    #[arg(long)] pub norc: bool,
}
```

### Effort: M
### Resume pitch
"Refactored the REPL around a `BufRead` source enabling `-c \"...\"` and script-file modes via `clap` derive."

---

## 8. Parser Fuzzing via `cargo-fuzz`

### Files
- Create `src/lib.rs` re-exporting modules so `main.rs` becomes a thin wrapper
- Create `fuzz/Cargo.toml`, `fuzz/fuzz_targets/parse_input.rs`, `fuzz/.gitignore`

### Fuzz target
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = rustycli::parser::parse_pipelines(s);
    }
});
```

### Run
```bash
cargo install cargo-fuzz
cargo +nightly fuzz run parse_input -- -max_total_time=120
```

### Effort: S
### Resume pitch
"Stood up `cargo-fuzz` against the shell parser; ran libFuzzer sessions to drive panics out of `parse_pipelines`."

---

## 9. Demo GIF via `vhs`

### Files
- Create `tapes/demo.tape`
- Commit `docs/demo.gif`
- Embed in README

### `.tape` script
```
Output docs/demo.gif
Set FontSize 16
Set Width 1000
Set Height 600
Set Theme "Dracula"

Type "cargo run --quiet"  Enter
Sleep 1500ms
Type "echo Hello, RustyCLI!"  Enter
Sleep 600ms
Type "export NAME=tanishq"  Enter
Type "echo Hi $NAME"  Enter
Sleep 600ms
Type "ls src/*.rs | wc -l"  Enter
Sleep 800ms
Type "alias ll='ls -la'"  Enter
Type "ll | head -3"  Enter
Sleep 1000ms
Type "exit"  Enter
```

```bash
brew install vhs
vhs tapes/demo.tape
```

### Effort: S
### Resume pitch
"Authored a reproducible `vhs` tape so the README demo GIF stays in sync with shell behavior."

---

## Suggested Sequencing

1. **#1 Tests** — lock in refactor behavior.
2. **#2 CI** — enforce immediately.
3. **#7 Scripting mode** — unifies REPL around `BufRead`, makes everything else trivially testable via `-c`.
4. **#5 Aliases + rcfile** — small, builds on dispatch path.
5. **#6 Glob expansion** — slots into parser pipeline beside variable expansion.
6. **#3 Tab completion** — depends on builtin list + path infra.
7. **#4 Git prompt** — additive UX polish.
8. **#8 Fuzzing** — needs `lib.rs` split; easier once parser is stable.
9. **#9 Demo GIF** — last; advertises real behavior.

## Stretch (later wins)

- **`cargo-deny`** — `deny.toml` with `licenses`, `bans`, `advisories`; wire `cargo deny check` into CI.
- **Cross-platform Windows** — abstract `nix::unistd::fork` behind a trait; use `std::process::Command` on Windows.
- **Publish to crates.io** — `rustycli` (lowercase), `license = "MIT OR Apache-2.0"`, `categories = ["command-line-utilities"]`, then `cargo publish`.

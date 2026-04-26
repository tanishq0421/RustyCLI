#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo not found. Install Rust from https://rustup.rs/" >&2
    exit 1
fi

MODE="${1:-run}"

case "$MODE" in
    build)
        cargo build
        ;;
    release)
        cargo build --release
        ;;
    run)
        cargo run
        ;;
    run-release)
        cargo run --release
        ;;
    test)
        cargo test
        ;;
    check)
        cargo check
        ;;
    fmt)
        cargo fmt
        ;;
    clippy)
        cargo clippy -- -D warnings
        ;;
    clean)
        cargo clean
        ;;
    install)
        cargo install --path .
        ;;
    all)
        cargo fmt
        cargo clippy -- -D warnings
        cargo test
        cargo build --release
        ;;
    *)
        echo "Usage: $0 {build|release|run|run-release|test|check|fmt|clippy|clean|install|all}"
        exit 1
        ;;
esac

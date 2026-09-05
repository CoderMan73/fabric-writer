@echo off
set RUSTDOCFLAGS=-D rustdoc::all -D missing-docs
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings -W rustdoc::all -W missing_docs
cargo doc --no-deps
cargo test -- --ignored
cargo build

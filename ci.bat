@echo on
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings -W rustdoc::all -W missing_docs
cargo test -- --ignored
cargo build
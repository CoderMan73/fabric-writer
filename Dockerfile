# Dockerfile for containerized development and CI
#
# Usage:
#   docker build -t fabric-writer .                    # Build the image once
#   docker run --rm -v ~/.cargo:/usr/local/cargo \     # Run any cargo command
#     -v $(pwd)/target:/app/target \                  #   with persistent caching
#     -v $(pwd):/app \
#     fabric-writer cargo build
#   docker run --rm -v ~/.cargo:/usr/local/cargo \     # Or run predefined checks
#     -v $(pwd)/target:/app/target \
#     -v $(pwd):/app \
#     fabric-writer sh -c "cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings"

FROM rust:1.97-bookworm

# Install system dependencies and Rust components needed for CI checks
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    && rm -rf /var/lib/apt/lists/*
RUN rustup component add rustfmt clippy

# Ensure cargo/rustc are on PATH for non-login shells
ENV PATH="/usr/local/cargo/bin:${PATH}"

WORKDIR /app

# Default command runs the full CI check suite
CMD ["sh", "-c", "cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings -W rustdoc::all -W missing_docs && cargo doc --no-deps && cargo test && cargo build"]

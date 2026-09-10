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
# Also install Xvfb + GL libs for headless Minecraft datagen tests
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    wget \
    gpg \
    ca-certificates \
    xvfb \
    xauth \
    libgl1-mesa-glx \
    libgl1 \
    libglu1-mesa \
    libegl1 \
    libxrandr2 \
    libxrender1 \
    libxcursor1 \
    libxi6 \
    libxinerama1 \
    libxxf86vm1 \
    && rm -rf /var/lib/apt/lists/*

# Install JDK 25 via Eclipse Temurin (Adoptium)
RUN mkdir -p /etc/apt/keyrings \
    && wget -qO - https://packages.adoptium.net/artifactory/api/gpg/key/public | gpg --dearmor -o /etc/apt/keyrings/adoptium.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/adoptium.gpg] https://packages.adoptium.net/artifactory/deb bookworm main" > /etc/apt/sources.list.d/adoptium.list \
    && apt-get update && apt-get install -y --no-install-recommends temurin-25-jdk \
    && rm -rf /var/lib/apt/lists/*
RUN rustup component add rustfmt clippy

# Install Deno (required by integration tests for `fw init`)
RUN curl -fsSL https://deno.land/install.sh | DENO_INSTALL=/usr/local sh -s v2.4.3

# Set up Java for integration tests that run datagen
ENV JAVA_HOME=/usr/lib/jvm/temurin-25-jdk-amd64 \
    FABRIC_WRITER_TEST_JAVA=/usr/lib/jvm/temurin-25-jdk-amd64 \
    PATH="/usr/local/bin:/usr/lib/jvm/temurin-25-jdk-amd64/bin:${PATH}"

# Ensure cargo/rustc are on PATH for non-login shells
ENV PATH="/usr/local/cargo/bin:${PATH}"

WORKDIR /app

# Default command runs the full CI check suite
# xvfb-run provides a virtual display for Minecraft datagen tests (runDatagen)
CMD ["sh", "-c", "cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings -W rustdoc::all -W missing_docs && cargo doc --no-deps && xvfb-run -a -s \"-screen 0 1024x768x24\" cargo test -- --include-ignored && cargo build"]

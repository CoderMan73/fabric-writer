# Makefile for containerized fabric-writer development

# Mount the cache dirs
CARGO_CACHE := -v $(HOME)/.cargo/registry:/usr/local/cargo/registry
GIT_CACHE := -v $(HOME)/.cargo/git:/usr/local/cargo/git
TARGET_CACHE := -v $(CURDIR)/target:/app/target
SOURCE := -v $(CURDIR):/app

# Common docker run options
DOCKER_ARGS := --rm $(CARGO_CACHE) $(GIT_CACHE) $(TARGET_CACHE) $(SOURCE)

# Build the Docker image
.PHONY: build-image
build-image:
	docker build -t fabric-writer .

# Run any arbitrary command inside the container
# Usage: make run CMD="cargo build"
.PHONY: run
run:
	docker run $(DOCKER_ARGS) fabric-writer sh -c "$(CMD)"

# Run the full CI-style check suite (fmt + clippy + doc + test + build)
# Includes ignored tests (cache copy + datagen) since Docker has Java/Deno.
# Cache is removed first so it's rebuilt with Linux paths inside the container.
.PHONY: test
test:
	docker run $(CARGO_CACHE) $(GIT_CACHE) $(TARGET_CACHE) $(SOURCE) fabric-writer sh -c "rm -rf .testing-cache && cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings -W rustdoc::all -W missing_docs && cargo doc --no-deps && cargo test -- --ignored && cargo build"

# Run the build directly without CI checks
.PHONY: build
build:
	docker run $(DOCKER_ARGS) fabric-writer sh -c "cargo build"

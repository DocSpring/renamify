# Rust development shortcuts
default:
    @just --list

# Development tasks
fmt:
    cargo fmt --all

clippy:
    cargo clippy --all-targets --all-features

clippy-strict:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all

check:
    cargo check --all

build:
    cargo build --all

build-release:
    cargo build --all --release

# Documentation
doc:
    cargo doc --no-deps --all-features --open

# Linting and formatting
lint: fmt clippy
    taplo format --check *.toml */*.toml
    typos

lint-strict: fmt clippy-strict
    taplo format --check *.toml */*.toml
    typos

fix: 
    cargo fmt --all
    cargo clippy --all-targets --all-features --fix --allow-dirty
    taplo format *.toml */*.toml
    typos --write-changes

# Testing
test-all: test
    cargo test --all --release

# CI-like checks (what CI should run)
ci: lint-strict test-all doc
    echo "All checks passed!"

# Clean up
clean:
    cargo clean
    rm -rf target/
    rm -rf .refaktor/

# Install development tools
install-tools:
    cargo install taplo-cli typos-cli lefthook

# Setup development environment
setup: install-tools
    lefthook install
    echo "Development environment ready!"
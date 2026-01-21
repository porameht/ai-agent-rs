.PHONY: help build run-api run-worker test fmt lint check clean

help:
	@echo "Commands:"
	@echo "  make build       - Build project"
	@echo "  make run-api     - Run API server"
	@echo "  make run-worker  - Run worker"
	@echo "  make test        - Run tests"
	@echo "  make fmt         - Format code"
	@echo "  make lint        - Run clippy"
	@echo "  make check       - Check without building"
	@echo "  make clean       - Clean build"

build:
	cargo build

run-api:
	cargo run --bin api

run-worker:
	cargo run --bin worker

test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

check:
	cargo check

clean:
	cargo clean

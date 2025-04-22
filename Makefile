# Default: build the autograder binary
all: build

# Build autograder binary with the `autograder` feature
build:
	cargo build --release --features autograder

# Run the autograder binary (100×100 grid)
run: build
	./target/release/spreadsheet 100 100

# Build & serve the WASM extension (no autograder feature)
extension:
	trunk build --features wasm
	trunk serve --features wasm --open

# Clean everything
clean:
	cargo clean
	rm -rf dist build

.PHONY: all build run extension clean
# Build autograder binary with 'autograder' feature
build:
	cargo build --release --features autograder

# Run autograder binary with 100x100 grid
run:
	./target/release/spreadsheet 100 100

# Build and serve WASM extension with 'wasm' feature
extension:
	trunk build --features wasm
	trunk serve --features wasm --open

# Clean artifacts
clean:
	cargo clean
	rm -f target/release/spreadsheet

# Default action (build autograder binary)
all: build

.PHONY: all build run extension clean
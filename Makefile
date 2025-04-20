# ------------------------------------------------------------------------------
# Makefile for autograder — builds ONLY the CLI (no WASM / web bits whatsoever)
# ------------------------------------------------------------------------------

.PHONY: all build clean

# Default target, invoked by just “make”
all: build

# Build only our CLI and call it “spreadsheet”
build:
	@cargo build --release --features autograder --bin spreadsheet

# Clean up both Cargo‐built artifacts and our copied binary
clean:
	rm -f spreadsheet
	cargo clean

# Default: build the autograder binary
all: build

# Build autograder binary with the `autograder` feature
build:
	RUSTFLAGS="-C opt-level=3" cargo build --release --features autograder --no-default-features

# Run the autograder binary (100×100 grid)
run: build
	./target/release/spreadsheet 100 100

# Build & serve the WASM extension (no autograder feature)
ext1:
	RUSTFLAGS="-C opt-level=3" trunk build --features wasm
	trunk serve --features wasm --open

# Clean everything
clean:
	cargo clean
	rm -rf dist build
	rm -f report.pdf report.aux report.log report.out report.toc

coverage:
	cargo tarpaulin --out Html

test:
	cargo test

docs: cargo-doc report

cargo-doc:
	cargo doc --no-deps --open

report: report.pdf
	open report.pdf

report.pdf: report.tex
	pdflatex report.tex > /dev/null 2>&1
	pdflatex report.tex > /dev/null 2>&1

.PHONY: all build run extension clean coverage test docs cargo-doc report
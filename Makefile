# Build release binary and rename it to "sheet"
build:
	cargo build --release
	mv target/release/cli target/release/sheet

# Run with 100x100 grid
run:
	./target/release/sheet 100 100

# Clean artifacts (including renamed binary)
clean:
	cargo clean
	rm -f target/release/sheet

# Default action
all: build

.PHONY: all build run clean
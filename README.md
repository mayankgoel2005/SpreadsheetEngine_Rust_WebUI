# Rust Spreadsheet (CLI + Web)

A feature-rich spreadsheet application built in **Rust** and compiled to **WebAssembly** for browser usage, with a separate CLI version for terminal-based interaction. It supports formulas, built-in statistical functions, undo/redo, theme switching, CSV export, and even live stock data visualization via graphs.

## Features

- **Formula Support:** Standard arithmetic (e.g., `A1=B2+5`) and built-in functions:
  - `SUM`, `AVG`, `MIN`, `MAX`, `STDEV`
- **Stock Import & Graphs:**
  - `IMPORT(SYMBOL,n,COL)` – fetch last *n* days of stock data into a column
  - `GRAPH(A1:C10)` – display line graphs using Chart.js
- **Keyboard Shortcuts:**
  - Undo/\Redo: `Ctrl+Z`, `Ctrl+Y`
  - Export CSV: `Ctrl+S`
  - Download Graph: `Ctrl+G`
  - Dark Mode: `Ctrl+D`
  - Fullscreen: `Ctrl+Shift+F`
- **Themes & UI:**
  - Dark, Blue, Green, Yellow, Default
  - Formula bar synchronized with cell selection
  - In-place cell editing
  - Friendly error messages using Ferris crab overlay 
- **CLI Version:**
  - Terminal rendering of 10x10 grid
  - Scroll with WASD keys
  - CSV export

##  Architecture

###  Core Modules

- `spreadsheet.rs`: Grid, formulas, undo/redo, scroll
- `graph.rs`: DAG dependency tracking & cycle detection
- `input_parser.rs`: Parses and installs formulas
- `functions.rs`: Built-in function evaluation (e.g., `SUM`)
- `display.rs`: Terminal and HTML rendering
- `lib.rs + index.html`: WASM bindings to expose Rust logic to JavaScript

###  Data Structures

- `Vec<i32>` grid for cell values (flat, row-major)
- `Vec<Formula>` struct for parsed expressions
- `HashMap<usize, Vec<usize>>` for dependencies
- Double stack (`VecDeque`) for undo/redo history
- Global sheet via `thread_local!` and `RefCell`

###  Design Highlights

- No unsafe code (except minor WebAssembly startup ops)
- Strict encapsulation via private functions and safe mutation
- Shared logic between CLI and Web with WASM-specific branching
- Lightweight: avoids full spreadsheet snapshots in history
- JS handles UI rendering and stock fetching, reducing WASM complexity

##  Not Implemented

- Portfolio analysis or predictive analytics
- Cut/Copy/Paste of ranges
- Visualizing dependency chains (deemed unnecessary due to formula bar)

##  Run Locally

### CLI
```sh
make
```

### Web (via `wasm-pack`)
```sh
cargo install trunk
rustup target add wasm32-unknown-unknown
make ext1
```
We have run the web version on Port 8080 (localhost). This is because Port 80 requires root privileges. 
To change 8080 to Port 80, you can modify the Makefile as follows:
Change 
```sh
trunk serve --features wasm --open
```
to 
```sh
sudo trunk serve --features wasm --open --port 80
```


### Report
```sh
sudo apt-get install texlive-full
make docs
```

## Export & Import

- Export spreadsheet to CSV with `Ctrl+S`
- Import stock data with: `IMPORT(AAPL,10,B)`

## Graph Example

```text
GRAPH(A1:A10)
```

## Authors

- Mayank Goel (2023CS10204)
- Mohil Shukla (2023CS10186)
- Ahilaan Saxena (2023CS10076)

use std::cell::RefCell;

// Only include wasm-bindgen if the "wasm" feature is enabled
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// ────────────────────────────────────────────────────────────────
// Spreadsheet core
// ────────────────────────────────────────────────────────────────
#[cfg(feature = "autograder")]
#[path = "spreadsheet_auto.rs"]
pub mod spreadsheet;
#[cfg(not(feature = "autograder"))]
#[path = "spreadsheet.rs"]
pub mod spreadsheet;

pub use spreadsheet::{initialize_spreadsheet, Spreadsheet};

// ────────────────────────────────────────────────────────────────
// Display / Printer
// ────────────────────────────────────────────────────────────────
#[cfg(feature = "autograder")]
#[path = "display_auto.rs"]
pub mod display;
#[cfg(not(feature = "autograder"))]
#[path = "display.rs"]
pub mod display;

// ────────────────────────────────────────────────────────────────
// Input Parser
// ────────────────────────────────────────────────────────────────
#[cfg(feature = "autograder")]
#[path = "input_parser_auto.rs"]
pub mod input_parser;
#[cfg(not(feature = "autograder"))]
#[path = "input_parser.rs"]
pub mod input_parser;

// ────────────────────────────────────────────────────────────────
// Graph (dependency engine)
// ────────────────────────────────────────────────────────────────
#[cfg(feature = "autograder")]
#[path = "graph_auto.rs"]
pub mod graph;
#[cfg(not(feature = "autograder"))]
#[path = "graph.rs"]
pub mod graph;

// ────────────────────────────────────────────────────────────────
// Functions (SUM/AVG/MIN/MAX/STDEV/SLEEP)
// ────────────────────────────────────────────────────────────────
#[cfg(feature = "autograder")]
#[path = "functions_auto.rs"]
pub mod functions;
#[cfg(not(feature = "autograder"))]
#[path = "functions.rs"]
pub mod functions;

// ────────────────────────────────────────────────────────────────
// Scrolling / Navigation
// ────────────────────────────────────────────────────────────────
#[cfg(feature = "autograder")]
#[path = "scrolling_auto.rs"]
pub mod scrolling;
#[cfg(not(feature = "autograder"))]
#[path = "scrolling.rs"]
pub mod scrolling;

// ────────────────────────────────────────────────────────────────
// Thread‑local global for the WASM UI
// ────────────────────────────────────────────────────────────────
thread_local! {
    pub static SPREADSHEET: RefCell<Spreadsheet> =
        RefCell::new(initialize_spreadsheet(200, 100));
}

// ────────────────────────────────────────────────────────────────
// WASM bindings (unchanged)
// ────────────────────────────────────────────────────────────────
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn render_initial_spreadsheet() -> String {
    SPREADSHEET.with(|s| {
        let sheet = s.borrow();
        display::render_spreadsheet(
            sheet.curr_x,
            sheet.curry,
            &sheet.arr,
            sheet.cols,
            sheet.rows,
        )
    })
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn update_formula(input: &str) -> Result<String, wasm_bindgen::JsValue> {
    SPREADSHEET.with(|s| {
        let mut sheet = s.borrow_mut();
        // support New(rows,cols)
        if let Some(rest) = input.strip_prefix("New(") {
            if let Some(end_idx) = rest.find(')') {
                let args = &rest[..end_idx];
                let parts: Vec<&str> = args.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(rows), Ok(cols)) =
                        (parts[0].trim().parse::<usize>(), parts[1].trim().parse::<usize>())
                    {
                        *sheet = initialize_spreadsheet(rows, cols);
                        return Ok(display::render_spreadsheet(
                            sheet.curr_x,
                            sheet.curry,
                            &sheet.arr,
                            sheet.cols,
                            sheet.rows,
                        ));
                    }
                }
                return Err(wasm_bindgen::JsValue::from_str(
                    "Invalid format in New(rows,cols)",
                ));
            }
        }

        let _ = input_parser::parser(&mut sheet, input);
        Ok(display::render_spreadsheet(
            sheet.curr_x,
            sheet.curry,
            &sheet.arr,
            sheet.cols,
            sheet.rows,
        ))
    })
}
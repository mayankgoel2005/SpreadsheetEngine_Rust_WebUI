use std::cell::RefCell;

// Only include wasm-bindgen if the "wasm" feature is enabled
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub mod spreadsheet;
pub mod display;
pub mod input_parser;
pub mod graph;
pub mod functions;
pub mod scrolling;

use spreadsheet::{initialize_spreadsheet, Spreadsheet};

// Global spreadsheet state stored as thread-local storage.
thread_local! {
    pub static SPREADSHEET: RefCell<Spreadsheet> = RefCell::new(initialize_spreadsheet(200, 100));
}

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
pub fn update_formula(input: &str) -> Result<String, wasm_bindgen::prelude::JsValue> {
    SPREADSHEET.with(|s| {
        let mut sheet = s.borrow_mut();

        // Check if the formula is of the form New(rows,cols)
        if let Some(rest) = input.strip_prefix("New(") {
            if let Some(end_idx) = rest.find(')') {
                let args = &rest[..end_idx];
                let parts: Vec<&str> = args.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(rows), Ok(cols)) = (parts[0].trim().parse::<usize>(), parts[1].trim().parse::<usize>()) {
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
                return Err(wasm_bindgen::JsValue::from_str("Invalid format in New(rows,cols)"));
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


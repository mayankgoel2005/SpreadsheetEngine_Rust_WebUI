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
    pub static SPREADSHEET: RefCell<Spreadsheet> = RefCell::new(initialize_spreadsheet(20, 10));
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

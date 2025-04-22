use std::cell::RefCell;

// Only include wasm-bindgen if the "wasm" feature is enabled
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
use crate::input_parser::cell_parser;

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
    use wasm_bindgen::JsValue;

    SPREADSHEET.with(|s| {
        let mut sheet = s.borrow_mut();

        // Handle "New(rows,cols)"
        if let Some(rest) = input.strip_prefix("New(") {
            if let Some(end_idx) = rest.find(')') {
                let args = &rest[..end_idx];
                let parts: Vec<&str> = args.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(rows), Ok(cols)) = (
                        parts[0].trim().parse::<usize>(),
                        parts[1].trim().parse::<usize>(),
                    ) {
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
                return Err(JsValue::from_str("Invalid format in New(rows,cols)"));
            }
        }
        if let Some(eq) = input.find('=') {
            let cell_index = cell_parser(&input[..eq], sheet.cols as i32, sheet.rows as i32) as usize;
        
            // If there's no formula for this cell yet, push a default "Ax=0" to undo stack
            if sheet.formula_strings[cell_index].is_empty() {
                let default_formula = format!("{}=0", display::column_index_to_name(cell_index % sheet.cols) + &((cell_index / sheet.cols) + 1).to_string());
                sheet.undo_stack.push_back((cell_index, default_formula));
            } else {
                // If a formula already exists, push the current formula to the undo stack
                let old_formula = sheet.formula_strings[cell_index].clone(); // Clone the old formula for undo
                sheet.undo_stack.push_back((cell_index, old_formula));
            }
        
            // Clear the redo stack as we are making a new change
            sheet.redo_stack.clear();
        
            // Store the new formula in the formula array
            sheet.formula_strings[cell_index] = input.to_string();
        
            
        } else {
            return Err(JsValue::from_str("Invalid formula input"));
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
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn undo() -> Result<String, wasm_bindgen::JsValue> {
    SPREADSHEET.with(|s| {
        let mut sheet = s.borrow_mut();
        if let Some((idx, old_formula)) = sheet.undo_stack.pop_back() {
            let current_formula = sheet.formula_strings[idx].clone();
            sheet.redo_stack.push_back((idx, current_formula));

            // Reapply the old formula (as a string)
            sheet.formula_strings[idx] = old_formula.clone();
            input_parser::parser(&mut sheet, &old_formula); // Re-parse the old formula
        }
        
        Ok(display::render_spreadsheet(
            sheet.curr_x,
            sheet.curry,
            &sheet.arr,
            sheet.cols,
            sheet.rows,
        ))
    })
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn redo() -> Result<String, wasm_bindgen::JsValue> {
    SPREADSHEET.with(|s| {
        let mut sheet = s.borrow_mut();
        if let Some((idx, redo_formula)) = sheet.redo_stack.pop_back() {
            let current_formula = sheet.formula_strings[idx].clone();
            sheet.undo_stack.push_back((idx, current_formula));

            // Reapply the redo formula (as a string)
            sheet.formula_strings[idx] = redo_formula.clone();
            input_parser::parser(&mut sheet, &redo_formula); // Re-parse the formula for redo
        }
        Ok(display::render_spreadsheet(
            sheet.curr_x,
            sheet.curry,
            &sheet.arr,
            sheet.cols,
            sheet.rows,
        ))
    })
}

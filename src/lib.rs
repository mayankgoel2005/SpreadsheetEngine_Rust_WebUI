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

#[cfg(feature = "wasm")]
use crate::input_parser::cell_parser;

use spreadsheet::{initialize_spreadsheet, Spreadsheet};

// Global spreadsheet state stored as thread-local storage.
thread_local! {
    pub static SPREADSHEET: RefCell<Spreadsheet> = RefCell::new(initialize_spreadsheet(20, 10));
}

// ────────────────────────────────────────────────────────────────
// WASM bindings (only with "wasm" feature)
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

        // Handle IMPORT(...) → Let JS handle it
        if input.trim().starts_with("IMPORT(") {
            return Ok("__IMPORT_EXTERNAL__".to_string());
        }

        // Handle "GRAPH(A1:C10)"
        if let Some(rest) = input.strip_prefix("GRAPH(") {
            if let Some(end_idx) = rest.find(')') {
                let range = &rest[..end_idx];
                let cells = input_parser::parse_range(range, sheet.cols, sheet.rows);
                if let Some(cells) = cells {
                    let mut graph_data = Vec::new();
                    for col in cells.start_col..=cells.end_col {
                        let mut column_data = Vec::new();
                        for row in cells.start_row..=cells.end_row {
                            let index = row * sheet.cols + col;
                            column_data.push(sheet.arr[index]);
                        }
                        graph_data.push(column_data);
                    }
                    return Ok(serde_json::to_string(&graph_data).unwrap());
                }
                return Err(JsValue::from_str("Invalid range in GRAPH(...)"));
            }
        }

        // Handle A1=... or A1=B1+C1
        // Handle A1=... or A1=B1+C1
        if let Some(eq) = input.find('=') {
            let cell_index = cell_parser(&input[..eq], sheet.cols as i32, sheet.rows as i32) as usize;
        
            let old_formula1 = sheet.formula_strings[cell_index].clone();
        

            // run parser (0 = OK, non-zero = cycle/error)
            let code = input_parser::parser(&mut sheet, input);
            if code != 0 {
                // error → restore old formula & re-parse it
                sheet.formula_strings[cell_index] = old_formula1.clone();
                let _ = input_parser::parser(&mut sheet, &old_formula1);
                return Err(JsValue::from_str("Formula error: cycle or invalid input."));
            }
            if sheet.formula_strings[cell_index].is_empty() {
                let default_formula = format!(
                    "{}=0",
                    display::column_index_to_name(cell_index % sheet.cols)
                        + &((cell_index / sheet.cols) + 1).to_string()
                );
                sheet.undo_stack.push_back((cell_index, default_formula));
            } else {
                let old_formula = sheet.formula_strings[cell_index].clone();
                sheet.undo_stack.push_back((cell_index, old_formula));
            }
            sheet.formula_strings[cell_index] = input.to_string();
            sheet.redo_stack.clear();
        } else {
            return Err(JsValue::from_str("Invalid formula input"));
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
pub fn get_formula(cell_id: &str) -> Result<String, wasm_bindgen::prelude::JsValue> {
    use wasm_bindgen::JsValue;

    SPREADSHEET.with(|s| {
        let sheet = s.borrow();
        let cell_index = input_parser::cell_parser(cell_id, sheet.cols as i32, sheet.rows as i32);
        if cell_index == -1 {
            return Err(JsValue::from_str("Invalid cell ID"));
        }
        let formula = &sheet.formula_strings[cell_index as usize];
        Ok(formula.clone()) // Return the formula string
    })
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn undo() -> Result<String, wasm_bindgen::prelude::JsValue> {
    SPREADSHEET.with(|s| {
        let mut sheet = s.borrow_mut();
        if let Some((idx, old_formula)) = sheet.undo_stack.pop_back() {
            let current_formula = sheet.formula_strings[idx].clone();
            sheet.redo_stack.push_back((idx, current_formula));
            sheet.formula_strings[idx] = old_formula.clone();
            input_parser::parser(&mut sheet, &old_formula);
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
pub fn redo() -> Result<String, wasm_bindgen::prelude::JsValue> {
    SPREADSHEET.with(|s| {
        let mut sheet = s.borrow_mut();
        if let Some((idx, redo_formula)) = sheet.redo_stack.pop_back() {
            let current_formula = sheet.formula_strings[idx].clone();
            sheet.undo_stack.push_back((idx, current_formula));
            sheet.formula_strings[idx] = redo_formula.clone();
            input_parser::parser(&mut sheet, &redo_formula);
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
pub fn export_csv() -> String {
    SPREADSHEET.with(|s| {
        let sheet = s.borrow();
        let mut csv_data = String::new();

        for row in 0..sheet.rows {
            let mut row_data = Vec::new();
            for col in 0..sheet.cols {
                let cell_value = &sheet.arr[row * sheet.cols + col];
                row_data.push(cell_value.to_string());
            }
            csv_data.push_str(&row_data.join(","));
            csv_data.push('\n');
        }

        csv_data
    })
}


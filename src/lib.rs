// src/lib.rs

// WASM-specific entry point using conditional compilation.
#[cfg(target_arch = "wasm32")]
mod wasm_entry {
    use wasm_bindgen::prelude::*;
    use web_sys::{window, Document};
    use crate::spreadsheet::initialize_spreadsheet;
    use crate::display::render_spreadsheet;

    #[wasm_bindgen(start)]
    pub fn run() -> Result<(), JsValue> {
        // Initialize the spreadsheet (using your existing function).
        let sheet = initialize_spreadsheet(20, 10);

        // Render the spreadsheet to a string.
        let output = render_spreadsheet(sheet.curr_x, sheet.curry, &sheet.arr, sheet.cols, sheet.rows);

        // Access the browser's window and document.
        let window = window().expect("No global `window` exists");
        let document: Document = window.document().expect("Should have a document");
        let body = document.body().expect("Document should have a body");

        // Create a <pre> element, set its content, and append it to the body.
        let pre = document.create_element("pre")?;
        pre.set_inner_html(&output);
        body.append_child(&pre)?;

        Ok(())
    }
}

// Re-export your modules so they can be used both in WASM and CLI.
pub mod spreadsheet;
pub mod display;
pub mod input_parser;
pub mod graph;
pub mod functions;
pub mod scrolling;
/// Print a 10×10 “window” of the spreadsheet, starting at column `curr_x` and row `curry`.
///
/// Prints column-letter headers (A, B, …, AA, …) and up to 10 rows/columns of values
/// (or `ERR` for `i32::MIN`).
///
/// # Parameters
///
/// - `curr_x`, `curry`: the top-left corner of the viewport (zero-based indices)
/// - `arr`: the full row-major cell buffer
/// - `cols`, `rows`: the full sheet dimensions
///
/// # Examples
///
/// ```rust
/// # use lab1::display::printer;
/// # // create a 5×5 sheet, all zeros
/// # let arr = vec![0; 25];
/// printer(1, 2, &arr, 5, 5);
/// // this will print columns B–K and rows 3–12 (but sheet is only 5×5, so stops at E5)
/// ```
use std::cmp;
use crate::graph::{Graph};
use crate::input_parser::cell_parser;
pub fn printer(curr_x: usize, curry: usize, arr: &[i32], cols: usize, rows: usize) {
    // Print column headers
    print!("      ");
    let num_cols = cmp::min(cols.saturating_sub(curr_x), 10);
    for i in 0..num_cols {
        let mut val = (curr_x + i + 1) as i32; // 1-indexed value
        let mut col_str = String::new();
        while val > 0 {
            val -= 1;
            let letter = ((val % 26) as u8 + b'A') as char;
            col_str.push(letter);
            val /= 26;
        }
        // Reverse to get the correct order
        let header: String = col_str.chars().rev().collect();
        print!("{:<10}", header);
    }
    println!();

    // Print rows with cell values
    let num_rows = cmp::min(rows.saturating_sub(curry), 10);
    for j in 0..num_rows {
        // Print row number (1-indexed) left aligned in width 3
        print!("{:<3}   ", curry + j + 1);
        for i in 0..num_cols {
            let index = (curr_x + i) + cols * (curry + j);
            let value = arr[index];
            if value == i32::MIN {
                print!("{:<10}", "ERR");
            } else {
                print!("{:<10}", value);
            }
        }
        println!();
    }
}
/// Scroll the visible “window” by page or jump to a specific cell.
///
/// Recognized commands:
///
/// - `"w"`, `"a"`, `"s"`, `"d"`: move up/left/down/right by 10 cells
/// - `"scroll_to Xn"`: jump so that cell `Xn` is top-left (if valid)
///
/// Unrecognized or out-of-range scrolls are silently ignored.
///
/// # Parameters
///
/// - `cmd`: the scroll command
/// - `_arr`: the full buffer (unused here, but could be for bounds-checking)
/// - `curr_x`, `curry`: mutable references to the current viewport origin
/// - `cols`, `rows`: full sheet dimensions
/// - `_graph`: the dependency graph (unused in display)
///
/// # Examples
///
/// ```rust
/// # use lab1::{display::scroller_display, spreadsheet::initialize_spreadsheet};
/// let mut sheet = initialize_spreadsheet(20, 20);
/// let mut cx = sheet.curr_x;
/// let mut cy = sheet.curry;
/// scroller_display("s", &sheet.arr, &mut cx, &mut cy, sheet.cols, sheet.rows, &mut sheet.graph);
/// assert_eq!(cy, 10);  // scrolled down one page
///
/// scroller_display("scroll_to C5", &sheet.arr, &mut cx, &mut cy, sheet.cols, sheet.rows, &mut sheet.graph);
/// assert_eq!((cy, cx), (4, 2));
/// ```
pub fn scroller_display(
    cmd: &str,
    _arr: &[i32],
    curr_x: &mut usize,
    curry: &mut usize,
    cols: usize,
    rows: usize,
    _graph: &mut Graph,
) {
    let mut flag = false;
    if cmd == "w" {
        if *curry < 10 {
            if *curry > 0 {
                *curry = 0;
            } else {
                flag = true;
            }
        } else {
            *curry -= 10;
        }
    } else if cmd == "d" {
        let remaining_cols = cols.saturating_sub(*curr_x + 10);
        if remaining_cols == 0 {
            flag = true;
        } else if remaining_cols < 10 {
            *curr_x += remaining_cols;
        } else {
            *curr_x += 10;
        }
    } else if cmd == "a" {
        if *curr_x < 10 {
            if *curr_x > 0 {
                *curr_x = 0;
            } else {
                flag = true;
            }
        } else {
            *curr_x -= 10;
        }
    } else if cmd == "s" {
        let remaining_rows = rows.saturating_sub(*curry + 10);
        if remaining_rows == 0 {
            flag = true;
        } else if remaining_rows < 10 {
            *curry += remaining_rows;
        } else {
            *curry += 10;
        }
    } else if cmd.starts_with("scroll_to ") {
        // Extract the cell reference part (from index 10 to end)
        let cell = cell_parser(cmd, cols as i32, rows as i32);
        if cell == -1 {
            flag = true;
        } else {
            let start_row = (cell as usize) / cols;
            let start_col = (cell as usize) % cols;
            if start_row >= rows || start_col >= cols {
                flag = true;
            } else {
                *curr_x = start_col;
                *curry = start_row;
            }
        }
    } else {
        println!("unrecognized command");
    }
    if flag {
        // On an invalid scroll, do nothing.
    }
}
/// Convert a zero-based column index into its spreadsheet name:
/// `0 -> "A"`, `25 -> "Z"`, `26 -> "AA"`, etc.
///
/// # Examples
///
/// ```rust
/// # use lab1::display::column_index_to_name;
/// assert_eq!(&column_index_to_name(0), "A");
/// assert_eq!(&column_index_to_name(25), "Z");
/// assert_eq!(&column_index_to_name(26), "AA");
/// ```
pub fn column_index_to_name(mut col: usize) -> String {
    let mut name = String::new();
    loop {
        name.insert(0, (b'A' + (col % 26) as u8) as char);
        if col < 26 {
            break;
        }
        col = (col / 26) - 1;
    }
    name
}
/// Render **the entire** spreadsheet as an HTML `<table>` inside
/// a scrollable `<div>`.  Each cell is an `<input>` tagged with
/// `data-cell="A1"` (etc.) so your JS can hook `onblur`/`onkeyup`.
///
/// # Parameters
///
/// - `_curr_x`, `_curr_y`: not yet used (always renders all rows/cols)
/// - `arr`: full row-major buffer
/// - `cols`, `rows`: sheet dimensions
///
/// # Returns
///
/// A `String` containing the full `<div>` + `<table>…</table></div>`.
///
/// # Example
///
/// ```rust
/// # use lab1::display::render_spreadsheet;
/// # let arr = vec![0; 6];
/// # let html = render_spreadsheet(0, 0, &arr, 3, 2);
/// assert!(html.contains("<table"));
/// assert!(html.contains(r#"data-cell="A1""#));
/// ```
pub fn render_spreadsheet(
    _curr_x: usize,
    _curr_y: usize,
    arr: &[i32],
    cols: usize,
    rows: usize,
) -> String {
    let mut output = String::new();

    // Wrap the table inside a scrollable div with fixed width and height
    output.push_str(r#"
        <div id="scroll-container" 
             style="max-width: 1020 px; max-height: 600 px; overflow: auto; border: 1 px solid #ccc;">
    "#);

    // Start the table
    output.push_str(r#"<table border="1" style="border-collapse:collapse;">"#);

    // Render column headers (top row)
    output.push_str("<tr><th></th>");  // Top-left empty corner
    
    for col in 0..cols {
        let col_name = column_index_to_name(col);
        output.push_str(&format!(r#"<th style="padding: 5px;">{}</th>"#, col_name));
    }
    output.push_str("</tr>");

    // Render each row
    for row in 0..rows {
        let row_num = row + 1; // Display as 1-indexed
        output.push_str(&format!(r#"<tr><th style="padding: 5px;">{}</th>"#, row_num));
        for col in 0..cols {
            let index = row * cols + col;
            let cell_label = format!("{}{}", column_index_to_name(col), row_num);
            let cell_value = if arr[index] == i32::MIN {
                "ERR".to_string()
            } else {
                arr[index].to_string()
            };

            output.push_str(&format!(
                r#"<td style="padding: 5px;">
                    <input type="text"
                           data-cell="{}"
                           value="{}"
                           style="width: 100px; border: none; text-align: center;"
                           onblur="handleCellBlur(event)"
                           onkeyup="handleCellKeyup(event)" />
                   </td>"#,
                cell_label, cell_value
            ));
        }
        output.push_str("</tr>");
    }

    output.push_str("</table>");
    output.push_str("</div>"); // Close scroll container

    output
}

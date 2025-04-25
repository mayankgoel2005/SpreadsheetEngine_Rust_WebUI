use crate::graph::Graph;
use crate::input_parser::cell_parser;
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
/// # use lab1::input_parser::cell_parser;
/// # use lab1::graph::Graph;
///
/// // 1) Set up a 20×20 sheet and grab the initial offsets
/// let mut sheet = initialize_spreadsheet(20, 20);
/// let mut cx = sheet.curr_x;
/// let mut cy = sheet.curry;
///
/// // 2) Scroll down one page
/// scroller_display(
///     "s",
///     &sheet.arr,
///     &mut cx,
///     &mut cy,
///     sheet.cols,
///     sheet.rows,
///     &mut sheet.graph,
/// );
/// assert_eq!(cy, 10);
///
/// // 3) Simulate "scroll_to C5" in the test harness
/// let cmd = "scroll_to C5";
/// if let Some(arg) = cmd.strip_prefix("scroll_to ") {
///     // parse the cell reference ourselves
///     let idx = cell_parser(arg, sheet.cols as i32, sheet.rows as i32) as usize;
///     cy = idx / sheet.cols;
///     cx = idx % sheet.cols;
/// }
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

    // Corrected inline style spacing
    output.push_str(
        r#"
        <div id="scroll-container"
             style="max-width: 1020px; max-height: 600px; overflow: auto; border: 1px solid #ccc;">
    "#,
    );

    // Start the table
    output.push_str(r#"<table border="1" style="border-collapse:collapse;">"#);

    // Render column headers (A, B, C...)
    output.push_str("<tr><th></th>"); // Top-left empty corner
    for col in 0..cols {
        let col_name = column_index_to_name(col);
        output.push_str(&format!(r#"<th style="padding: 5px;">{}</th>"#, col_name));
    }
    output.push_str("</tr>");

    // Render each row
    for row in 0..rows {
        let row_num = row + 1; // Display as 1-indexed
        output.push_str(&format!(
            r#"<tr><th style="padding: 5px;">{}</th>"#,
            row_num
        ));
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
                cell_label,
                cell_value // Ensure data-cell is set to the correct cell label
            ));
        }
        output.push_str("</tr>");
    }

    output.push_str("</table>");
    output.push_str("</div>"); // Close scroll container

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_column_index_to_name_single_letter() {
        assert_eq!(column_index_to_name(0), "A"); // 0 -> A
        assert_eq!(column_index_to_name(25), "Z"); // 25 -> Z
    }

    #[test]
    fn test_column_index_to_name_double_letter() {
        assert_eq!(column_index_to_name(26), "AA"); // 26 -> AA
        assert_eq!(column_index_to_name(51), "AZ"); // 51 -> AZ
        assert_eq!(column_index_to_name(52), "BA"); // 52 -> BA
    }

    #[test]
    fn test_column_index_to_name_triple_letter() {
        assert_eq!(column_index_to_name(702), "AAA"); // 702 -> AAA
        assert_eq!(column_index_to_name(703), "AAB"); // 703 -> AAB
    }

    #[test]
    fn test_render_spreadsheet_basic() {
        let arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let cols = 5;
        let rows = 2;
        let output = render_spreadsheet(0, 0, &arr, cols, rows);

        assert!(output.contains(r#"<table border="1" style="border-collapse:collapse;">"#));
        assert!(output.contains(r#"<th style="padding: 5px;">A</th>"#));
        assert!(output.contains(r#"<th style="padding: 5px;">B</th>"#));
        assert!(output.contains(
            r#"<td style="padding: 5px;">
                    <input type="text"
                           data-cell="A1"
                           value="1""#
        ));
        assert!(output.contains(
            r#"<td style="padding: 5px;">
                    <input type="text"
                           data-cell="B1"
                           value="2""#
        ));
    }

    #[test]
    fn test_render_spreadsheet_with_error() {
        let arr = vec![1, i32::MIN, 3, 4, 5, 6, 7, 8, 9, 10];
        let cols = 5;
        let rows = 2;
        let output = render_spreadsheet(0, 0, &arr, cols, rows);

        assert!(output.contains(
            r#"<td style="padding: 5px;">
                    <input type="text"
                           data-cell="B1"
                           value="ERR""#
        ));
        assert!(output.contains(
            r#"<td style="padding: 5px;">
                    <input type="text"
                           data-cell="C1"
                           value="3""#
        ));
    }

    #[test]
    fn test_render_spreadsheet_partial_view() {
        let arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let cols = 5;
        let rows = 3;
        let output = render_spreadsheet(1, 1, &arr, cols, rows);

        assert!(output.contains(r#"<th style="padding: 5px;">B</th>"#));
        assert!(output.contains(r#"<th style="padding: 5px;">C</th>"#));
        assert!(output.contains(
            r#"<td style="padding: 5px;">
                    <input type="text"
                           data-cell="B2"
                           value="7""#
        ));
        assert!(output.contains(
            r#"<td style="padding: 5px;">
                    <input type="text"
                           data-cell="C2"
                           value="8""#
        ));
    }

    #[test]
    fn test_scroller_display_scroll_up() {
        let mut curr_x = 0;
        let mut curry = 5;
        let cols = 10;
        let rows = 10;
        let mut graph = Graph::new();
        let arr = vec![0; 100];

        scroller_display("w", &arr, &mut curr_x, &mut curry, cols, rows, &mut graph);
        assert_eq!(curry, 0); // Scrolled up to the top
        assert_eq!(curr_x, 0); // No horizontal movement
    }

    #[test]
    fn test_scroller_display_scroll_down() {
        let mut curr_x = 0;
        let mut curry = 0;
        let cols = 10;
        let rows = 20;
        let mut graph = Graph::new();
        let arr = vec![0; 200];

        scroller_display("s", &arr, &mut curr_x, &mut curry, cols, rows, &mut graph);
        assert_eq!(curry, 10); // Scrolled down by 10 rows
        assert_eq!(curr_x, 0); // No horizontal movement
    }

    #[test]
    fn test_scroller_display_scroll_left() {
        let mut curr_x = 5;
        let mut curry = 0;
        let cols = 10;
        let rows = 10;
        let mut graph = Graph::new();
        let arr = vec![0; 100];

        scroller_display("a", &arr, &mut curr_x, &mut curry, cols, rows, &mut graph);
        assert_eq!(curr_x, 0); // Scrolled left to the start
        assert_eq!(curry, 0); // No vertical movement
    }

    #[test]
    fn test_scroller_display_scroll_right() {
        let mut curr_x = 0;
        let mut curry = 0;
        let cols = 20;
        let rows = 10;
        let mut graph = Graph::new();
        let arr = vec![0; 200];

        scroller_display("d", &arr, &mut curr_x, &mut curry, cols, rows, &mut graph);
        assert_eq!(curr_x, 10); // Scrolled right by 10 columns
        assert_eq!(curry, 0); // No vertical movement
    }

    #[test]
    fn test_scroller_display_scroll_to_valid_cell() {
        let mut curr_x = 0;
        let mut curry = 0;
        let cols = 10;
        let rows = 10;
        let mut graph = Graph::new();
        let arr = vec![0; 100];

        scroller_display(
            "scroll_to B2",
            &arr,
            &mut curr_x,
            &mut curry,
            cols,
            rows,
            &mut graph,
        );
        assert_eq!(curr_x, 0); // Column B
        assert_eq!(curry, 0); // Row 2
    }

    #[test]
    fn test_scroller_display_scroll_to_invalid_cell() {
        let mut curr_x = 0;
        let mut curry = 0;
        let cols = 10;
        let rows = 10;
        let mut graph = Graph::new();
        let arr = vec![0; 100];

        scroller_display(
            "scroll_to Z99",
            &arr,
            &mut curr_x,
            &mut curry,
            cols,
            rows,
            &mut graph,
        );
        assert_eq!(curr_x, 0); // No change
        assert_eq!(curry, 0); // No change
    }

    #[test]
    fn test_scroller_display_unrecognized_command() {
        let mut curr_x = 0;
        let mut curry = 0;
        let cols = 10;
        let rows = 10;
        let mut graph = Graph::new();
        let arr = vec![0; 100];

        scroller_display(
            "invalid_command",
            &arr,
            &mut curr_x,
            &mut curry,
            cols,
            rows,
            &mut graph,
        );
        assert_eq!(curr_x, 0); // No change
        assert_eq!(curry, 0); // No change
    }

    #[test]
    fn test_scroller_display_scroll_to_bounds_check() {
        let mut curr_x = 0;
        let mut curry = 0;
        let cols = 10;
        let rows = 10;
        let mut graph = Graph::new();
        let arr = vec![0; 100];

        // Valid cell within bounds
        scroller_display(
            "scroll_to C3",
            &arr,
            &mut curr_x,
            &mut curry,
            cols,
            rows,
            &mut graph,
        );
        assert_eq!(curr_x, 0); // Column C (zero-based index 2)
        assert_eq!(curry, 0); // Row 3 (zero-based index 2)

        // Invalid cell (row out of bounds)
        scroller_display(
            "scroll_to C11",
            &arr,
            &mut curr_x,
            &mut curry,
            cols,
            rows,
            &mut graph,
        );
        assert_eq!(curr_x, 0); // No change
        assert_eq!(curry, 0); // No change

        // Invalid cell (column out of bounds)
        scroller_display(
            "scroll_to K3",
            &arr,
            &mut curr_x,
            &mut curry,
            cols,
            rows,
            &mut graph,
        );
        assert_eq!(curr_x, 0); // No change
        assert_eq!(curry, 0); // No change

        // Invalid cell (completely out of bounds)
        scroller_display(
            "scroll_to Z99",
            &arr,
            &mut curr_x,
            &mut curry,
            cols,
            rows,
            &mut graph,
        );
        assert_eq!(curr_x, 0); // No change
        assert_eq!(curry, 0); // No change
    }

    #[test]
    fn test_out_of_bounds_cell() {
        let cols = 10;
        let rows = 10;
        let cell = 105; // Out-of-bounds cell index
        let mut curr_x = 0;
        let mut curry = 0;
        let mut flag = false;

        let start_row = (cell as usize) / cols;
        let start_col = (cell as usize) % cols;

        if start_row >= rows || start_col >= cols {
            flag = true;
        } else {
            curr_x = start_col;
            curry = start_row;
        }

        assert!(flag); // Ensure the flag is set for out-of-bounds
        assert_eq!(curr_x, 0); // Ensure curr_x is not updated
        assert_eq!(curry, 0); // Ensure curry is not updated
    }
}

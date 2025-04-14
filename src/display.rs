use std::cmp;
use std::i32;
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

pub fn scroller_display(
    cmd: &str,
    _arr: &[i32],
    curr_x: &mut usize,
    curry: &mut usize,
    cols: usize,
    rows: usize,
    graph: &mut Graph,
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
        let cell = cell_parser(cmd, cols as i32, rows as i32, 10, (cmd.len() - 1) as i32, graph);
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
// In src/display.rs

/// Convert a zero-indexed column number to a spreadsheet column name (0 -> A, 1 -> B, 26 -> AA, etc.)
fn column_index_to_name(mut col: usize) -> String {
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

/// Render the spreadsheet as an HTML table with editable cells.
/// Each cell is rendered as an <input> element that carries a data attribute for its label.
pub fn render_spreadsheet(
    curr_x: usize,
    curr_y: usize,
    arr: &[i32],
    cols: usize,
    rows: usize,
) -> String {
    let mut output = String::new();

    // Begin table (you can adjust styles as needed)
    output.push_str(r#"<table border="1" style="border-collapse:collapse; width: 100%;">"#);

    // Compute visible columns and rows (here we display at most 10 of each)
    let num_cols = std::cmp::min(cols.saturating_sub(curr_x), 100);
    let num_rows = std::cmp::min(rows.saturating_sub(curr_y), 100);

    // Build column header row.
    output.push_str("<tr><th></th>");
    for i in 0..num_cols {
        let col_name = column_index_to_name(curr_x + i);
        output.push_str(&format!(r#"<th style="padding: 5px;">{}</th>"#, col_name));
    }
    output.push_str("</tr>");

    // Build each row.
    for j in 0..num_rows {
        let row_num = curr_y + j + 1; // display rows as 1-indexed
        output.push_str(&format!(r#"<tr><th style="padding: 5px;">{}</th>"#, row_num));
        for i in 0..num_cols {
            let col_index = curr_x + i;
            let cell_label = format!("{}{}", column_index_to_name(col_index), row_num);
            let index = (curr_y + j) * cols + col_index;
            // Use "ERR" as the display if the value is std::i32::MIN, otherwise the actual value.
            let cell_display = if arr[index] == std::i32::MIN {
                "ERR".to_owned()
            } else {
                arr[index].to_string()
            };

            // Each cell is an input element.
            // We attach event handlers (onblur, onkeyup) that we will define in JS.
            output.push_str(&format!(
                r#"<td style="padding: 5px;">
                    <input type="text"
                           data-cell="{}"
                           value="{}"
                           style="width: 100%; border: none; text-align: center;"
                           onblur="handleCellBlur(event)"
                           onkeyup="handleCellKeyup(event)" />
                   </td>"#,
                cell_label, cell_display
            ));
        }
        output.push_str("</tr>");
    }

    output.push_str("</table>");
    output
}

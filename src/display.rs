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
pub fn render_spreadsheet(curr_x: usize, curry: usize, arr: &[i32], cols: usize, rows: usize) -> String {
    let mut output = String::new();
    
    // Add a debug prefix so you can see something in the console output.
    output.push_str("DEBUG: Rendering Spreadsheet\n\n");

    // If your code is in any way returning early, or is empty, you won’t see anything.
    // Make sure you have logic like this:
    let num_cols = std::cmp::min(cols.saturating_sub(curr_x), 10);
    let num_rows = std::cmp::min(rows.saturating_sub(curry), 10);

    // Build column headers
    output.push_str("      ");
    for i in 0..num_cols {
        let mut val = (curr_x + i + 1) as i32; // 1-indexed
        let mut col_str = String::new();
        while val > 0 {
            val -= 1;
            let letter = ((val % 26) as u8 + b'A') as char;
            col_str.push(letter);
            val /= 26;
        }
        let header: String = col_str.chars().rev().collect();
        output.push_str(&format!("{:<10}", header));
    }
    output.push('\n');

    // Build row lines
    for j in 0..num_rows {
        output.push_str(&format!("{:<3}   ", curry + j + 1));
        for i in 0..num_cols {
            let index = (curry + j) * cols + (curr_x + i);
            let value = arr[index];
            if value == std::i32::MIN {
                output.push_str(&format!("{:<10}", "ERR"));
            } else {
                output.push_str(&format!("{:<10}", value));
            }
        }
        output.push('\n');
    }

    output
}
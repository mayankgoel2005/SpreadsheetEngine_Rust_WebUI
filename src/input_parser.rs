use std::time::Instant;
use crate::spreadsheet::Spreadsheet;
use crate::scrolling::scroll_to;

fn is_number(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_digit(10))
}

fn column_to_index(col: &str) -> Option<usize> {
    let mut index = 0;
    for c in col.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        index = index * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
    }
    Some(index)
}

pub fn parse_input(input: &str, spreadsheet: &mut Spreadsheet, start: Instant) -> bool {
    let input = input.trim();

    match input {
        "q" => {
            let cpu_time_used = start.elapsed().as_secs_f64();
            spreadsheet.time = cpu_time_used;
            return false;
        }
        "disable_output" => {
            spreadsheet.time = start.elapsed().as_secs_f64();
            spreadsheet.display = true;
            println!("[{:.1}] (ok)", spreadsheet.time);
        }
        "enable_output" => {
            spreadsheet.time = start.elapsed().as_secs_f64();
            spreadsheet.display = false;
            spreadsheet.print();
            println!("[{:.1}] (ok)", spreadsheet.time);
        }
        "w" | "s" | "a" | "d" => {
            // These commands are handled elsewhere (e.g., in the scrolling module).
        }
        _ if input.starts_with("scroll_to ") => {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() != 2 {
                spreadsheet.time = start.elapsed().as_secs_f64();
                println!("[{:.1}] (Error)", spreadsheet.time);
                return true;
            }
            let cell_ref = parts[1];
            let split_index = cell_ref.chars().take_while(|c| c.is_alphabetic()).count();
            let (col_part, row_part) = cell_ref.split_at(split_index);
            if col_part.is_empty() || col_part.len() > 3 || !is_number(row_part) {
                spreadsheet.time = start.elapsed().as_secs_f64();
                println!("[{:.1}] (Error)", spreadsheet.time);
                return true;
            }
            let col = match column_to_index(col_part) {
                Some(c) => c,
                None => {
                    spreadsheet.time = start.elapsed().as_secs_f64();
                    println!("[{:.1}] (Error)", spreadsheet.time);
                    return true;
                }
            };
            let row: usize = row_part.parse().unwrap();
            if row == 0 || col == 0 || row > spreadsheet.rows || col > spreadsheet.cols {
                spreadsheet.time = start.elapsed().as_secs_f64();
                println!("[{:.1}] (Error)", spreadsheet.time);
                return true;
            }
            scroll_to(spreadsheet, row, col);
            spreadsheet.time = start.elapsed().as_secs_f64();
            spreadsheet.print();
            println!("[{:.1}] (ok)", spreadsheet.time);
        }
        _ => {
            // Inline processing for operations (formula or cell update)
            // Replace the code below with your actual operation parsing logic.
            println!("Processing operation: {}", input);
            // For example, you might update a cell value, recalculate dependencies, etc.
        }
    }
    true
}

pub fn cell_parser(
    a: &str,
    cols: i32,
    rows: i32,
    start: usize,
    end: usize,
) -> i32 {
    let s = &a[start..=end];
    let mut col_part = String::new();
    let mut row_part = String::new();
    for c in s.chars() {
        if c.is_alphabetic() {
            col_part.push(c);
        } else if c.is_digit(10) {
            row_part.push(c);
        } else {
            return -1;
        }
    }
    if col_part.is_empty() || row_part.is_empty() {
        return -1;
    }
    let mut col = 0;
    for c in col_part.chars() {
        col = col * 26 + (c.to_ascii_uppercase() as i32 - 'A' as i32 + 1);
    }
    col -= 1;
    let row: i32 = match row_part.parse::<i32>() {
        Ok(num) => num - 1,
        Err(_) => return -1,
    };
    if col < 0 || row < 0 || col >= cols || row >= rows {
        -1
    } else {
        row * cols + col
    }
}

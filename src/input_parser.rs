use std::io::{self, Write};
use std::time::Instant;
use std::collections::HashMap;
use crate::spreadsheet::Spreadsheet;
use crate::scrolling::{scroll_up, scroll_down, scroll_left, scroll_right, scroll_to};
use crate::input_parser::handle_operation;

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
        },
        "disable_output" => {
            let cpu_time_used = start.elapsed().as_secs_f64();
            spreadsheet.time = cpu_time_used;
            spreadsheet.display = true;
            println!("[{:.1}] (ok) ", spreadsheet.time);
        },
        "enable_output" => {
            let cpu_time_used = start.elapsed().as_secs_f64();
            spreadsheet.time = cpu_time_used;
            spreadsheet.display = false;
            spreadsheet.print();
            println!("[{:.1}] (ok) ", spreadsheet.time);
        },
        "w" => {
            scroll_up(spreadsheet);
            let cpu_time_used = start.elapsed().as_secs_f64();
            spreadsheet.time = cpu_time_used;
            spreadsheet.print();
            println!("[{:.1}] (ok) ", spreadsheet.time);
        },
        "s" => {
            scroll_down(spreadsheet);
            let cpu_time_used = start.elapsed().as_secs_f64();
            spreadsheet.time = cpu_time_used;
            spreadsheet.print();
            println!("[{:.1}] (ok) ", spreadsheet.time);
        },
        "a" => {
            scroll_left(spreadsheet);
            let cpu_time_used = start.elapsed().as_secs_f64();
            spreadsheet.time = cpu_time_used;
            spreadsheet.print();
            println!("[{:.1}] (ok) ", spreadsheet.time);
        },
        "d" => {
            scroll_right(spreadsheet);
            let cpu_time_used = start.elapsed().as_secs_f64();
            spreadsheet.time = cpu_time_used;
            spreadsheet.print();
            println!("[{:.1}] (ok) ", spreadsheet.time);
        },
        _ if input.starts_with("scroll_to ") => {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() != 2 {
                let cpu_time_used = start.elapsed().as_secs_f64();
                spreadsheet.time = cpu_time_used;
                println!("[{:.1}] (Error) ", spreadsheet.time);
                return true;
            }
            let cell_ref = parts[1];
            let (col_part, row_part) = cell_ref.split_at(cell_ref.chars().take_while(|c| c.is_alphabetic()).count());
            if col_part.is_empty() || col_part.len() > 3 || !is_number(row_part) {
                let cpu_time_used = start.elapsed().as_secs_f64();
                spreadsheet.time = cpu_time_used;
                println!("[{:.1}] (Error) ", spreadsheet.time);
                return true;
            }
            let col = match column_to_index(col_part) {
                Some(c) => c,
                None => {
                    let cpu_time_used = start.elapsed().as_secs_f64();
                    spreadsheet.time = cpu_time_used;
                    println!("[{:.1}] (Error) ", spreadsheet.time);
                    return true;
                }
            };
            let row: usize = row_part.parse().unwrap();
            if row > spreadsheet.rows || col > spreadsheet.cols || row == 0 || col == 0 {
                let cpu_time_used = start.elapsed().as_secs_f64();
                spreadsheet.time = cpu_time_used;
                println!("[{:.1}] (Error) ", spreadsheet.time);
                return true;
            }
            scroll_to(spreadsheet, row, col);
            let cpu_time_used = start.elapsed().as_secs_f64();
            spreadsheet.time = cpu_time_used;
            spreadsheet.print();
            println!("[{:.1}] (ok) ", spreadsheet.time);
        },
        _ => handle_operation(input, spreadsheet, start),
    }
    true
}
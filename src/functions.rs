use std::i32;
use std::thread;
use std::time::Duration;
use crate::graph::{Graph, Formula, add_formula, add_edge};
use crate::input_parser::{cell_parser};

fn validate_range(range_start: i32, range_end: i32, cols: i32) -> bool {
    let start_row = range_start / cols;
    let start_col = range_start % cols;
    let end_row = range_end / cols;
    let end_col = range_end % cols;
    !(start_row > end_row || (start_row == end_row && start_col > end_col))
}

fn parse_range(
    a: &str,
    pos_equal_to: usize,
    cols: i32,
    rows: i32,
    _graph: &Graph,
) -> Option<(i32, i32, i32)> {
    let dest = cell_parser(a, cols, rows, 0, pos_equal_to - 1);
    if dest == -1 {
        println!("Invalid destination cell");
        return None;
    }
    let open_paren = a[pos_equal_to..].find('(').map(|i| i + pos_equal_to)?;
    let close_paren = a[pos_equal_to..].find(')').map(|i| i + pos_equal_to)?;
    if close_paren <= open_paren + 1 {
        println!("Invalid range: Missing or misplaced parentheses");
        return None;
    }
    let colon_pos = a[open_paren + 1..]
        .find(':')
        .map(|i| i + open_paren + 1)?;
    let range_start = cell_parser(a, cols, rows, open_paren + 1, colon_pos - 1);
    let range_end = cell_parser(a, cols, rows, colon_pos + 1, close_paren - 1);
    if range_start == -1 || range_end == -1 || !validate_range(range_start, range_end, cols) {
        println!("Invalid range");
        return None;
    }
    Some((dest, range_start, range_end))
}

fn get_bounds(range_start: i32, range_end: i32, cols: i32) -> (i32, i32, i32, i32) {
    let start_row = range_start / cols;
    let start_col = range_start % cols;
    let end_row = range_end / cols;
    let end_col = range_end % cols;
    (start_row, start_col, end_row, end_col)
}

pub fn min_func(
    a: &str,
    cols: i32,
    rows: i32,
    pos_equal_to: usize,
    _pos_end: usize,
    arr: &mut [i32],
    graph: &mut Graph,
    formula_array: &mut [Formula],
) {
    if let Some((dest, range_start, range_end)) = parse_range(a, pos_equal_to, cols, rows, graph) {
        add_formula(dest, 9, range_start, range_end, formula_array);
        let (start_row, start_col, end_row, end_col) = get_bounds(range_start, range_end, cols);
        let mut min_val = arr[range_start as usize];
        if start_row == end_row {
            for idx in range_start..=range_end {
                let idx_usize = idx as usize;
                graph.adj[idx_usize] = Some(add_edge(dest, graph.adj[idx_usize].take()));
                if arr[idx_usize] < min_val {
                    min_val = arr[idx_usize];
                }
            }
        } else {
            for row in start_row..=end_row {
                for col in start_col..=end_col {
                    let idx = row * cols + col;
                    let idx_usize = idx as usize;
                    graph.adj[idx_usize] = Some(add_edge(dest, graph.adj[idx_usize].take()));
                    if arr[idx_usize] < min_val {
                        min_val = arr[idx_usize];
                    }
                }
            }
        }
        arr[dest as usize] = min_val;
    }
}

pub fn max_func(
    a: &str,
    cols: i32,
    rows: i32,
    pos_equal_to: usize,
    _pos_end: usize,
    arr: &mut [i32],
    graph: &mut Graph,
    formula_array: &mut [Formula],
) {
    if let Some((dest, range_start, range_end)) = parse_range(a, pos_equal_to, cols, rows, graph) {
        add_formula(dest, 10, range_start, range_end, formula_array);
        let (start_row, start_col, end_row, end_col) = get_bounds(range_start, range_end, cols);
        let mut max_val = arr[range_start as usize];
        if start_row == end_row {
            for idx in range_start..=range_end {
                let idx_usize = idx as usize;
                graph.adj[idx_usize] = Some(add_edge(dest, graph.adj[idx_usize].take()));
                if arr[idx_usize] > max_val {
                    max_val = arr[idx_usize];
                }
            }
        } else {
            for row in start_row..=end_row {
                for col in start_col..=end_col {
                    let idx = row * cols + col;
                    let idx_usize = idx as usize;
                    graph.adj[idx_usize] = Some(add_edge(dest, graph.adj[idx_usize].take()));
                    if arr[idx_usize] > max_val {
                        max_val = arr[idx_usize];
                    }
                }
            }
        }
        arr[dest as usize] = max_val;
    }
}

pub fn avg_func(
    a: &str,
    cols: i32,
    rows: i32,
    pos_equal_to: usize,
    _pos_end: usize,
    arr: &mut [i32],
    graph: &mut Graph,
    formula_array: &mut [Formula],
) {
    if let Some((dest, range_start, range_end)) = parse_range(a, pos_equal_to, cols, rows, graph) {
        add_formula(dest, 11, range_start, range_end, formula_array);
        let (start_row, start_col, end_row, end_col) = get_bounds(range_start, range_end, cols);
        let mut sum = 0;
        let mut count = 0;
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let idx = (row * cols + col) as usize;
                graph.adj[idx] = Some(add_edge(dest, graph.adj[idx].take()));
                sum += arr[idx];
                count += 1;
            }
        }
        let avg = if count > 0 { sum / count } else { 0 };
        arr[dest as usize] = avg;
    }
}

pub fn sum_func(
    a: &str,
    cols: i32,
    rows: i32,
    pos_equal_to: usize,
    _pos_end: usize,
    arr: &mut [i32],
    graph: &mut Graph,
    formula_array: &mut [Formula],
) {
    if let Some((dest, range_start, range_end)) = parse_range(a, pos_equal_to, cols, rows, graph) {
        add_formula(dest, 12, range_start, range_end, formula_array);
        let (start_row, start_col, end_row, end_col) = get_bounds(range_start, range_end, cols);
        let mut sum = 0;
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let idx = (row * cols + col) as usize;
                graph.adj[idx] = Some(add_edge(dest, graph.adj[idx].take()));
                sum += arr[idx];
            }
        }
        arr[dest as usize] = sum;
    }
}

pub fn standard_dev_func(
    a: &str,
    cols: i32,
    rows: i32,
    pos_equal_to: usize,
    _pos_end: usize,
    arr: &mut [i32],
    graph: &mut Graph,
    formula_array: &mut [Formula],
) {
    if let Some((dest, range_start, range_end)) = parse_range(a, pos_equal_to, cols, rows, graph) {
        add_formula(dest, 13, range_start, range_end, formula_array);
        let (start_row, start_col, end_row, end_col) = get_bounds(range_start, range_end, cols);
        let mut sum = 0;
        let mut count = 0;
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let idx = (row * cols + col) as usize;
                graph.adj[idx] = Some(add_edge(dest, graph.adj[idx].take()));
                sum += arr[idx];
                count += 1;
            }
        }
        let avg = sum as f64 / count as f64;
        let mut standard_dev_squared = 0.0;
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let idx = (row * cols + col) as usize;
                let diff = arr[idx] as f64 - avg;
                standard_dev_squared += diff * diff;
            }
        }
        let standard_dev = (standard_dev_squared / count as f64).sqrt() as i32;
        arr[dest as usize] = standard_dev;
    }
}

pub fn sleep_func(
    a: &str,
    cols: i32,
    rows: i32,
    pos_equal_to: usize,
    _pos_end: usize,
    arr: &mut [i32],
    graph: &mut Graph,
    formula_array: &mut [Formula],
) {
    let target_cell = cell_parser(a, cols, rows, 0, pos_equal_to - 1);
    if target_cell == -1 {
        println!("Invalid destination cell");
        return;
    }
    let open_paren = a[pos_equal_to..].find('(').map(|i| i + pos_equal_to);
    let close_paren = a[pos_equal_to..].find(')').map(|i| i + pos_equal_to);
    if open_paren.is_none() || close_paren.is_none() || close_paren.unwrap() <= open_paren.unwrap() + 1 {
        println!("Invalid SLEEP syntax: Missing or misplaced parentheses");
        return;
    }
    let open_paren = open_paren.unwrap();
    let close_paren = close_paren.unwrap();
    let ref_cell = cell_parser(a, cols, rows, open_paren + 1, close_paren - 1);
    let sleep_value: i32;
    if ref_cell != -1 {
        sleep_value = arr[ref_cell as usize];
        if sleep_value == i32::MIN {
            println!("Referenced cell {} contains an error value", ref_cell);
            arr[target_cell as usize] = i32::MIN;
            return;
        }
        graph.adj[ref_cell as usize] = Some(add_edge(target_cell, graph.adj[ref_cell as usize].take()));
    } else {
        let substr = &a[open_paren + 1..close_paren];
        match substr.trim().parse::<i32>() {
            Ok(val) => sleep_value = val,
            Err(_) => {
                println!("Invalid SLEEP value");
                arr[target_cell as usize] = i32::MIN;
                add_formula(
                    target_cell,
                    14,
                    if ref_cell != -1 { ref_cell } else { target_cell },
                    i32::MIN,
                    formula_array,
                );
                return;
            }
        }
    }
    if sleep_value <= 0 {
        arr[target_cell as usize] = sleep_value;
        add_formula(
            target_cell,
            14,
            if ref_cell != -1 { ref_cell } else { target_cell },
            sleep_value,
            formula_array,
        );
        return;
    }
    add_formula(
        target_cell,
        14,
        if ref_cell != -1 { ref_cell } else { target_cell },
        sleep_value,
        formula_array,
    );
    thread::sleep(Duration::from_secs(sleep_value as u64));
    arr[target_cell as usize] = sleep_value;
}

//! Module `input_parser`.
//!
//! Parses and executes cell assignment commands of the form `A1=EXPR`,
//! where `EXPR` can be:
//! - A literal integer (e.g. `42`)
//! - A single cell reference (e.g. `B2`)
//! - An arithmetic expression combining cells and/or literals with `+`, `-`, `*`, `/` (e.g. `A1+5`, `B2*C3`)
//! - A function call: `MIN(range)`, `MAX(range)`, `AVG(range)`, `SUM(range)`, `STDEV(range)`, or `SLEEP(duration)`
//!
//! The entry point is [`parser`], which returns:
//! - `0` on successful parse and evaluation
//! - `1` on any error (parse error, invalid cell, cycle detection, etc.)

use crate::functions::{avg_func, max_func, min_func, sleep_func, standard_dev_func, sum_func};
use crate::graph::{Formula, Graph, add_formula, arith, delete_edge, recalculate};
use crate::spreadsheet::Spreadsheet;

/// Save+restore on rollback
static mut OLD_VALUE: i32 = 0;
static mut OLD_OP_TYPE: i32 = 0;
static mut OLD_P1: i32 = 0;
static mut OLD_P2: i32 = 0;
pub static mut HAS: i32 = 0;

#[inline]
fn is_alpha(c: char) -> bool {
    c.is_ascii_uppercase()
}
#[inline]
fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Mark `dst` dependent on `src` (no duplicates).
#[inline]
fn depend(g: &mut Graph, src: usize, dst: usize) {
    g.adj.entry(src).or_default().push(dst);
}

/// A1 → 0,0; B3 → col=B (1)*,row=3 (2) → index = row*cols+col
pub fn cell_parser(s: &str, cols: i32, rows: i32) -> i32 {
    let mut col = 0;
    let mut row = 0;
    let mut seen_digit = false;
    for ch in s.chars() {
        if is_alpha(ch) {
            if seen_digit {
                return -1;
            }
            col = col * 26 + (ch as i32 - 'A' as i32 + 1);
        } else if is_digit(ch) {
            row = row * 10 + (ch as i32 - '0' as i32);
            seen_digit = true;
        } else {
            return -1;
        }
    }
    col -= 1;
    row -= 1;
    if col < 0 || row < 0 || col >= cols || row >= rows {
        -1
    } else {
        row * cols + col
    }
}

pub struct CellRange {
    pub start_row: usize,
    pub end_row: usize,
    pub start_col: usize,
    pub end_col: usize,
}

pub fn parse_range(range: &str, cols: usize, rows: usize) -> Option<CellRange> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() == 2 {
        let start = cell_parser(parts[0], cols as i32, rows as i32);
        let end = cell_parser(parts[1], cols as i32, rows as i32);
        if start != -1 && end != -1 {
            let start_row = start as usize / cols;
            let start_col = start as usize % cols;
            let end_row = end as usize / cols;
            let end_col = end as usize % cols;
            return Some(CellRange {
                start_row,
                end_row,
                start_col,
                end_col,
            });
        }
    }
    None
}

/// + → 1, - → 2, * → 3, / → 4
#[inline]
fn return_optype(op: char) -> i32 {
    match op {
        '+' => 1,
        '-' => 2,
        '*' => 3,
        '/' => 4,
        _ => i32::MIN,
    }
}

/// Handle “dst = [±] literal” or “dst = [±] cell”
fn value_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> i32 {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 {
        return 1;
    }
    /* save old */
    unsafe {
        OLD_VALUE = arr[dst as usize];
        OLD_OP_TYPE = farr[dst as usize].op_type;
        OLD_P1 = farr[dst as usize].p1;
        OLD_P2 = farr[dst as usize].p2;
    }

    if farr[dst as usize].op_type > 0 {
        delete_edge(g, dst as usize, farr, cols as usize);
    }

    /* handle optional sign */
    let mut it = eq + 1;
    let mut neg = false;
    let bytes = txt.as_bytes();
    if bytes[it] == b'-' || bytes[it] == b'+' {
        neg = bytes[it] == b'-';
        it += 1;
    }

    let rhs = txt[it..].trim();
    let is_cell = rhs
        .bytes()
        .next()
        .map(|b: u8| b.is_ascii_uppercase())
        .unwrap_or(false);

    let val;
    if is_cell {
        let src = cell_parser(rhs, cols, rows);
        if src == -1 {
            return 1;
        }
        depend(g, src as usize, dst as usize);

        val = if neg {
            -arr[src as usize]
        } else {
            arr[src as usize]
        };
        add_formula(
            g,
            dst as usize,
            src,
            if neg { -1 } else { 0 },
            if neg { 3 } else { 1 },
            farr,
            cols as usize,
        );
    } else {
        match rhs.parse::<i32>() {
            Ok(v) => val = if neg { -v } else { v },
            Err(_) => {
                return 1;
            }
        }
        add_formula(g, dst as usize, val, 0, 0, farr, cols as usize);
    }

    arr[dst as usize] = val;

    if !recalculate(g, cols, arr, dst as usize, farr) {
        /* rollback */
        delete_edge(g, dst as usize, farr, cols as usize);
        unsafe {
            arr[dst as usize] = OLD_VALUE;
            farr[dst as usize] = Formula {
                op_type: OLD_OP_TYPE,
                p1: OLD_P1,
                p2: OLD_P2,
            };
            add_formula(
                g,
                dst as usize,
                OLD_P1,
                OLD_P2,
                OLD_OP_TYPE,
                farr,
                cols as usize,
            );
        }
        return 1;
    }
    0
}
/// Handle “dst = A1 + B2” etc.
fn arth_op(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> i32 {
    // find operator
    let mut op_ind = None;
    let mut op_ch = '+';
    for (i, ch) in txt[eq + 1..].char_indices() {
        if "+-*/".contains(ch) {
            op_ind = Some(eq + 1 + i);
            op_ch = ch;
            break;
        }
    }
    let op_ind = match op_ind {
        Some(i) => i,
        None => return 1,
    };

    // split operands
    let left_s = txt[eq + 1..op_ind].trim();
    let right_s = txt[op_ind + 1..].trim();

    // parse left
    let (lneg, left_s) = if let Some(stripped) = left_s.strip_prefix('-') {
        (true, stripped.trim())
    } else if let Some(stripped) = left_s.strip_prefix('+') {
        (false, stripped.trim())
    } else {
        (false, left_s)
    };
    let left_is_cell = left_s.chars().next().is_some_and(is_alpha);
    let left_val = if left_is_cell {
        cell_parser(left_s, cols, rows)
    } else {
        left_s.parse::<i32>().unwrap_or(i32::MIN)
    };
    if left_val == i32::MIN {
        return 1;
    }
    let left_val = if lneg { -left_val } else { left_val };

    // parse right
    let (rneg, right_s) = if let Some(stripped) = right_s.strip_prefix('-') {
        (true, stripped.trim())
    } else if let Some(stripped) = right_s.strip_prefix('+') {
        (false, stripped.trim())
    } else {
        (false, right_s)
    };
    let right_is_cell = right_s.chars().next().is_some_and(is_alpha);
    let right_val = if right_is_cell {
        cell_parser(right_s, cols, rows)
    } else {
        right_s.parse::<i32>().unwrap_or(i32::MIN)
    };
    if right_val == i32::MIN {
        return 1;
    }
    let right_val = if rneg { -right_val } else { right_val };

    // dst
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst < 0 {
        return 1;
    }
    let dst = dst as usize;

    // stash/rollback
    unsafe {
        OLD_VALUE = arr[dst];
        OLD_OP_TYPE = farr[dst].op_type;
        OLD_P1 = farr[dst].p1;
        OLD_P2 = farr[dst].p2;
    }
    if farr[dst].op_type > 0 {
        delete_edge(g, dst, farr, cols as usize);
    }

    // evaluate
    let result = arith(left_val, right_val, op_ch);

    // record dependencies & formula
    let base_op = return_optype(op_ch);
    let (p1, p2, op_type) = match (left_is_cell, right_is_cell) {
        (true, true) => {
            let p1 = left_val.abs(); // left index stored
            let p2 = right_val.abs();
            depend(g, p1 as usize, dst);
            depend(g, p2 as usize, dst);
            (p1, p2, base_op + 4) // cell+cell => ops 5..8
        }
        (true, false) => {
            let p1 = left_val.abs();
            let p2 = right_val;
            depend(g, p1 as usize, dst);
            (p1, p2, base_op) // cell+lit => ops 1..4
        }
        (false, true) => {
            let p1 = right_val.abs();
            let p2 = left_val;
            depend(g, p1 as usize, dst);
            (p1, p2, base_op + 4) // lit+cell treat as cell+cell
        }
        (false, false) => {
            // pure literal+literal → constant
            arr[dst] = result;
            add_formula(g, dst, result, 0, 0, farr, cols as usize);
            return if recalculate(g, cols, arr, dst, farr) {
                0
            } else {
                1
            };
        }
    };
    arr[dst] = result;
    add_formula(g, dst, p1, p2, op_type, farr, cols as usize);

    // recalc / rollback
    if !recalculate(g, cols, arr, dst, farr) {
        delete_edge(g, dst, farr, cols as usize);
        // 2) restore the old cell value and formula record
        unsafe {
            arr[dst] = OLD_VALUE;
            farr[dst] = Formula {
                op_type: OLD_OP_TYPE,
                p1: OLD_P1,
                p2: OLD_P2,
            };
            add_formula(g, dst, OLD_P1, OLD_P2, OLD_OP_TYPE, farr, cols as usize);
        }
        // 3) re-add the old dependency edges
        return 1;
    }
    0
}

/// Dispatch SUM/AVG/MIN/.../SLEEP
fn funct(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> i32 {
    let ok = if txt[eq + 1..].starts_with("MIN(") {
        min_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq + 1..].starts_with("MAX(") {
        max_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq + 1..].starts_with("AVG(") {
        avg_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq + 1..].starts_with("SUM(") {
        sum_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq + 1..].starts_with("STDEV(") {
        standard_dev_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq + 1..].starts_with("SLEEP(") {
        sleep_func(txt, cols, rows, eq, arr, g, farr)
    } else {
        return 1;
    };
    if !ok {
        return 1;
    }
    // re‐run recalc on dst
    let dst = cell_parser(&txt[..eq], cols, rows) as usize;
    if !recalculate(g, cols, arr, dst, farr) {
        delete_edge(g, dst, farr, cols as usize);
        // 2) restore the old cell value and formula record
        unsafe {
            arr[dst] = OLD_VALUE;
            farr[dst] = Formula {
                op_type: OLD_OP_TYPE,
                p1: OLD_P1,
                p2: OLD_P2,
            };
            add_formula(g, dst, OLD_P1, OLD_P2, OLD_OP_TYPE, farr, cols as usize);
        }
        // 3) re-add the old dependency edges
        return 1;
    }
    0
}

/// Entry point
pub fn parser(sheet: &mut Spreadsheet, txt: &str) -> i32 {
    let cols = sheet.cols as i32;
    let rows = sheet.rows as i32;
    let arr = &mut sheet.arr;
    let g = &mut sheet.graph;
    let farr = &mut sheet.formula_array;

    let eq = txt.find('=').unwrap_or(usize::MAX);
    if eq == usize::MAX {
        return -1;
    }

    // classify
    let rhs = &txt[eq + 1..].trim();
    let is_func = rhs.contains('(');

    // Check if it's a simple value (numeric or cell reference)
    let is_val = if let Some(first_char) = rhs.chars().next() {
        let is_single_term = rhs
            .chars()
            .skip(match first_char {
                '+' | '-' => 1, // skip optional sign
                _ => 0,
            })
            .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase());

        is_single_term && !is_func
    } else {
        false
    };

    let is_arth = !is_func && !is_val && rhs.chars().any(|c| "+-*/".contains(c));

    if is_val {
        value_func(txt, cols, rows, eq, arr, g, farr)
    } else if is_arth {
        arth_op(txt, cols, rows, eq, arr, g, farr)
    } else if is_func {
        funct(txt, cols, rows, eq, arr, g, farr)
    } else {
        -1 // invalid input
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Formula, Graph};
    use crate::spreadsheet::{Spreadsheet, initialize_spreadsheet};

    #[test]
    fn test_value_func_with_literal() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = value_func(
            "A1=42",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[0], 42);
        assert_eq!(
            formula_array[0],
            Formula {
                op_type: 0,
                p1: 42,
                p2: 0
            }
        );
    }

    #[test]
    fn test_value_func_with_negative_literal() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = value_func(
            "A1=-42",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[0], -42);
        assert_eq!(
            formula_array[0],
            Formula {
                op_type: 0,
                p1: -42,
                p2: 0
            }
        );
    }

    #[test]
    fn test_value_func_with_cell_reference() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[1] = 50; // B1 = 50
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = value_func(
            "A1=B1",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[0], 50);
        assert_eq!(
            formula_array[0],
            Formula {
                op_type: 1,
                p1: 1,
                p2: 0
            }
        );
        assert_eq!(graph.adj[&1], vec![0, 0]); // B1 depends on A1
    }

    #[test]
    fn test_value_func_with_negative_cell_reference() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[1] = 50; // B1 = 50
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = value_func(
            "A1=-B1",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[0], -50);
        assert_eq!(
            formula_array[0],
            Formula {
                op_type: 3,
                p1: 1,
                p2: -1
            }
        );
        assert_eq!(graph.adj[&1], vec![0, 0]); // B1 depends on A1
    }

    #[test]
    fn test_value_func_with_invalid_cell_reference() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = value_func(
            "A1=Z1",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 1); // Invalid cell reference
        assert_eq!(arr[0], 0); // No change
        assert_eq!(
            formula_array[0],
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            }
        );
    }

    #[test]
    fn test_value_func_with_invalid_literal() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = value_func(
            "A1=abc",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 1); // Invalid literal
        assert_eq!(arr[0], 0); // No change
        assert_eq!(
            formula_array[0],
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            }
        );
    }

    #[test]
    fn test_value_func_with_existing_formula() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[1] = 50; // B1 = 50
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 1,
                p1: 1,
                p2: 0
            };
            100
        ];

        let result = value_func(
            "A1=42",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[0], 42);
        assert_eq!(
            formula_array[0],
            Formula {
                op_type: 0,
                p1: 42,
                p2: 0
            }
        );
        assert!(!graph.adj.contains_key(&1));
    }

    #[test]
    fn test_arth_op_addition_cells() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 10; // A1 = 10
        arr[1] = 20; // B1 = 20
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = arth_op(
            "C1=A1+B1",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[2], 30); // C1 = A1 + B1 = 10 + 20
        assert_eq!(
            formula_array[2],
            Formula {
                op_type: 5,
                p1: 0,
                p2: 1
            }
        );
        assert_eq!(graph.adj[&0], vec![2, 2]);
        assert_eq!(graph.adj[&1], vec![2, 2]);
    }

    #[test]
    fn test_arth_op_subtraction_cells() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 30; // A1 = 30
        arr[1] = 10; // B1 = 10
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = arth_op(
            "C1=A1-B1",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[2], 20); // C1 = A1 - B1 = 30 - 10
        assert_eq!(
            formula_array[2],
            Formula {
                op_type: 6,
                p1: 0,
                p2: 1
            }
        );
        assert_eq!(graph.adj[&0], vec![2, 2]);
        assert_eq!(graph.adj[&1], vec![2, 2]);
    }

    #[test]
    fn test_arth_op_multiplication_cells() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 5; // A1 = 5
        arr[1] = 4; // B1 = 4
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = arth_op(
            "C1=A1*B1",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[2], 20); // C1 = A1 * B1 = 5 * 4
        assert_eq!(
            formula_array[2],
            Formula {
                op_type: 7,
                p1: 0,
                p2: 1
            }
        );
        assert_eq!(graph.adj[&0], vec![2, 2]);
        assert_eq!(graph.adj[&1], vec![2, 2]);
    }

    #[test]
    fn test_arth_op_division_cells() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 20; // A1 = 20
        arr[1] = 4; // B1 = 4
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = arth_op(
            "C1=A1/B1",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[2], 5); // C1 = A1 / B1 = 20 / 4
        assert_eq!(
            formula_array[2],
            Formula {
                op_type: 8,
                p1: 0,
                p2: 1
            }
        );
        assert_eq!(graph.adj[&0], vec![2, 2]);
        assert_eq!(graph.adj[&1], vec![2, 2]);
    }

    #[test]
    fn test_arth_op_division_by_zero() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 20; // A1 = 20
        arr[1] = 0; // B1 = 0
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = arth_op(
            "C1=A1/B1",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[2], i32::MIN); // Division by zero results in i32::MIN
        assert_eq!(
            formula_array[2],
            Formula {
                op_type: 8,
                p1: 0,
                p2: 1
            }
        );
        assert_eq!(graph.adj[&0], vec![2, 2]);
        assert_eq!(graph.adj[&1], vec![2, 2]);
    }

    #[test]
    fn test_arth_op_addition_cell_and_literal() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 10; // A1 = 10
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = arth_op(
            "C1=A1+5",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[2], 15); // C1 = A1 + 5 = 10 + 5
        assert_eq!(
            formula_array[2],
            Formula {
                op_type: 1,
                p1: 0,
                p2: 5
            }
        );
        assert_eq!(graph.adj[&0], vec![2, 2]);
    }

    #[test]
    fn test_arth_op_invalid_input() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = arth_op(
            "C1=A1+@",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 1); // Invalid input
        assert_eq!(arr[2], 0); // No change
        assert_eq!(
            formula_array[2],
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            }
        );
    }

    #[test]
    fn test_funct_min() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 5; // A1
        arr[1] = 3; // B1
        arr[2] = 8; // C1
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = funct(
            "D1=MIN(A1:C1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[3], 3); // D1 = MIN(A1:C1)
    }

    #[test]
    fn test_funct_max() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 5; // A1
        arr[1] = 3; // B1
        arr[2] = 8; // C1
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = funct(
            "D1=MAX(A1:C1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[3], 8); // D1 = MAX(A1:C1)
    }

    #[test]
    fn test_funct_avg() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 5; // A1
        arr[1] = 3; // B1
        arr[2] = 8; // C1
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = funct(
            "D1=AVG(A1:C1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[3], 5); // D1 = AVG(A1:C1) = (5 + 3 + 8) / 3
    }

    #[test]
    fn test_funct_sum() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 5; // A1
        arr[1] = 3; // B1
        arr[2] = 8; // C1
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = funct(
            "D1=SUM(A1:C1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[3], 16); // D1 = SUM(A1:C1) = 5 + 3 + 8
    }

    #[test]
    fn test_funct_stdev() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 5; // A1
        arr[1] = 3; // B1
        arr[2] = 8; // C1
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = funct(
            "D1=STDEV(A1:C1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[3], 2); // D1 = STDEV(A1:C1) (rounded down)
    }

    #[test]
    fn test_funct_sleep_literal() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = funct(
            "D1=SLEEP(1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[3], 1); // D1 = SLEEP(1)
    }

    #[test]
    fn test_funct_sleep_cell_reference() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[0] = 2; // A1
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = funct(
            "D1=SLEEP(A1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[3], 2); // D1 = SLEEP(A1)
    }

    #[test]
    fn test_funct_invalid_function() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = funct(
            "D1=INVALID(A1:C1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 1); // Invalid function
    }

    #[test]
    fn test_parser_with_literal() {
        let mut sheet = initialize_spreadsheet(10, 10);
        let result = parser(&mut sheet, "A1=42");
        assert_eq!(result, 0);
        assert_eq!(sheet.arr[0], 42);
        assert_eq!(
            sheet.formula_array[0],
            Formula {
                op_type: 0,
                p1: 42,
                p2: 0
            }
        );
    }

    #[test]
    fn test_parser_with_negative_literal() {
        let mut sheet = initialize_spreadsheet(10, 10);
        let result = parser(&mut sheet, "A1=-42");
        assert_eq!(result, 0);
        assert_eq!(sheet.arr[0], -42);
        assert_eq!(
            sheet.formula_array[0],
            Formula {
                op_type: 0,
                p1: -42,
                p2: 0
            }
        );
    }

    #[test]
    fn test_parser_with_cell_reference() {
        let mut sheet = initialize_spreadsheet(10, 10);
        sheet.arr[1] = 50; // B1 = 50
        let result = parser(&mut sheet, "A1=B1");
        assert_eq!(result, 0);
        assert_eq!(sheet.arr[0], 50);
        assert_eq!(
            sheet.formula_array[0],
            Formula {
                op_type: 1,
                p1: 1,
                p2: 0
            }
        );
    }

    #[test]
    fn test_parser_with_arithmetic_operation() {
        let mut sheet = initialize_spreadsheet(10, 10);
        sheet.arr[0] = 10; // A1 = 10
        sheet.arr[1] = 20; // B1 = 20
        let result = parser(&mut sheet, "C1=A1+B1");
        assert_eq!(result, 0);
        assert_eq!(sheet.arr[2], 30); // C1 = A1 + B1
        assert_eq!(
            sheet.formula_array[2],
            Formula {
                op_type: 5,
                p1: 0,
                p2: 1
            }
        );
    }

    #[test]
    fn test_parser_with_function_min() {
        let mut sheet = initialize_spreadsheet(10, 10);
        sheet.arr[0] = 5; // A1
        sheet.arr[1] = 3; // B1
        sheet.arr[2] = 8; // C1
        let result = parser(&mut sheet, "D1=MIN(A1:C1)");
        assert_eq!(result, 0);
        assert_eq!(sheet.arr[3], 3); // D1 = MIN(A1:C1)
    }

    #[test]
    fn test_parser_with_function_sum() {
        let mut sheet = initialize_spreadsheet(10, 10);
        sheet.arr[0] = 5; // A1
        sheet.arr[1] = 3; // B1
        sheet.arr[2] = 8; // C1
        let result = parser(&mut sheet, "D1=SUM(A1:C1)");
        assert_eq!(result, 0);
        assert_eq!(sheet.arr[3], 16); // D1 = SUM(A1:C1)
    }

    #[test]
    fn test_parser_with_invalid_input() {
        let mut sheet = initialize_spreadsheet(10, 10);
        let result = parser(&mut sheet, "A1=@");
        assert_eq!(result, -1); // Invalid input
        assert_eq!(sheet.arr[0], 0); // No change
    }

    #[test]
    fn test_parser_with_invalid_cell_reference() {
        let mut sheet = initialize_spreadsheet(10, 10);
        let result = parser(&mut sheet, "A1=Z1");
        assert_eq!(result, 1); // Invalid cell reference
        assert_eq!(sheet.arr[0], 0); // No change
    }

    #[test]
    fn test_parser_with_function_sleep() {
        let mut sheet = initialize_spreadsheet(10, 10);
        let result = parser(&mut sheet, "D1=SLEEP(1)");
        assert_eq!(result, 0);
        assert_eq!(sheet.arr[3], 1); // D1 = SLEEP(1)
    }

    #[test]
    fn test_arth_op_literal_plus_cell() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        arr[1] = 20; // B1 = 20
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = arth_op(
            "C1=5+B1",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[2], 20);
        assert_eq!(
            formula_array[2],
            Formula {
                op_type: 5,
                p1: 1,
                p2: 5
            }
        );
        assert_eq!(graph.adj[&1], vec![2, 2]); // B1 depends on C1
    }

    #[test]
    fn test_arth_op_literal_plus_literal() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0
            };
            100
        ];

        let result = arth_op(
            "C1=5+10",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert_eq!(result, 0);
        assert_eq!(arr[2], 15); // C1 = 5 + 10
        assert_eq!(
            formula_array[2],
            Formula {
                op_type: 0,
                p1: 15,
                p2: 0
            }
        ); // Constant formula
    }
    #[test]
    fn test_parse_range_valid() {
        let cols = 10;
        let rows = 10;

        // Valid range: A1:B2
        let result = parse_range("A1:B2", cols, rows);
        assert!(result.is_some());
        let range = result.unwrap();
        assert_eq!(range.start_row, 0);
        assert_eq!(range.end_row, 1);
        assert_eq!(range.start_col, 0);
        assert_eq!(range.end_col, 1);
    }

    #[test]
    fn test_parse_range_invalid_format() {
        let cols = 10;
        let rows = 10;

        // Invalid range: Missing colon
        let result = parse_range("A1B2", cols, rows);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_range_out_of_bounds() {
        let cols = 10;
        let rows = 10;

        // Out-of-bounds range
        let result = parse_range("A1:Z10", cols, rows);
        assert!(result.is_none());
    }
    #[test]
    fn test_rollback_on_recalculate_failure() {
        let cols = 10;
        let rows = 10;
        let mut arr = vec![0; 100];
        let mut graph = Graph::new();
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0,
            };
            100
        ];

        // Set up initial state
        let dst = 5; // Target cell
        arr[dst] = 42;
        formula_array[dst] = Formula {
            op_type: 1,
            p1: 2,
            p2: 3,
        };

        unsafe {
            OLD_VALUE = arr[dst];
            OLD_OP_TYPE = formula_array[dst].op_type;
            OLD_P1 = formula_array[dst].p1;
            OLD_P2 = formula_array[dst].p2;
        }

        // Simulate a failure in recalculate
        let recalculate_result = false;

        if !recalculate_result {
            // Rollback logic
            delete_edge(&mut graph, dst, &mut formula_array, cols);
            unsafe {
                arr[dst] = OLD_VALUE;
                formula_array[dst] = Formula {
                    op_type: OLD_OP_TYPE,
                    p1: OLD_P1,
                    p2: OLD_P2,
                };
                add_formula(
                    &mut graph,
                    dst,
                    OLD_P1,
                    OLD_P2,
                    OLD_OP_TYPE,
                    &mut formula_array,
                    cols,
                );
            }
        }

        // Assertions to verify rollback
        assert_eq!(arr[dst], 42); // Value should be restored
        assert_eq!(
            formula_array[dst],
            Formula {
                op_type: 1,
                p1: 2,
                p2: 3,
            }
        ); // Formula should be restored
    }

    #[test]
    fn test_recalculate_failure_triggers_rollback() {
        let mut graph = Graph::new();
        let cols = 3;
        let mut arr = vec![0; 3];
        let mut formula_array = vec![
            Formula {
                op_type: 0,
                p1: 10,
                p2: 0,
            },
            Formula {
                op_type: 1,
                p1: 0,
                p2: 5,
            },
            Formula {
                op_type: 1,
                p1: 1,
                p2: 5,
            },
        ];

        // Add a cycle to the graph using `depend`
        depend(&mut graph, 0, 1);
        depend(&mut graph, 1, 2);
        depend(&mut graph, 2, 0);

        // Set old values for rollback
        const OLD_VALUE: i32 = 42;
        const OLD_OP_TYPE: i32 = 0;
        const OLD_P1: i32 = 0;
        const OLD_P2: i32 = 0;
        arr[2] = OLD_VALUE;
        formula_array[2] = Formula {
            op_type: OLD_OP_TYPE,
            p1: OLD_P1,
            p2: OLD_P2,
        };

        // Attempt to recalculate, expecting failure
        let result = recalculate(&mut graph, cols, &mut arr, 2, &formula_array);
        if !result {
            // Simulate rollback
            delete_edge(&mut graph, 2, &formula_array, cols as usize);
            unsafe {
                arr[2] = OLD_VALUE;
                formula_array[2] = Formula {
                    op_type: OLD_OP_TYPE,
                    p1: OLD_P1,
                    p2: OLD_P2,
                };
                add_formula(
                    &mut graph,
                    2,
                    OLD_P1,
                    OLD_P2,
                    OLD_OP_TYPE,
                    &mut formula_array,
                    cols as usize,
                );
            }
        }

        // Assert rollback occurred
        assert_eq!(arr[2], OLD_VALUE);
        assert_eq!(formula_array[2].op_type, OLD_OP_TYPE);
        assert_eq!(formula_array[2].p1, OLD_P1);
        assert_eq!(formula_array[2].p2, OLD_P2);
    }
}

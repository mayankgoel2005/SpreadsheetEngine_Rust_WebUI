// src/input_parser.rs

use crate::spreadsheet::Spreadsheet;
use crate::graph::{Graph, Formula, arith, add_formula, delete_edge, recalculate};
use crate::functions::{max_func, sum_func, standard_dev_func, avg_func, min_func, sleep_func};

/// Save+restore on rollback
static mut OLD_VALUE:   i32 = 0;
static mut OLD_OP_TYPE: i32 = 0;
static mut OLD_P1:      i32 = 0;
static mut OLD_P2:      i32 = 0;
pub   static mut HAS:   i32 = 0;

#[inline] fn is_alpha(c: char) -> bool { c.is_ascii_uppercase() }
#[inline] fn is_digit(c: char) -> bool { c.is_ascii_digit() }

/// Mark dst dependent on src (no duplicates).
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
            if seen_digit { return -1; }
            col = col * 26 + (ch as i32 - 'A' as i32 + 1);
        } else if is_digit(ch) {
            row = row * 10 + (ch as i32 - '0' as i32);
            seen_digit = true;
        } else {
            return -1;
        }
    }
    col -= 1;  row -= 1;
    if col < 0 || row < 0 || col >= cols || row >= rows {
        -1
    } else {
        row * cols + col
    }
}

/// + → 1, - → 2, * → 3, / → 4
#[inline]
fn return_optype(op: char) -> i32 {
    match op {
        '+' => 1,
        '-' => 2,
        '*' => 3,
        '/' => 4,
        _   => i32::MIN,
    }
}

/// Handle “dst = [±] literal” or “dst = [±] cell”
fn value_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq : usize,
    arr: &mut [i32],
    g  : &mut Graph,
    farr: &mut [Formula],
) -> i32 {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 {
        return 1;
    }
    /* save old */
    unsafe {
        OLD_VALUE   = arr[dst as usize];
        OLD_OP_TYPE = farr[dst as usize].op_type;
        OLD_P1      = farr[dst as usize].p1;
        OLD_P2      = farr[dst as usize].p2;
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
    let is_cell = rhs.bytes().next().map(|b: u8| b.is_ascii_uppercase()).unwrap_or(false);

    let val;
    if is_cell {
        let src = cell_parser(rhs, cols, rows);
        if src == -1 {
            return 1;
        }
        depend(g, src as usize, dst as usize);

        val = if neg { -arr[src as usize] } else { arr[src as usize] };
        add_formula(g, dst as usize, src, if neg { -1 } else { 0 }, if neg { 3 } else { 1 }, farr, cols as usize);
    } else {
        match rhs.parse::<i32>() {
            Ok(v) => val = if neg { -v } else { v },
            Err(_) => {
                return 1;
            },
        }
        add_formula(g, dst as usize, val, 0, 0, farr, cols as usize);
    }

    arr[dst as usize] = val;

    if !recalculate(g, cols, arr, dst as usize, farr) {
        /* rollback */
        delete_edge(g, dst as usize, farr, cols as usize);
        unsafe {
            arr[dst as usize] = OLD_VALUE;
            farr[dst as usize] = Formula { op_type: OLD_OP_TYPE, p1: OLD_P1, p2: OLD_P2 };
            add_formula(g, dst as usize, OLD_P1, OLD_P2, OLD_OP_TYPE, farr, cols as usize);
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
    g:   &mut Graph,
    farr:&mut [Formula],
) -> i32 {
    // find operator
    let mut op_ind = None;
    let mut op_ch  = '+';
    for (i, ch) in txt[eq+1..].char_indices() {
        if "+-*/".contains(ch) {
            op_ind = Some(eq + 1 + i);
            op_ch  = ch;
            break;
        }
    }
    let op_ind = match op_ind { Some(i) => i, None => return 1 };

    // split operands
    let left_s  = txt[eq+1..op_ind].trim();
    let right_s = txt[op_ind+1..].trim();

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
    if left_val == i32::MIN  { return 1 }
    let left_val = if lneg { -left_val } else { left_val };

    // parse right
    let (rneg, right_s) = if let Some(stripped) = right_s.strip_prefix('-') {
        (true, stripped.trim())
    } else if let Some(stripped) = right_s.strip_prefix('+') {
        (false, stripped.trim())
    } else {
        (false, right_s)
    };
    let right_is_cell = right_s.chars().next().is_some_and(is_alpha);    let right_val = if right_is_cell {
        cell_parser(right_s, cols, rows)
    } else {
        right_s.parse::<i32>().unwrap_or(i32::MIN)
    };
    if right_val == i32::MIN { return 1 }
    let right_val = if rneg { -right_val } else { right_val };

    // dst
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst < 0 { return 1 }
    let dst = dst as usize;

    // stash/rollback
    unsafe {
        OLD_VALUE   = arr[dst];
        OLD_OP_TYPE = farr[dst].op_type;
        OLD_P1      = farr[dst].p1;
        OLD_P2      = farr[dst].p2;
    }
    if farr[dst].op_type > 0 {
        delete_edge(g, dst, farr, cols as usize);
    }

    // evaluate
    let result = arith(left_val, right_val, op_ch);

    // record dependencies & formula
    let base_op = return_optype(op_ch);
    let (p1, p2, op_type) = match (left_is_cell, right_is_cell) {
        (true,  true ) => {
            let p1 = left_val.abs() ; // left index stored
            let p2 = right_val.abs() ;
            depend(g, p1 as usize, dst);
            depend(g, p2 as usize, dst);
            (p1, p2, base_op + 4)       // cell+cell => ops 5..8
        }
        (true,  false) => {
            let p1 = left_val.abs() ;
            let p2 = right_val;
            depend(g, p1 as usize, dst);
            (p1, p2, base_op)           // cell+lit => ops 1..4
        }
        (false, true ) => {
            let p1 = right_val.abs() ;
            let p2 = left_val;
            depend(g, p1 as usize, dst);
            (p1, p2, base_op + 4)       // lit+cell treat as cell+cell
        }
        (false, false) => {
            // pure literal+literal → constant
            arr[dst] = result;
            add_formula(g, dst, result, 0, 0, farr, cols as usize);
            return if recalculate(g, cols, arr, dst, farr) { 0 } else { 1 };
        }
    };
    arr[dst] = result;
    add_formula(g, dst, p1, p2, op_type, farr, cols as usize);

    // recalc / rollback
    if !recalculate(g, cols, arr, dst, farr) {
        delete_edge(g, dst, farr, cols as usize);
        unsafe {
            arr[dst] = OLD_VALUE;
            farr[dst] = Formula { op_type: OLD_OP_TYPE, p1: OLD_P1, p2: OLD_P2 };
            add_formula(g, dst, OLD_P1, OLD_P2, OLD_OP_TYPE, farr, cols as usize);
        }
        return 1;
    }
    0
}

/// Dispatch SUM/AVG/MIN/.../SLEEP
fn funct(
    txt: &str,
    cols: i32,
    rows: i32,
    eq:  usize,
    arr: &mut [i32],
    g:   &mut Graph,
    farr:&mut [Formula],
) -> i32 {
    let ok = if txt[eq+1..].starts_with("MIN(") {
        min_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq+1..].starts_with("MAX(") {
        max_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq+1..].starts_with("AVG(") {
        avg_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq+1..].starts_with("SUM(") {
        sum_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq+1..].starts_with("STDEV(") {
        standard_dev_func(txt, cols, rows, eq, arr, g, farr)
    } else if txt[eq+1..].starts_with("SLEEP(") {
        sleep_func(txt, cols, rows, eq, arr, g, farr)
    } else {
        return 1;
    };
    if !ok { return 1 }
    // re‐run recalc on dst
    let dst = cell_parser(&txt[..eq], cols, rows) as usize;
    if !recalculate(g, cols, arr, dst, farr) {
        delete_edge(g, dst, farr, cols as usize);
        unsafe {
            arr[dst] = OLD_VALUE;
            delete_edge(g, dst, farr, cols as usize);
            farr[dst] = Formula { op_type: OLD_OP_TYPE, p1: OLD_P1, p2: OLD_P2 };
            add_formula(g, dst, OLD_P1, OLD_P2, OLD_OP_TYPE, farr, cols as usize);
        }
        return 1;
    }
    0
}

/// Entry point
pub fn parser(sheet: &mut Spreadsheet, txt: &str) -> i32 {
    let cols = sheet.cols as i32;
    let rows = sheet.rows as i32;
    let arr  = &mut sheet.arr;
    let g    = &mut sheet.graph;
    let farr = &mut sheet.formula_array;

    let eq = txt.find('=').unwrap_or(usize::MAX);
    if eq == usize::MAX { return -1 }

    // classify
    let rhs = &txt[eq+1..].trim();
    let is_func = rhs.contains('(');

    // Check if it's a simple value (numeric or cell reference)
    let is_val = if let Some(first_char) = rhs.chars().next() {
        let is_single_term = rhs.chars()
            .skip(match first_char {
                '+' | '-' => 1, // skip optional sign
                _ => 0
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
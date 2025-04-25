// src/functions.rs
use std::{thread, time::Duration};

use crate::graph::{Formula, Graph, add_formula};
use crate::input_parser::cell_parser;

/// insert dest into the dependents list of src, deduplicating
#[inline]
fn depend(g: &mut Graph, src: usize, dst: usize) {
    g.adj.entry(src).or_default().push(dst);
}

#[inline]
fn validate_range(start: i32, end: i32, cols: i32) -> bool {
    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end / cols, end % cols);
    !(sr > er || (sr == er && sc > ec))
}
/// Compute the minimum value over a range
/// record the formula in the dependency graph, and store the result in `dst`.
///
/// Returns `true` on success, `false` if the text is malformed or the range is invalid.
///
/// # Parameters
///
/// - `txt`: the entire formula text, e.g. `"C3=MIN(A1:B2)"`
/// - `cols`, `rows`: sheet dimensions
/// - `eq`: index of the `=` in `txt`
/// - `arr`: the current cell values (row-major)
/// - `g`: the dependency graph to update
/// - `farr`: the array of recorded formulas (one per cell)
///
/// # Examples
///
/// ```rust
/// # use lab1::functions::min_func;
/// # use lab1::graph::{Graph, Formula};
/// # use lab1::input_parser::cell_parser;
/// let mut arr = vec![10, 3, 0, 7, 2,0];
/// let mut graph = Graph::new();
/// let mut farr = vec![Formula{op_type:9,p1:0,p2:4}; arr.len()];
/// let ok = min_func("C1=MIN(A1:B2)", 3, 2, 2, &mut arr, &mut graph, &mut farr);
/// assert!(ok);
/// assert_eq!(arr[2], 2);
/// ```
pub fn min_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    // _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 {
        return false;
    }

    /* locate “( A1:B2 )” */
    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close = txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 {
        return false;
    }

    let inside = &txt[open + 1..close];
    let colon = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX {
        return false;
    }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end = cell_parser(&inside[colon + 1..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) {
        return false;
    }

    /* register formula & dependencies */
    add_formula(g, dst as usize, start, end, 9, farr, cols as usize);

    let mut min_val = arr[start as usize];
    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end / cols, end % cols);

    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            if idx != dst as usize {
                depend(g, idx, dst as usize);
            } else {
                return false;
            }

            if arr[idx] < min_val {
                min_val = arr[idx];
            }
        }
    }
    arr[dst as usize] = min_val;
    true
}
/// Compute the maximum value over a range
/// record the formula in the dependency graph, and store the result in `dst`.
///
/// Returns `true` on success, `false` if the text is malformed or the range is invalid.
///
/// # Parameters
///
/// - `txt`: the entire formula text, e.g. `"C3=MIN(A1:B2)"`
/// - `cols`, `rows`: sheet dimensions
/// - `eq`: index of the `=` in `txt`
/// - `arr`: the current cell values (row-major)
/// - `g`: the dependency graph to update
/// - `farr`: the array of recorded formulas (one per cell)
///
/// # Examples
///
/// ```rust
/// # use lab1::functions::max_func;
/// # use lab1::graph::{Graph, Formula};
/// # use lab1::input_parser::cell_parser;
/// let mut arr = vec![10, 3, 0, 7, 2,0];
/// let mut graph = Graph::new();
/// let mut farr = vec![Formula{op_type:10,p1:0,p2:4}; arr.len()];
/// let ok = max_func("C1=MAX(A1:B2)", 3, 2, 2, &mut arr, &mut graph, &mut farr);
/// assert!(ok);
/// assert_eq!(arr[2], 10);
/// ```
pub fn max_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    // _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 {
        return false;
    }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close = txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 {
        return false;
    }

    let inside = &txt[open + 1..close];
    let colon = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX {
        return false;
    }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end = cell_parser(&inside[colon + 1..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) {
        return false;
    }

    add_formula(g, dst as usize, start, end, 10, farr, cols as usize);

    let mut max_val = arr[start as usize];
    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end / cols, end % cols);

    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            if idx != dst as usize {
                depend(g, idx, dst as usize);
            } else {
                return false;
            }
            if arr[idx] > max_val {
                max_val = arr[idx];
            }
        }
    }
    arr[dst as usize] = max_val;
    true
}
/// Compute the average value over a range
/// record the formula in the dependency graph, and store the result in `dst`.
///
/// Returns `true` on success, `false` if the text is malformed or the range is invalid.
///
/// # Parameters
///
/// - `txt`: the entire formula text, e.g. `"C3=MIN(A1:B2)"`
/// - `cols`, `rows`: sheet dimensions
/// - `eq`: index of the `=` in `txt`
/// - `arr`: the current cell values (row-major)
/// - `g`: the dependency graph to update
/// - `farr`: the array of recorded formulas (one per cell)
///
/// # Examples
///
/// ```rust
/// # use lab1::functions::avg_func;
/// # use lab1::graph::{Graph, Formula};
/// # use lab1::input_parser::cell_parser;
/// let mut arr = vec![10, 12, 0, 14, 16,0];
/// let mut graph = Graph::new();
/// let mut farr = vec![Formula{op_type:11,p1:0,p2:4}; arr.len()];
/// let ok = avg_func("C1=AVG(A1:B2)", 3, 2, 2, &mut arr, &mut graph, &mut farr);
/// assert!(ok);
/// assert_eq!(arr[2], 13);
/// ```
pub fn avg_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    // _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 {
        return false;
    }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close = txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 {
        return false;
    }

    let inside = &txt[open + 1..close];
    let colon = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX {
        return false;
    }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end = cell_parser(&inside[colon + 1..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) {
        return false;
    }

    add_formula(g, dst as usize, start, end, 11, farr, cols as usize);

    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end / cols, end % cols);

    let mut sum = 0;
    let mut cnt = 0;
    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            if idx != dst as usize {
                depend(g, idx, dst as usize);
            } else {
                return false;
            }
            sum += arr[idx];
            cnt += 1;
        }
    }
    arr[dst as usize] = if cnt == 0 { 0 } else { sum / cnt };
    true
}
/// Compute the sum over a range
/// record the formula in the dependency graph, and store the result in `dst`.
///
/// Returns `true` on success, `false` if the text is malformed or the range is invalid.
///
/// # Parameters
///
/// - `txt`: the entire formula text, e.g. `"C3=MIN(A1:B2)"`
/// - `cols`, `rows`: sheet dimensions
/// - `eq`: index of the `=` in `txt`
/// - `arr`: the current cell values (row-major)
/// - `g`: the dependency graph to update
/// - `farr`: the array of recorded formulas (one per cell)
///
/// # Examples
///
/// ```rust
/// # use lab1::functions::sum_func;
/// # use lab1::graph::{Graph, Formula};
/// # use lab1::input_parser::cell_parser;
/// let mut arr = vec![10, 3, 0, 7, 2,0];
/// let mut graph = Graph::new();
/// let mut farr = vec![Formula{op_type:12,p1:0,p2:4}; arr.len()];
/// let ok = sum_func("C1=SUM(A1:B2)", 3, 2, 2, &mut arr, &mut graph, &mut farr);
/// assert!(ok);
/// assert_eq!(arr[2], 22);
/// ```
pub fn sum_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    // _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 {
        return false;
    }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close = txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 {
        return false;
    }

    let inside = &txt[open + 1..close];
    let colon = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX {
        return false;
    }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end = cell_parser(&inside[colon + 1..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) {
        return false;
    }

    add_formula(g, dst as usize, start, end, 12, farr, cols as usize);

    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end / cols, end % cols);

    let mut sum = 0;
    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            if idx != dst as usize {
                depend(g, idx, dst as usize);
            } else {
                return false;
            }
            sum += arr[idx];
        }
    }
    arr[dst as usize] = sum;
    true
}
/// Compute the stdev over a range
/// record the formula in the dependency graph, and store the result in `dst`.
///
/// Returns `true` on success, `false` if the text is malformed or the range is invalid.
///
/// # Parameters
///
/// - `txt`: the entire formula text, e.g. `"C3=MIN(A1:B2)"`
/// - `cols`, `rows`: sheet dimensions
/// - `eq`: index of the `=` in `txt`
/// - `arr`: the current cell values (row-major)
/// - `g`: the dependency graph to update
/// - `farr`: the array of recorded formulas (one per cell)
///
/// # Examples
///
/// ```rust
/// # use lab1::functions::standard_dev_func;
/// # use lab1::graph::{Graph, Formula};
/// # use lab1::input_parser::cell_parser;
/// let mut arr = vec![10, 10, 0, 10, 10,0];
/// let mut graph = Graph::new();
/// let mut farr = vec![Formula{op_type:13,p1:0,p2:4}; arr.len()];
/// let ok = standard_dev_func("C1=STDEV(A1:B2)", 3, 2, 2, &mut arr, &mut graph, &mut farr);
/// assert!(ok);
/// assert_eq!(arr[2], 0);
/// ```
pub fn standard_dev_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    // _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 {
        return false;
    }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close = txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 {
        return false;
    }

    let inside = &txt[open + 1..close];
    let colon = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX {
        return false;
    }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end = cell_parser(&inside[colon + 1..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) {
        return false;
    }

    add_formula(g, dst as usize, start, end, 13, farr, cols as usize);

    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end / cols, end % cols);

    let mut sum = 0;
    let mut cnt = 0;
    let mut sum_sq = 0;
    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            if idx != dst as usize {
                depend(g, idx, dst as usize);
            } else {
                return false;
            }
            sum += arr[idx];
            cnt += 1;
            sum_sq += arr[idx] * arr[idx];
        }
    }
    if cnt <= 1 {
        arr[dst as usize] = 0;
        true
    } else {
        let avg = sum / cnt;
        let var = ((sum_sq - 2 * sum * avg + avg * avg * cnt) as f64) / (cnt as f64);
        arr[dst as usize] = var.sqrt().round() as i32;
        true
    }
}
/// Pause the thread for the specified number of seconds (literal or cell reference),
/// record that as a “SLEEP” formula, and store the elapsed seconds in `dst`.
///
/// Returns `true` on success or `false` on parse/range error.
///
/// # Examples
///
/// ```rust
/// # use lab1::functions::sleep_func;
/// # use lab1::graph::{Graph, Formula};
/// # use lab1::input_parser::cell_parser;
/// let mut arr = vec![5];      // we’ll sleep for 5 seconds
/// let mut graph = Graph::new();
/// let mut farr = vec![Formula{op_type:0,p1:0,p2:0}; 1];
/// let ok = sleep_func("A1=SLEEP(1)", 1, 1, 2, &mut arr, &mut graph, &mut farr);
/// assert!(ok);
/// assert_eq!(arr[0], 1);
/// ```
pub fn sleep_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    // _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 {
        return false;
    }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close = txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 {
        return false;
    }

    let inside = &txt[open + 1..close];
    let maybe_ref = cell_parser(inside, cols, rows);

    let secs = if maybe_ref != -1 {
        depend(g, maybe_ref as usize, dst as usize);
        arr[maybe_ref as usize]
    } else {
        match inside.trim().parse::<i32>() {
            Ok(v) => v,
            Err(_) => return false,
        }
    };

    add_formula(
        g,
        dst as usize,
        if maybe_ref != -1 { maybe_ref } else { dst },
        secs,
        14,
        farr,
        cols as usize,
    );

    if secs > 0 {
        thread::sleep(Duration::from_secs(secs as u64));
    }
    arr[dst as usize] = secs;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Formula, Graph};

    #[test]
    fn test_min_func_invalid_dst() {
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

        let result = min_func(
            "Z1=MIN(A1:B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Invalid destination cell
    }

    #[test]
    fn test_min_func_invalid_range_format() {
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

        let result = min_func(
            "A1=MIN(A1B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Missing colon in range
    }

    #[test]
    fn test_min_func_invalid_range_cells() {
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

        let result = min_func(
            "A1=MIN(A1:Z10)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Out-of-bounds range
    }

    #[test]
    fn test_min_func_self_dependency() {
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

        let result = min_func(
            "A1=MIN(A1:A1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Self-dependency
    }

    #[test]
    fn test_max_func_invalid_dst() {
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

        let result = max_func(
            "Z1=MAX(A1:B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Invalid destination cell
    }

    #[test]
    fn test_max_func_invalid_range_format() {
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

        let result = max_func(
            "A1=MAX(A1B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Missing colon in range
    }

    #[test]
    fn test_max_func_invalid_range_cells() {
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

        let result = max_func(
            "A1=MAX(A1:Z10)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Out-of-bounds range
    }

    #[test]
    fn test_avg_func_invalid_dst() {
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

        let result = avg_func(
            "Z1=AVG(A1:B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Invalid destination cell
    }

    #[test]
    fn test_avg_func_invalid_range_format() {
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

        let result = avg_func(
            "A1=AVG(A1B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Missing colon in range
    }

    #[test]
    fn test_sum_func_invalid_dst() {
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

        let result = sum_func(
            "Z1=SUM(A1:B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Invalid destination cell
    }

    #[test]
    fn test_sum_func_invalid_range_format() {
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

        let result = sum_func(
            "A1=SUM(A1B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Missing colon in range
    }

    #[test]
    fn test_standard_dev_func_invalid_dst() {
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

        let result = standard_dev_func(
            "Z1=STDEV(A1:B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Invalid destination cell
    }

    #[test]
    fn test_standard_dev_func_invalid_range_format() {
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

        let result = standard_dev_func(
            "A1=STDEV(A1B2)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Missing colon in range
    }

    #[test]
    fn test_sleep_func_invalid_dst() {
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

        let result = sleep_func(
            "Z1=SLEEP(1)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Invalid destination cell
    }

    #[test]
    fn test_sleep_func_invalid_literal() {
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

        let result = sleep_func(
            "A1=SLEEP(abc)",
            cols,
            rows,
            2,
            &mut arr,
            &mut graph,
            &mut formula_array,
        );
        assert!(!result); // Invalid literal
    }
}

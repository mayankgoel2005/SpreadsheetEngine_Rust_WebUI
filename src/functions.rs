// src/functions.rs
use std::{thread, time::Duration};

use crate::graph::{Graph, Formula, add_formula};
use crate::input_parser::cell_parser;

/// insert `dest` into the dependents list of `src`, deduplicating
#[inline]
pub fn depend(graph: &mut Graph, src: usize, dest: usize) {
    if !graph.adj[src].contains(&dest) {
        graph.adj[src].push(dest);
    }
}

#[inline]
fn validate_range(start: i32, end: i32, cols: i32) -> bool {
    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end   / cols, end   % cols);
    !(sr > er || (sr == er && sc > ec))
}

/*───────────────────────────────────────────────────────────────────────────*/
/*  MIN                                                                     */
/*───────────────────────────────────────────────────────────────────────────*/
pub fn min_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 { return false; }

    /* locate “( A1:B2 )” */
    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close= txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 { return false; }

    let inside = &txt[open + 1 .. close];
    let colon  = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX { return false; }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end   = cell_parser(&inside[colon + 1 ..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) { return false; }

    /* register formula & dependencies */
    add_formula(g, dst as usize, start, end, 9, farr, cols as usize);

    let mut min_val = arr[start as usize];
    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end   / cols, end   % cols);

    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            depend(g, idx, dst as usize);

            if arr[idx] < min_val { min_val = arr[idx]; }
        }
    }
    arr[dst as usize] = min_val;
    true
}

/*───────────────────────────────────────────────────────────────────────────*/
/*  MAX ­– identical structure, only the reducer changes                    */
/*───────────────────────────────────────────────────────────────────────────*/
pub fn max_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 { return false; }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close= txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 { return false; }

    let inside = &txt[open + 1 .. close];
    let colon  = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX { return false; }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end   = cell_parser(&inside[colon + 1 ..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) { return false; }

    add_formula(g, dst as usize, start, end, 10, farr, cols as usize);

    let mut max_val = arr[start as usize];
    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end   / cols, end   % cols);

    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            depend(g, idx, dst as usize);
            if arr[idx] > max_val { max_val = arr[idx]; }
        }
    }
    arr[dst as usize] = max_val;
    true
}

/*───────────────────────────────────────────────────────────────────────────*/
/*  AVG                                                                     */
/*───────────────────────────────────────────────────────────────────────────*/
pub fn avg_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 { return false; }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close= txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 { return false; }

    let inside = &txt[open + 1 .. close];
    let colon  = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX { return false; }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end   = cell_parser(&inside[colon + 1 ..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) { return false; }

    add_formula(g, dst as usize, start, end, 11, farr, cols as usize);

    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end   / cols, end   % cols);

    let mut sum = 0;
    let mut cnt = 0;
    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            depend(g, idx, dst as usize);
            sum += arr[idx];
            cnt += 1;
        }
    }
    arr[dst as usize] = if cnt == 0 { 0 } else { sum / cnt };
    true
}

/*───────────────────────────────────────────────────────────────────────────*/
/*  SUM ­– straightforward                                                  */
/*───────────────────────────────────────────────────────────────────────────*/
pub fn sum_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 { return false; }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close= txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 { return false; }

    let inside = &txt[open + 1 .. close];
    let colon  = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX { return false; }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end   = cell_parser(&inside[colon + 1 ..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) { return false; }

    add_formula(g, dst as usize, start, end, 12, farr, cols as usize);

    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end   / cols, end   % cols);

    let mut sum = 0;
    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            depend(g, idx, dst as usize);
            sum += arr[idx];
        }
    }
    arr[dst as usize] = sum;
    true
}

/*───────────────────────────────────────────────────────────────────────────*/
/*  STDEV                                                                   */
/*───────────────────────────────────────────────────────────────────────────*/
pub fn standard_dev_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 { return false; }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close= txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 { return false; }

    let inside = &txt[open + 1 .. close];
    let colon  = inside.find(':').unwrap_or(usize::MAX);
    if colon == usize::MAX { return false; }

    let start = cell_parser(&inside[..colon], cols, rows);
    let end   = cell_parser(&inside[colon + 1 ..], cols, rows);
    if start == -1 || end == -1 || !validate_range(start, end, cols) { return false; }

    add_formula(g, dst as usize, start, end, 13, farr, cols as usize);

    let (sr, sc) = (start / cols, start % cols);
    let (er, ec) = (end   / cols, end   % cols);

    let mut sum = 0;
    let mut cnt = 0;
    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            depend(g, idx, dst as usize);
            sum += arr[idx];
            cnt += 1;
        }
    }
    let avg = if cnt == 0 { 0.0 } else { sum as f64 / cnt as f64 };

    let mut acc = 0.0;
    for r in sr..=er {
        for c0 in sc..=ec {
            let idx = (r * cols + c0) as usize;
            let d = arr[idx] as f64 - avg;
            acc += d * d;
        }
    }
    arr[dst as usize] = (acc / cnt as f64).sqrt() as i32;
    true
}

/*───────────────────────────────────────────────────────────────────────────*/
/*  SLEEP                                                                   */
/*───────────────────────────────────────────────────────────────────────────*/
pub fn sleep_func(
    txt: &str,
    cols: i32,
    rows: i32,
    eq: usize,
    _end: usize,
    arr: &mut [i32],
    g: &mut Graph,
    farr: &mut [Formula],
) -> bool {
    let dst = cell_parser(&txt[..eq], cols, rows);
    if dst == -1 { return false; }

    let open = txt[eq..].find('(').map(|o| o + eq).unwrap_or(0);
    let close= txt[eq..].find(')').map(|c| c + eq).unwrap_or(0);
    if close <= open + 1 { return false; }

    let inside = &txt[open + 1 .. close];
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

    add_formula(g, dst as usize,
                if maybe_ref != -1 { maybe_ref } else { dst },
                secs, 14, farr, cols as usize);

    if secs > 0 { thread::sleep(Duration::from_secs(secs as u64)); }
    arr[dst as usize] = secs;
    true
}

// src/graph.rs

use std::collections::VecDeque;
use std::fmt;
// use std::i32;

/// A recorded formula:
///  op_type:
///    0 = constant
///    1–4 = "cell ±/* literal"
///    5–8 = "cell ±/* cell"
///    9–13 = MIN, MAX, AVG, SUM, STDEV over a range [p1..p2]
///    14 = SLEEP
#[derive(Copy, Clone, Debug)]
pub struct Formula {
    pub op_type: i32,
    pub p1:     i32,
    pub p2:     i32,
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Formula {{ op_type: {}, p1: {}, p2: {} }}", self.op_type, self.p1, self.p2)
    }
}


/// A very simple adjacency list: for each cell index, a Vec of its dependents.
pub struct Graph {
    pub adj: Vec<Vec<usize>>,
}

impl Graph {
    /// Create a new graph for `size` cells (0..size-1).
    pub fn new(size: usize) -> Self {
        Graph { adj: vec![Vec::new(); size] }
    }
}

/// Record a new formula in `formula_array[cell]` and install its dependency edges.
pub fn add_formula(
    graph:          &mut Graph,
    cell:           usize,
    p1:             i32,
    p2:             i32,
    op_type:        i32,
    formula_array:  &mut [Formula],
    cols: usize,
) {
    formula_array[cell] = Formula { op_type, p1, p2 };
    match op_type {
        1..=4 => {
            let src = p1 as usize;
            graph.adj[src].push(cell);
        }
        5..=8 => {
            let s1 = p1 as usize;
            let s2 = p2 as usize;
            graph.adj[s1].push(cell);
            graph.adj[s2].push(cell);
        }
        9..=13 => {
            let start = p1 as usize;
            let end   = p2 as usize;
            let (sr, sc) = (start / cols, start % cols);
            let (er, ec) = (end / cols, end % cols);

            // reject true rectangles
            if sr != er && sc != ec { return; }

            if sc == ec {
                // vertical
                for r in sr..=er {
                    let src = r * cols + sc;
                    if src == cell { continue; }
                    graph.adj[src].push(cell);
                }
            } else {
                // horizontal
                for c in sc..=ec {
                    let src = sr * cols + c;
                    if src == cell { continue; }
                    graph.adj[src].push(cell);
                }
            }
        }
        _ => {}
    }
}

/// Remove the old edges for whatever formula was in `formula_array[cell]`.
pub fn delete_edge(
    graph:         &mut Graph,
    cell:          usize,
    formula_array: &[Formula],
    cols:          usize,
) {
    let f = formula_array[cell];
    match f.op_type {
        1..=4 => {
            let src = f.p1 as usize;
            graph.adj[src].retain(|&d| d != cell);
        }
        5..=8 => {
            let s1 = f.p1 as usize;
            let s2 = f.p2 as usize;
            graph.adj[s1].retain(|&d| d != cell);
            graph.adj[s2].retain(|&d| d != cell);
        }
        9..=13 => {
            let start = f.p1 as usize;
            let end   = f.p2 as usize;
            let (sr, sc) = (start / cols, start % cols);
            let (er, ec) = (end / cols, end % cols);

            if sc == ec {
                // vertical
                for r in sr..=er {
                    let src = r * cols + sc;
                    if src == cell { continue; }
                    graph.adj[src].push(cell);
                }
            } else {
                // horizontal
                for c in sc..=ec {
                    let src = sr * cols + c;
                    if src == cell { continue; }
                    graph.adj[src].push(cell);
                }
            }
        }
        _ => {}
    }
}

#[inline]
pub fn arith(v1: i32, v2: i32, op: char) -> i32 {
    match op {
        '+' => v1.wrapping_add(v2),
        '-' => v1.wrapping_sub(v2),
        '*' => v1.wrapping_mul(v2),
        '/' => if v2 != 0 { v1 / v2 } else { i32::MIN },
        _   => i32::MIN,
    }
}

/// Kahn’s algorithm over the **reachable** subgraph starting at `start`.
/// Returns `None` if a cycle is found; otherwise the topo order.
pub fn topological_sort(
    graph:      &Graph,
    start:      usize,
    total_size: usize,
) -> Option<Vec<usize>> {
    // 1) BFS to mark reachable and count in‑degrees
    let mut in_degree = vec![0; total_size];
    let mut reachable = vec![false; total_size];
    let mut queue     = VecDeque::new();
    reachable[start] = true;
    let mut reach=1;
    queue.push_back(start);
    while let Some(u) = queue.pop_front() {
        for &v in &graph.adj[u] {
            in_degree[v] += 1;
            if !reachable[v] {
                reachable[v] = true;
                reach+=1;
                queue.push_back(v);
            }
        }
    }

    // 2) Kahn’s zero‑queue
    let mut zero   = VecDeque::new();
    let mut result = Vec::new();
    zero.push_back(start);
    while let Some(u) = zero.pop_front() {
        result.push(u);
        for &v in &graph.adj[u] {
            in_degree[v] -= 1;
            if in_degree[v] == 0 {
                zero.push_back(v);
            }
        }
    }
    if result.len() != reach{
        // cycle among the reachable nodes
        None
    } else {
        Some(result)
    }
}

/// Recalculate all formulas downstream of `start_cell` into `arr`.
/// If a cycle is detected, returns `false` and leaves `arr` untouched.
pub fn recalculate(
    graph:          &mut Graph,
    cols:           i32,
    arr:            &mut [i32],
    start_cell:     usize,
    formula_array:  &[Formula],
) -> bool {
    let total_size = arr.len();
    let sorted = match topological_sort(graph, start_cell, total_size) {
        Some(v) => v,
        None    => return false,
    };

    // make a working copy and zero out all dependents
    for &c in &sorted {
        arr[c] = 0;
    }

    // now re‑evaluate in topo order
    for &c in &sorted {
        let f = formula_array[c];
        match f.op_type {
            0 => {
                // constant / direct value
                arr[c] = if f.p1 == i32::MIN { i32::MIN } else { f.p1 };
            }
            1..=4 => {
                let v1 = arr[f.p1 as usize];
                let v2 = f.p2;
                if v1 == i32::MIN {
                    arr[c] = i32::MIN;
                } else {
                    let op = match f.op_type { 1 => '+', 2 => '-', 3 => '*', 4 => '/', _ => '+' };
                    arr[c] = if op == '/' && v2 == 0 {
                        i32::MIN
                    } else {
                        arith(v1, v2, op)
                    };
                }
            }
            5..=8 => {
                let v1 = arr[f.p1 as usize];
                let v2 = arr[f.p2 as usize];
                if v1 == i32::MIN || v2 == i32::MIN {
                    arr[c] = i32::MIN;
                } else {
                    let op = match f.op_type { 5 => '+', 6 => '-', 7 => '*', 8 => '/', _ => '+' };
                    arr[c] = if op == '/' && v2 == 0 {
                        i32::MIN
                    } else {
                        arith(v1, v2, op)
                    };
                }
            }
            9..=13 => {
                // ranges
                let start = f.p1 as usize;
                let end   = f.p2 as usize;
                let sr = start / cols as usize;
                let sc = start % cols as usize;
                let er = end   / cols as usize;
                let ec = end   % cols as usize;

                let mut cnt = 0;
                let mut sum = 0;
                let mut mn  = i32::MAX;
                let mut mx  = i32::MIN;
                let mut sd_acc = 0.0;
                let mut err = false;

                for r in sr..=er {
                    for col in sc..=ec {
                        let idx = r * cols as usize + col;
                        let v   = arr[idx];
                        if v == i32::MIN { err = true }
                        cnt += 1;
                        sum += v;
                        mn = mn.min(v);
                        mx = mx.max(v);
                    }
                }

                if err || cnt == 0 {
                    arr[c] = i32::MIN;
                } else {
                    arr[c] = match f.op_type {
                        9  => mn,
                        10 => mx,
                        11 => sum / cnt,
                        12 => sum,
                        13 => {
                            let avg = sum as f64 / cnt as f64;
                            for r in sr..=er {
                                for col in sc..=ec {
                                    let idx = r * cols as usize + col;
                                    let d   = arr[idx] as f64 - avg;
                                    sd_acc += d * d;
                                }
                            }
                            (sd_acc / cnt as f64).sqrt() as i32
                        }
                        _ => unreachable!(),
                    };
                }
            }
            14 => {
                // Sleep / passthrough
                let val = if f.p1 as usize == c {
                    f.p2
                } else {
                    arr[f.p1 as usize]
                };
                arr[c] = val;
            }
            _ => {}
        }
    }
    true
}

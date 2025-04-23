// src/graph.rs

use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;
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
    pub adj: HashMap<usize, Vec<usize>>,
}

impl Graph {
    /// Create a new graph with an initially empty adjacency map.
    pub fn new() -> Self {
        Graph {
            adj: HashMap::new(),
        }
    }
}

/// Record a new formula in formula_array[cell] and install its dependency edges.
pub fn add_formula(
    graph:         &mut Graph,
    cell:          usize,
    p1:            i32,
    p2:            i32,
    op_type:       i32,
    formula_array: &mut [Formula],
    cols:          usize,
) {
    formula_array[cell] = Formula { op_type, p1, p2 };

    match op_type {
        1..=4 => {
            let src = p1 as usize;
            graph.adj.entry(src).or_default().push(cell);
        }
        5..=8 => {
            let s1 = p1 as usize;
            let s2 = p2 as usize;
            graph.adj.entry(s1).or_default().push(cell);
            graph.adj.entry(s2).or_default().push(cell);
        }
        9..=13 => {
            let start = p1 as usize;
            let end   = p2 as usize;
            let (sr, sc) = (start / cols, start % cols);
            let (er, ec) = (end   / cols, end   % cols);

            // Reject rectangles: only allow full row or full column ranges
            if sr != er && sc != ec { return; }

            if sc == ec {
                // vertical range
                for r in sr..=er {
                    let src = r * cols + sc;
                    if src != cell {
                        graph.adj.entry(src).or_default().push(cell);
                    }
                }
            } else {
                // horizontal range
                for c in sc..=ec {
                    let src = sr * cols + c;
                    if src != cell {
                        graph.adj.entry(src).or_default().push(cell);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Remove the old edges for whatever formula was in formula_array[cell].
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
            if let Some(dependents) = graph.adj.get_mut(&src) {
                dependents.retain(|&d| d != cell);
                if dependents.is_empty() {
                    graph.adj.remove(&src);
                }
            }
        }
        5..=8 => {
            let s1 = f.p1 as usize;
            let s2 = f.p2 as usize;
            for src in [s1, s2] {
                if let Some(dependents) = graph.adj.get_mut(&src) {
                    dependents.retain(|&d| d != cell);
                    if dependents.is_empty() {
                        graph.adj.remove(&src);
                    }
                }
            }
        }
        9..=13 => {
            let start = f.p1 as usize;
            let end   = f.p2 as usize;
            let (sr, sc) = (start / cols, start % cols);
            let (er, ec) = (end   / cols, end   % cols);

            if sr != er && sc != ec {
                return; // skip true rectangles
            }

            if sc == ec {
                // vertical range
                for r in sr..=er {
                    let src = r * cols + sc;
                    if src == cell { continue; }
                    if let Some(dependents) = graph.adj.get_mut(&src) {
                        dependents.retain(|&d| d != cell);
                        if dependents.is_empty() {
                            graph.adj.remove(&src);
                        }
                    }
                }
            } else {
                // horizontal range
                for c in sc..=ec {
                    let src = sr * cols + c;
                    if src == cell { continue; }
                    if let Some(dependents) = graph.adj.get_mut(&src) {
                        dependents.retain(|&d| d != cell);
                        if dependents.is_empty() {
                            graph.adj.remove(&src);
                        }
                    }
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

/// Kahn’s algorithm over the *reachable* subgraph starting at start.
/// Returns None if a cycle is found; otherwise the topo order.
pub fn topological_sort(
    graph: &Graph,
    start: usize,
) -> Option<Vec<usize>> {
    // Only store reachable in-degrees
    let mut in_degree: HashMap<usize, usize> = HashMap::new();
    let mut reachable: HashSet<usize> = HashSet::new();
    let mut queue = VecDeque::new();

    // Step 1: Discover reachable nodes and count in-degrees
    reachable.insert(start);
    queue.push_back(start);

    while let Some(u) = queue.pop_front() {
        if let Some(neighbors) = graph.adj.get(&u) {
            for &v in neighbors {
                *in_degree.entry(v).or_insert(0) += 1;
                if reachable.insert(v) {
                    queue.push_back(v);
                }
            }
        }
    }

    // Step 2: Kahn’s algorithm using only reachable nodes
    let mut zero_in = VecDeque::new();
    let mut result = Vec::new();

    // Add nodes with zero in-degree
    for &node in &reachable {
        if !in_degree.contains_key(&node) {
            zero_in.push_back(node);
        }
    }

    while let Some(u) = zero_in.pop_front() {
        result.push(u);
        if let Some(neighbors) = graph.adj.get(&u) {
            for &v in neighbors {
                if let Some(indeg) = in_degree.get_mut(&v) {
                    *indeg -= 1;
                    if *indeg == 0 {
                        zero_in.push_back(v);
                    }
                }
            }
        }
    }

    if result.len() != reachable.len() {
        None
    } else {
        Some(result)
    }
}

/// Recalculate all formulas downstream of start_cell into arr.
/// If a cycle is detected, returns false and leaves arr untouched.
pub fn recalculate(
    graph:          &mut Graph,
    cols:           i32,
    arr:            &mut [i32],
    start_cell:     usize,
    formula_array:  &[Formula],
) -> bool {
    let _total_size = arr.len();
    let sorted = match topological_sort(graph, start_cell) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arith_addition() {
        assert_eq!(arith(10, 5, '+'), 15);
        assert_eq!(arith(-10, 5, '+'), -5);
        assert_eq!(arith(i32::MAX, 1, '+'), i32::MAX.wrapping_add(1)); // Wrapping addition
    }

    #[test]
    fn test_arith_subtraction() {
        assert_eq!(arith(10, 5, '-'), 5);
        assert_eq!(arith(5, 10, '-'), -5);
        assert_eq!(arith(i32::MIN, 1, '-'), i32::MIN.wrapping_sub(1)); // Wrapping subtraction
    }

    #[test]
    fn test_arith_multiplication() {
        assert_eq!(arith(3, 4, '*'), 12);
        assert_eq!(arith(-3, 4, '*'), -12);
        assert_eq!(arith(i32::MAX, 2, '*'), i32::MAX.wrapping_mul(2)); // Wrapping multiplication
        assert_eq!(arith(i32::MIN, -1, '*'), i32::MIN.wrapping_mul(-1)); // Wrapping multiplication
    }

    #[test]
    fn test_arith_division() {
        assert_eq!(arith(10, 2, '/'), 5);
        assert_eq!(arith(10, 0, '/'), i32::MIN); // Division by zero
        assert_eq!(arith(-10, 2, '/'), -5);
    }

    #[test]
    fn test_arith_invalid_operation() {
        assert_eq!(arith(10, 5, '%'), i32::MIN); // Unsupported operator
    }
}

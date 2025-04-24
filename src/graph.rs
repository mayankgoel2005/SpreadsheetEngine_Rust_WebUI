// src/graph.rs

use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
// use std::i32;
/// graph_auto.rs is for terminal version
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


/// A very simple adjacency list: for each cell index, a hashmap of its dependents.
pub struct Graph {
    pub adj: HashMap<usize, Vec<usize>>,
}

impl Graph {
    /// Create a brand‐new, empty dependency graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use lab1::graph::Graph;
    ///
    /// let g = Graph::new();
    /// assert!(g.adj.is_empty());
    /// ```
    pub fn new() -> Self {
        Graph {
            adj: HashMap::new(),
        }
    }
}

/// Install a formula into `formula_array[cell]` *and* hook up its dependency edges.
///
/// - `graph`:  your dependency graph
/// - `cell`:  index of the cell being (re)defined
/// - `p1,p2`: formula operands (cells or literals)
/// - `op_type`: formula kind code (see [`Formula`])
/// - `formula_array`: the per‐cell storage you’ll replay in `recalculate`
/// - `cols`: number of columns (for decoding ranges)
///
/// # Examples
///
/// ```rust
/// use lab1::graph::{Graph, Formula, add_formula};
///
/// // formulas[3] = cell 0 + literal 5
/// let mut g = Graph::new();
/// let mut formulas = vec![Formula { op_type:0,p1:0,p2:0 }; 10];
/// add_formula(&mut g, 3, 0, 5, 1, &mut formulas, /*cols=*/5);
/// assert_eq!(formulas[3].op_type, 1);
/// assert_eq!(g.adj.get(&0).unwrap(), &vec![3]);
/// ```
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

/// Remove *all* dependency edges caused by whatever formula was in `formula_array[cell]`.
///
/// After this call, that cell’s previous dependents will no longer be notified when the
/// source cells change.
///
/// # Examples
///
/// ```rust
/// use lab1::graph::{Graph, Formula, add_formula, delete_edge};
///
/// let mut g = Graph::new();
/// let mut formulas = vec![Formula { op_type:0,p1:0,p2:0 }; 10];
/// // build a single dependency:
/// add_formula(&mut g, 3, 0, 5, 1, &mut formulas, 3);
/// assert!(g.adj.get(&0).unwrap().contains(&3));
/// // now drop it:
/// delete_edge(&mut g, 3, &formulas, 3);
/// assert!(!g.adj.contains_key(&0));
/// ```
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
/// Perform a single arithmetic operation, *wrapping* on overflow (and signalling division-by-zero
/// by returning `i32::MIN`).
///
/// # Examples
///
/// ```rust
/// use lab1::graph::arith;
/// assert_eq!(arith(5, 3, '+'), 8);
/// assert_eq!(arith(5, 0, '/'), i32::MIN);
/// ```
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
/// Return a topological ordering of all nodes reachable *from* `start`.  If any cycle is found
/// among those reachable nodes, returns `None`.
///
/// # Examples
///
/// ```rust
/// use lab1::graph::{Graph, add_formula, topological_sort};
///
/// // Build chain 0 → 1 → 2
/// let mut g = Graph::new();
/// let mut f = vec![super::Formula { op_type:0, p1:0, p2:0 }; 3];
/// add_formula(&mut g, 1, 0, 0, 1, &mut f, 3); // 1 depends on 0
/// add_formula(&mut g, 2, 1, 0, 1, &mut f, 3); // 2 depends on 1
///
/// assert_eq!(topological_sort(&g, 0), Some(vec![0,1,2]));
///
/// // Introduce a cycle 2 → 1:
/// add_formula(&mut g, 1, 2, 0, 1, &mut f, 3);
/// assert_eq!(topological_sort(&g, 0), None);
/// ```
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
/// Recompute (in topological‐sort order) **all** formulas downstream of `start_cell`, writing
/// their values into `arr`.  Returns `false` (and leaves `arr` untouched) if a cycle is detected.
///
/// # Examples
///
/// ```rust
/// use lab1::graph::{Graph, add_formula, recalculate};
/// use lab1::spreadsheet::initialize_spreadsheet;
///
/// // very small 1×3 sheet: cells 0,1,2
/// let mut sheet = initialize_spreadsheet(1,3);
/// sheet.arr[0] = 4;
/// sheet.arr[1] = 2;
///
/// // cell 2 = cell 0 + cell 1
/// add_formula(&mut sheet.graph, 2, 0, 1, 5, &mut sheet.formula_array, 3);
///
/// assert!(recalculate(&mut sheet.graph, 3, &mut sheet.arr, 2, &sheet.formula_array));
/// assert_eq!(sheet.arr[2], 6);
/// ```
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
                let sd_acc = 0.0;
                let mut err = false;
                let mut sum_sq=0;
                for r in sr..=er {
                    for col in sc..=ec {
                        let idx = r * cols as usize + col;
                        let v   = arr[idx];
                        if v == i32::MIN { err = true }
                        cnt += 1;
                        sum += v;
                        sum_sq+=v*v;
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
                            let avg = sum/cnt;
                            let variance = ((sum_sq - 2 * sum * avg + avg * avg * cnt) as f64) / (cnt as f64);
                            variance.sqrt().round() as i32
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
    fn test_graph_new() {
        let graph = Graph::new(5);
        assert_eq!(graph.adj.len(), 5);
        for adj_list in graph.adj {
            assert!(adj_list.is_empty());
        }
    }

    #[test]
    fn test_add_formula_simple_dependency() {
        let mut graph = Graph::new(5);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 5];
        add_formula(&mut graph, 1, 0, 0, 1, &mut formula_array, 5);
        assert_eq!(graph.adj[0], vec![1]);
        assert_eq!(formula_array[1], Formula { op_type: 1, p1: 0, p2: 0 });
    }

    #[test]
    fn test_add_formula_double_dependency() {
        let mut graph = Graph::new(5);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 5];
        add_formula(&mut graph, 2, 0, 1, 5, &mut formula_array, 5);
        assert_eq!(graph.adj[0], vec![2]);
        assert_eq!(graph.adj[1], vec![2]);
        assert_eq!(formula_array[2], Formula { op_type: 5, p1: 0, p2: 1 });
    }

    #[test]
    fn test_add_formula_range_dependency_vertical() {
        let mut graph = Graph::new(9);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 9];
        add_formula(&mut graph, 8, 0, 6, 9, &mut formula_array, 3);
        assert_eq!(graph.adj[0], vec![8]);
        assert_eq!(graph.adj[3], vec![8]);
        assert_eq!(graph.adj[6], vec![8]);
    }

    #[test]
    fn test_add_formula_range_dependency_horizontal() {
        let mut graph = Graph::new(9);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 9];
        add_formula(&mut graph, 8, 0, 2, 9, &mut formula_array, 3);
        assert_eq!(graph.adj[0], vec![8]);
        assert_eq!(graph.adj[1], vec![8]);
        assert_eq!(graph.adj[2], vec![8]);
    }

    #[test]
    fn test_delete_edge_simple_dependency() {
        let mut graph = Graph::new(5);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 5];
        add_formula(&mut graph, 1, 0, 0, 1, &mut formula_array, 5);
        delete_edge(&mut graph, 1, &formula_array, 5);
        assert!(graph.adj[0].is_empty());
    }

    #[test]
    fn test_delete_edge_double_dependency() {
        let mut graph = Graph::new(5);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 5];
        add_formula(&mut graph, 2, 0, 1, 5, &mut formula_array, 5);
        delete_edge(&mut graph, 2, &formula_array, 5);
        assert!(graph.adj[0].is_empty());
        assert!(graph.adj[1].is_empty());
    }

    #[test]
    fn test_topological_sort_no_cycle() {
        let mut graph = Graph::new(5);
        graph.adj[0].push(1);
        graph.adj[1].push(2);
        let result = topological_sort(&graph, 0, 5);
        assert_eq!(result, Some(vec![0, 1, 2]));
    }

    #[test]
    fn test_topological_sort_with_cycle() {
        let mut graph = Graph::new(3);
        graph.adj[0].push(1);
        graph.adj[1].push(2);
        graph.adj[2].push(0);
        let result = topological_sort(&graph, 0, 3);
        assert_eq!(result, None);
    }

    #[test]
    fn test_recalculate_no_cycle() {
        let mut graph = Graph::new(5);
        let mut arr = vec![0; 5];
        let formula_array = vec![
            Formula { op_type: 0, p1: 10, p2: 0 },
            Formula { op_type: 1, p1: 0, p2: 5 },
            Formula { op_type: 0, p1: 0, p2: 0 },
            Formula { op_type: 0, p1: 0, p2: 0 },
            Formula { op_type: 0, p1: 0, p2: 0 },
        ];
        graph.adj[0].push(1);
        let result = recalculate(&mut graph, 5, &mut arr, 0, &formula_array);
        assert!(result);
        assert_eq!(arr[0], 10);
        assert_eq!(arr[1], 15);
    }

    #[test]
    fn test_recalculate_with_cycle() {
        let mut graph = Graph::new(3);
        let mut arr = vec![0; 3];
        let formula_array = vec![
            Formula { op_type: 0, p1: 10, p2: 0 },
            Formula { op_type: 1, p1: 0, p2: 5 },
            Formula { op_type: 1, p1: 1, p2: 5 },
        ];
        graph.adj[0].push(1);
        graph.adj[1].push(2);
        graph.adj[2].push(0);
        let result = recalculate(&mut graph, 3, &mut arr, 0, &formula_array);
        assert!(!result);
    }

    #[test]
    fn test_formula_display() {
        let formula = Formula { op_type: 9, p1: 5, p2: 10 };
        let display = format!("{}", formula);
        assert_eq!(display, "Formula { op_type: 9, p1: 5, p2: 10 }");
    }

    #[test]
    fn test_add_formula_range_dependency_invalid_rectangle() {
        let mut graph = Graph::new(9);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 9];
        add_formula(&mut graph, 8, 0, 8, 9, &mut formula_array, 3); // Invalid rectangle
        assert!(graph.adj[0].is_empty());
        assert!(graph.adj[1].is_empty());
        assert!(graph.adj[2].is_empty());
    }

    #[test]
    fn test_add_formula_range_min_vertical() {
        let mut graph = Graph::new(9);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 9];
        add_formula(&mut graph, 8, 0, 6, 9, &mut formula_array, 3); // MIN over vertical range
        assert_eq!(graph.adj[0], vec![8]);
        assert_eq!(graph.adj[3], vec![8]);
        assert_eq!(graph.adj[6], vec![8]);
    }

    #[test]
    fn test_add_formula_range_max_horizontal() {
        let mut graph = Graph::new(9);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 9];
        add_formula(&mut graph, 8, 0, 2, 10, &mut formula_array, 3); // MAX over horizontal range
        assert_eq!(graph.adj[0], vec![8]);
        assert_eq!(graph.adj[1], vec![8]);
        assert_eq!(graph.adj[2], vec![8]);
    }

    #[test]
    fn test_add_formula_range_invalid_rectangle() {
        let mut graph = Graph::new(9);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 9];
        add_formula(&mut graph, 8, 0, 8, 9, &mut formula_array, 3); // Invalid rectangle
        assert!(graph.adj[0].is_empty());
        assert!(graph.adj[1].is_empty());
        assert!(graph.adj[2].is_empty());
    }

    #[test]
    fn test_add_formula_range_sum_vertical() {
        let mut graph = Graph::new(16);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 16];
        add_formula(&mut graph, 15, 4, 12, 12, &mut formula_array, 4); // SUM over vertical range
        assert_eq!(graph.adj[4], vec![15]);
        assert_eq!(graph.adj[8], vec![15]);
        assert_eq!(graph.adj[12], vec![15]);
    }

    #[test]
    fn test_add_formula_range_avg_horizontal() {
        let mut graph = Graph::new(16);
        let mut formula_array = vec![Formula { op_type: 0, p1: 0, p2: 0 }; 16];
        add_formula(&mut graph, 15, 8, 11, 11, &mut formula_array, 4); // AVG over horizontal range
        assert_eq!(graph.adj[8], vec![15]);
        assert_eq!(graph.adj[9], vec![15]);
        assert_eq!(graph.adj[10], vec![15]);
        assert_eq!(graph.adj[11], vec![15]);
    }

    #[test]
    fn test_arith_subtraction() {
        assert_eq!(arith(10, 5, '-'), 5);
        assert_eq!(arith(5, 10, '-'), -5);
        assert_eq!(arith(i32::MIN, 1, '-'), i32::MAX); // Wrapping subtraction
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

    #[test]
    fn test_recalculate_addition() {
        let mut graph = Graph::new(3);
        let mut arr = vec![10, 20, 0];
        let formula_array = vec![
            Formula { op_type: 0, p1: 10, p2: 0 },
            Formula { op_type: 0, p1: 20, p2: 0 },
            Formula { op_type: 5, p1: 0, p2: 1 }, // 10 + 20
        ];
        graph.adj[0].push(2);
        graph.adj[1].push(2);

        let result = recalculate(&mut graph, 3, &mut arr, 2, &formula_array);
        assert!(result);
        assert_eq!(arr[2], 30);
    }

    #[test]
    fn test_recalculate_subtraction() {
        let mut graph = Graph::new(3);
        let mut arr = vec![30, 10, 0];
        let formula_array = vec![
            Formula { op_type: 0, p1: 30, p2: 0 },
            Formula { op_type: 0, p1: 10, p2: 0 },
            Formula { op_type: 6, p1: 0, p2: 1 }, // 30 - 10
        ];
        graph.adj[0].push(2);
        graph.adj[1].push(2);

        let result = recalculate(&mut graph, 3, &mut arr, 2, &formula_array);
        assert!(result);
        assert_eq!(arr[2], 20);
    }

    #[test]
    fn test_recalculate_multiplication() {
        let mut graph = Graph::new(3);
        let mut arr = vec![3, 4, 0];
        let formula_array = vec![
            Formula { op_type: 0, p1: 3, p2: 0 },
            Formula { op_type: 0, p1: 4, p2: 0 },
            Formula { op_type: 7, p1: 0, p2: 1 }, // 3 * 4
        ];
        graph.adj[0].push(2);
        graph.adj[1].push(2);

        let result = recalculate(&mut graph, 3, &mut arr, 2, &formula_array);
        assert!(result);
        assert_eq!(arr[2], 12);
    }

    #[test]
    fn test_recalculate_division() {
        let mut graph = Graph::new(3);
        let mut arr = vec![20, 4, 0];
        let formula_array = vec![
            Formula { op_type: 0, p1: 20, p2: 0 },
            Formula { op_type: 0, p1: 4, p2: 0 },
            Formula { op_type: 8, p1: 0, p2: 1 }, // 20 / 4
        ];
        graph.adj[0].push(2);
        graph.adj[1].push(2);

        let result = recalculate(&mut graph, 3, &mut arr, 2, &formula_array);
        assert!(result);
        assert_eq!(arr[2], 5);
    }

    #[test]
    fn test_recalculate_division_by_zero() {
        let mut graph = Graph::new(3);
        let mut arr = vec![20, 0, 0];
        let formula_array = vec![
            Formula { op_type: 0, p1: 20, p2: 0 },
            Formula { op_type: 0, p1: 0, p2: 0 },
            Formula { op_type: 8, p1: 0, p2: 1 }, // 20 / 0
        ];
        graph.adj[0].push(2);
        graph.adj[1].push(2);

        let result = recalculate(&mut graph, 3, &mut arr, 2, &formula_array);
        assert!(result);
        assert_eq!(arr[2], i32::MIN); // Division by zero results in i32::MIN
    }

    #[test]
    fn test_recalculate_with_error_value() {
        let mut graph = Graph::new(3);
        let mut arr = vec![i32::MIN, 10, 0];
        let formula_array = vec![
            Formula { op_type: 0, p1: i32::MIN, p2: 0 },
            Formula { op_type: 0, p1: 10, p2: 0 },
            Formula { op_type: 5, p1: 0, p2: 1 }, // i32::MIN + 10
        ];
        graph.adj[0].push(2);
        graph.adj[1].push(2);

        let result = recalculate(&mut graph, 3, &mut arr, 2, &formula_array);
        assert!(result);
        assert_eq!(arr[2], i32::MIN); // Error propagates
    }

    #[test]
    fn test_recalculate_min() {
        let mut graph = Graph::new(9);
        let mut arr = vec![5, 3, 8, 2, 7, 6, 4, 9, 1];
        let mut formula_array = vec![
            Formula { op_type: 0, p1: 0, p2: 0 }; 9
        ];
        add_formula(&mut graph, 8, 0, 6, 9, &mut formula_array, 3); // MIN over range A1:A3
        let result = recalculate(&mut graph, 3, &mut arr, 8, &formula_array);
        assert!(result);
        assert_eq!(arr[8], 2); // Minimum value in range
    }

    #[test]
    fn test_recalculate_max() {
        let mut graph = Graph::new(9);
        let mut arr = vec![5, 3, 8, 2, 7, 6, 4, 9, 1];
        let mut formula_array = vec![
            Formula { op_type: 0, p1: 0, p2: 0 }; 9
        ];
        add_formula(&mut graph, 8, 0, 6, 10, &mut formula_array, 3); // MAX over range A1:A3
        let result = recalculate(&mut graph, 3, &mut arr, 8, &mut formula_array);
        assert!(result);
        assert_eq!(arr[8], 5); // Maximum value in range
    }
}


use std::cell::RefCell;
use std::cmp;
use std::rc::Rc;
const INT_MIN: i32 = i32::MIN;
const CELLS: usize = 18278000;

#[derive(Debug)]
pub struct QueueNode {
    cell: i32,
    next: Option<Rc<RefCell<QueueNode>>>,
}

#[derive(Debug)]
pub struct Queue {
    front: Option<Rc<RefCell<QueueNode>>>,
    rear: Option<Rc<RefCell<QueueNode>>>,
}

impl Queue {
    fn new() -> Self {
        Queue {
            front: None,
            rear: None,
        }
    }

    fn enqueue(&mut self, cell: i32) {
        let new_node = Rc::new(RefCell::new(QueueNode { cell, next: None }));
        if self.rear.is_none() {
            self.front = Some(Rc::clone(&new_node));
            self.rear = Some(new_node);
        } else {
            if let Some(rear) = self.rear.take() {
                rear.borrow_mut().next = Some(Rc::clone(&new_node));
                self.rear = Some(new_node);
            }
        }
    }

    fn dequeue(&mut self) -> Option<i32> {
        if let Some(front) = self.front.take() {
            let result = front.borrow().cell;
            self.front = front.borrow_mut().next.take();
            if self.front.is_none() {
                self.rear = None;
            }
            Some(result)
        } else {
            None
        }
    }
}

fn get_nodes_from_avl(root: Option<&Box<Cell>>, nodes: &mut Vec<i32>) {
    if let Some(node) = root {
        get_nodes_from_avl(node.left.as_ref(), nodes);
        nodes.push(node.cell);
        get_nodes_from_avl(node.right.as_ref(), nodes);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Formula {
    pub op_type: i32,
    pub p1: i32,
    pub p2: i32,
}

#[derive(Debug, Clone)]
pub struct Cell {
    cell: i32,
    height: i32,
    left: Option<Box<Cell>>,
    right: Option<Box<Cell>>,
}

pub struct Graph {
    pub adj: Vec<Option<Box<Cell>>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            adj: vec![None; 1000],
        }
    }
}

pub fn add_formula(_graph: &mut Graph, cell: i32, p1: i32, p2: i32, op_type: i32, formula_list: &mut [Formula]) {
    let f = if op_type == 0 {
        Formula {
            op_type,
            p1,
            p2: -1,
        }
    } else {
        Formula {
            op_type,
            p1,
            p2,
        }
    };
    formula_list[cell as usize] = f;
}

fn height(cell: Option<&Box<Cell>>) -> i32 {
    cell.map_or(0, |c| c.height)
}

fn balance(cell: Option<&Box<Cell>>) -> i32 {
    cell.map_or(0, |c| height(c.left.as_ref()) - height(c.right.as_ref()))
}

fn right_rotate(mut cell: Box<Cell>) -> Box<Cell> {
    let mut x = cell.left.take().expect("left child must exist for rotation");
    let t2 = x.right.take();
    x.right = Some(cell);
    x.right.as_mut().unwrap().left = t2;
    let right = x.right.as_mut().unwrap();
    right.height = 1 + cmp::max(height(right.left.as_ref()), height(right.right.as_ref()));
    x.height = 1 + cmp::max(height(x.left.as_ref()), height(x.right.as_ref()));
    x
}

fn left_rotate(mut cell: Box<Cell>) -> Box<Cell> {
    let mut x = cell.right.take().expect("right child must exist for rotation");
    let t2 = x.left.take();
    x.left = Some(cell);
    x.left.as_mut().unwrap().right = t2;
    let left = x.left.as_mut().unwrap();
    left.height = 1 + cmp::max(height(left.left.as_ref()), height(left.right.as_ref()));
    x.height = 1 + cmp::max(height(x.left.as_ref()), height(x.right.as_ref()));
    x
}

pub fn add_cell(cell: i32) -> Box<Cell> {
    Box::new(Cell {
        cell,
        left: None,
        right: None,
        height: 1,
    })
}

pub fn create_graph() -> Graph {
    Graph {
        adj: vec![None; CELLS],
    }
}

pub fn add_edge(cell1: i32, x: Option<Box<Cell>>) -> Box<Cell> {
    match x {
        None => add_cell(cell1),
        Some(mut node) => {
            if cell1 < node.cell {
                node.left = Some(add_edge(cell1, node.left));
            } else if cell1 > node.cell {
                node.right = Some(add_edge(cell1, node.right));
            } else {
                return node;
            }
            node.height = 1 + cmp::max(height(node.left.as_ref()), height(node.right.as_ref()));

            let bal = balance(Some(&node));
            if bal > 1 && cell1 < node.left.as_ref().unwrap().cell {
                return right_rotate(node);
            }
            if bal < -1 && cell1 > node.right.as_ref().unwrap().cell {
                return left_rotate(node);
            }
            if bal > 1 && cell1 > node.left.as_ref().unwrap().cell {
                node.left = Some(left_rotate(node.left.take().unwrap()));
                return right_rotate(node);
            }
            if bal < -1 && cell1 < node.right.as_ref().unwrap().cell {
                node.right = Some(right_rotate(node.right.take().unwrap()));
                return left_rotate(node);
            }
            node
        }
    }
}

pub fn delete_cell(cell1: i32, x: Option<Box<Cell>>) -> Option<Box<Cell>> {
    match x {
        None => None,
        Some(mut node) => {
            if cell1 < node.cell {
                node.left = delete_cell(cell1, node.left);
            } else if cell1 > node.cell {
                node.right = delete_cell(cell1, node.right);
            } else {
                if node.left.is_none() || node.right.is_none() {
                    return if node.left.is_some() {
                        node.left.take()
                    } else {
                        node.right.take()
                    };
                } else {
                    let min_val = {
                        let mut current = node.right.as_ref().unwrap();
                        while let Some(ref left) = current.left {
                            current = left;
                        }
                        current.cell
                    };
                    node.cell = min_val;
                    node.right = delete_cell(min_val, node.right);
                }
            }
            node.height = 1 + cmp::max(height(node.left.as_ref()), height(node.right.as_ref()));
            let bal = balance(Some(&node));
            if bal > 1 && balance(node.left.as_ref()) >= 0 {
                return Some(right_rotate(node));
            }
            if bal > 1 && balance(node.left.as_ref()) < 0 {
                node.left = Some(left_rotate(node.left.take().unwrap()));
                return Some(right_rotate(node));
            }
            if bal < -1 && balance(node.right.as_ref()) <= 0 {
                return Some(left_rotate(node));
            }
            if bal < -1 && balance(node.right.as_ref()) > 0 {
                node.right = Some(right_rotate(node.right.take().unwrap()));
                return Some(left_rotate(node));
            }
            Some(node)
        }
    }
}

pub fn delete_edge(graph: &mut Graph, cell: i32, cols: i32, formula_list: &[Formula]) {
    let f = formula_list[cell as usize];
    if (1..=4).contains(&f.op_type) {
        graph.adj[f.p1 as usize] = delete_cell(cell, graph.adj[f.p1 as usize].take());
    } else if (5..=8).contains(&f.op_type) {
        graph.adj[f.p1 as usize] = delete_cell(cell, graph.adj[f.p1 as usize].take());
        graph.adj[f.p2 as usize] = delete_cell(cell, graph.adj[f.p2 as usize].take());
    } else if (9..=13).contains(&f.op_type) {
        let start = f.p1;
        let end = f.p2;
        let start_row = start / cols;
        let end_row = end / cols;
        let start_col = start % cols;
        let end_col = end % cols;
        for i in start_row..=end_row {
            for j in start_col..=end_col {
                let index = (i * cols + j) as usize;
                graph.adj[index] = delete_cell(cell, graph.adj[index].take());
            }
        }
    }
}

pub fn add_edge_formula(graph: &mut Graph, cell: i32, cols: i32, formula_array: &[Formula]) {
    let x = formula_array[cell as usize];
    if (1..=4).contains(&x.op_type) {
        graph.adj[x.p1 as usize] =
            Some(add_edge(cell, graph.adj[x.p1 as usize].take()));
    } else if (5..=8).contains(&x.op_type) {
        graph.adj[x.p1 as usize] =
            Some(add_edge(cell, graph.adj[x.p1 as usize].take()));
        graph.adj[x.p2 as usize] =
            Some(add_edge(cell, graph.adj[x.p2 as usize].take()));
    } else if (9..=13).contains(&x.op_type) {
        let start_cell = x.p1;
        let end_cell = x.p2;
        let start_row = start_cell / cols;
        let start_col = start_cell % cols;
        let end_row = end_cell / cols;
        let end_col = end_cell % cols;
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let target_cell = row * cols + col;
                graph.adj[target_cell as usize] =
                    Some(add_edge(cell, graph.adj[target_cell as usize].take()));
            }
        }
    }
}

pub fn topological_sort(
    graph: &mut Graph,
    start: i32,
    size: &mut i32,
    has_cycle: &mut i32,
) -> Option<Vec<i32>> {
    *size = 0;
    *has_cycle = 0;
    let mut num_reachable = 0;
    let mut result: Vec<i32> = Vec::with_capacity(CELLS);
    let mut in_degree = vec![0; CELLS];
    let mut reachable = vec![0; CELLS];
    let mut q = Queue::new();
    let mut vis = Queue::new();
    vis.enqueue(start);
    reachable[start as usize] = 1;
    num_reachable += 1;

    while vis.front.is_some() {
        let cur = vis.dequeue().unwrap();
        if let Some(ref avl) = graph.adj[cur as usize] {
            let mut nodes = Vec::new();
            get_nodes_from_avl(Some(avl), &mut nodes);
            for &dep in &nodes {
                in_degree[dep as usize] += 1;
                if reachable[dep as usize] == 0 {
                    reachable[dep as usize] = 1;
                    num_reachable += 1;
                    vis.enqueue(dep);
                }
            }
        }
    }
    if in_degree[start as usize] > 0 {
        println!("Cycle detected");
        return None;
    }
    q.enqueue(start);
    while q.front.is_some() {
        let cur = q.dequeue().unwrap();
        result.push(cur);
        if let Some(ref avl) = graph.adj[cur as usize] {
            let mut nodes = Vec::new();
            get_nodes_from_avl(Some(avl), &mut nodes);
            for &dep in &nodes {
                in_degree[dep as usize] -= 1;
                if in_degree[dep as usize] == 0 {
                    q.enqueue(dep);
                }
            }
        }
    }
    *size = result.len() as i32;
    if *size < num_reachable {
        *has_cycle = 1;
        return None;
    }
    Some(result)
}

pub fn arith(v1: i32, v2: i32, op: Option<char>) -> i32 {
    match op {
        Some('+') => v1 + v2,
        Some('-') => v1 - v2,
        Some('*') => v1 * v2,
        Some('/') => {
            if v2 != 0 {
                v1 / v2
            } else {
                INT_MIN
            }
        }
        _ => INT_MIN,
    }
}

pub fn recalculate(
    graph: &mut Graph,
    _cols: i32,
    arr: &mut [i32],
    start_cell: i32,
    formula_array: &[Formula]
) -> bool {
    // Topological sort to detect cycles and determine update order
    let mut size = 0;
    let mut has_cycle = 0;
    let sorted_cells = match topological_sort(graph, start_cell, &mut size, &mut has_cycle) {
        Some(v) => v,
        None => {
            println!("Error: Circular dependency detected. Command rejected.");
            return false;
        }
    };

    // Simulate updates on a clone of the array
    let mut new_arr = arr.to_vec();
    // Reset only dependent cells
    for &cell in &sorted_cells {
        new_arr[cell as usize] = 0;
    }

    // Recompute in topologically sorted order
    for &cell in &sorted_cells {
        let f = formula_array[cell as usize];
        match f.op_type {
            0 => {
                // Constant or direct assignment (including error sentinel)
                new_arr[cell as usize] = if f.p1 == std::i32::MIN { std::i32::MIN } else { f.p1 };
            }
            1..=4 => {
                // Arithmetic: one constant second operand
                let v1 = new_arr[f.p1 as usize];
                let v2 = f.p2;
                if v1 == std::i32::MIN {
                    new_arr[cell as usize] = std::i32::MIN;
                    continue;
                }
                let op = match f.op_type {
                    1 => '+', 2 => '-', 3 => '*', 4 => '/', _ => unreachable!(),
                };
                if op == '/' && v2 == 0 {
                    new_arr[cell as usize] = std::i32::MIN;
                } else {
                    new_arr[cell as usize] = arith(v1, v2, Some(op));
                }
            }
            5..=8 => {
                // Arithmetic: two cell operands
                let v1 = new_arr[f.p1 as usize];
                let v2 = new_arr[f.p2 as usize];
                if v1 == std::i32::MIN || v2 == std::i32::MIN || (f.op_type == 8 && v2 == 0) {
                    new_arr[cell as usize] = std::i32::MIN;
                    continue;
                }
                let op = match f.op_type {
                    5 => '+', 6 => '-', 7 => '*', 8 => '/', _ => unreachable!(),
                };
                new_arr[cell as usize] = arith(v1, v2, Some(op));
            }
            9..=13 => {
                // Advanced functions: MIN, MAX, AVG, SUM, STDEV over a range
                let start = f.p1 as usize;
                let end = f.p2 as usize;
                // Propagate error if any input in the range is ERR
                if (start..=end).any(|i| new_arr[i] == std::i32::MIN) {
                    new_arr[cell as usize] = std::i32::MIN;
                    continue;
                }
                let mut count = 0;
                let mut sum = 0;
                let mut min_value = std::i32::MAX;
                let mut max_value = std::i32::MIN;
                for i in start..=end {
                    let v = new_arr[i];
                    count += 1;
                    sum += v;
                    if v < min_value { min_value = v; }
                    if v > max_value { max_value = v; }
                }
                if count == 0 {
                    new_arr[cell as usize] = std::i32::MIN;
                } else {
                    match f.op_type {
                        9  => new_arr[cell as usize] = min_value,
                        10 => new_arr[cell as usize] = max_value,
                        11 => new_arr[cell as usize] = sum / count,
                        12 => new_arr[cell as usize] = sum,
                        13 => {
                            let avg = sum as f64 / count as f64;
                            let mut sd_acc = 0.0;
                            for i in start..=end {
                                let v = new_arr[i] as f64;
                                sd_acc += (v - avg).powi(2);
                            }
                            new_arr[cell as usize] = (sd_acc / count as f64).sqrt() as i32;
                        }
                        _ => {}
                    }
                }
            }
            14 => {
                // Sleep: simply set to the stored value
                let val = if f.p1 == cell { f.p2 } else { new_arr[f.p1 as usize] };
                new_arr[cell as usize] = val;
            }
            _ => {}
        }
    }

    // Commit simulated results
    arr.copy_from_slice(&new_arr);
    true
}

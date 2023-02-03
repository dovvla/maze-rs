use super::COLUMN_SIZE;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex, RwLock},
    thread, vec,
};

fn heuristic(from: usize, to: usize) -> usize {
    let dist = from.abs_diff(to);
    let (r, c) = (dist / COLUMN_SIZE, dist % COLUMN_SIZE);
    r.pow(2) + c.pow(2) // shortest distance squared
}

fn traverse_path(came_from: &[Option<usize>], end: usize) -> Vec<usize> {
    let mut current = end;
    let mut total_path = vec![current];
    while let Some(previous) = came_from[current] {
        current = previous;
        total_path.push(current)
    }
    total_path.reverse();
    total_path
}

fn count_doors(graph: &[Vec<u8>], came_from: &[Option<usize>], end: usize) -> usize {
    let mut current = end;
    let mut door_count = 0usize;
    while let Some(previous) = came_from[current] {
        if graph[previous][current] == 255 {
            door_count += 1;
        }
        current = previous;
    }
    door_count
}

#[allow(unused)]
pub fn deduplicate_path(path: &[usize]) -> Vec<usize> {
    let mut path = path.to_owned();
    path.dedup();
    path
}

#[allow(unused)]
pub fn a_star(start: usize, end: usize, graph: &[Vec<u8>]) -> Option<(Vec<usize>, Vec<bool>)> {
    let mut graph = graph.to_owned();

    let mut open_set = BinaryHeap::<usize>::with_capacity(graph.len());
    let mut came_from: Vec<Option<usize>> = vec![None; graph.len()];
    let mut consumed_key: Vec<bool> = vec![false; graph.len()];
    open_set.push(start);

    let mut global_score = vec![usize::MAX; graph.len()];
    global_score[start] = 0;
    let mut finish_score = vec![usize::MAX; graph.len()];
    finish_score[start] = heuristic(start, end);

    let mut global_key_util = vec![usize::MAX; graph.len()];
    global_key_util[start] = 0;

    while let Some(current) = open_set.pop() {
        if current == end {
            return Some((traverse_path(&came_from, end), consumed_key));
        }
        for neighbour in 0..graph[current].len() {
            let tentative_score = global_score[current] + 1;
            let mut tentative_keys = global_key_util[current];
            if graph[current][neighbour] == 0 {
                continue;
            } else if graph[current][neighbour] == 255 {
                tentative_keys += 1;
                graph[current][neighbour] = 1;
                graph[neighbour][current] = 1;
                consumed_key[neighbour] = true;
                // println!("consumed key {current}->{neighbour}");
            }
            if tentative_score < global_score[neighbour]
                || tentative_keys < global_key_util[neighbour]
            {
                came_from[neighbour] = Some(current);
                global_score[neighbour] = tentative_score;
                global_key_util[neighbour] = tentative_keys;
                finish_score[neighbour] = tentative_score + heuristic(neighbour, end);
                if open_set.iter().all(|e| *e != neighbour) {
                    open_set.push(neighbour);
                }
            }
        }
    }
    None
}

#[derive(Debug, Default, Clone)]
pub struct State {
    walk: Vec<usize>, // walk[-1] -> last visited
    doors_opened: HashSet<(usize, usize)>,
    keys_pickedup: Vec<bool>,
    graph: Arc<Vec<Vec<u8>>>,
    keys: Arc<Vec<bool>>,
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.walk.len().cmp(&other.walk.len()))
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.walk.len() == other.walk.len()
    }
}

impl State {
    fn key_count(&self) -> usize {
        self.keys_pickedup.iter().filter(|e| **e).count() - self.doors_opened.len()
    }
    fn node_pair(&self, to: usize) -> (usize, usize) {
        let from = self.walk[self.walk.len() - 1];
        if from > to {
            (to, from)
        } else {
            (from, to)
        }
    }
    fn door_opened(&self, to: usize) -> bool {
        self.doors_opened.contains(&self.node_pair(to))
    }
    fn open_door(mut self, to: usize) -> Option<Self> {
        match self.key_count() > 0 {
            true => {
                self.doors_opened.insert(self.node_pair(to));
                self.walk.push(to);
                if self.keys[to] && !self.keys_pickedup[to] {
                    self.keys_pickedup[to] = true
                }
                Some(self)
            }
            false => None,
        }
    }
    fn walk(mut self, to: usize) -> Option<Self> {
        let from = self.walk[self.walk.len() - 1];
        match self.graph[from][to] {
            1 => {
                self.walk.push(to);
                if self.keys[to] && !self.keys_pickedup[to] {
                    self.keys_pickedup[to] = true
                }
                return Some(self);
            }
            255 => self.open_door(to),
            _ => None,
        }
    }
    fn next_states(&self) -> Vec<Self> {
        let from = self.walk[self.walk.len() - 1];
        let mut v = Vec::with_capacity(4);
        for next_field in 0..self.graph[from].len() {
            if self.graph[from][next_field] != 0 {
                if let Some(n) = self.clone().walk(next_field) {
                    if n.walk.len() >= 3 {
                        let l = n.walk.len();
                        if n.walk[l - 1] == n.walk[l - 3] && !self.keys[n.walk[l - 2]] {
                            continue;
                        }
                    }
                    v.push(n)
                }
            }
        }
        v
    }
    fn at_end(&self, end: usize) -> bool {
        self.walk[self.walk.len() - 1] == end
    }
}

pub fn worker(
    end: usize,
    queue: Arc<RwLock<VecDeque<State>>>,
    min_walk: Arc<Mutex<Option<Vec<usize>>>>,
    sync_flex: Arc<Mutex<isize>>,
) {
    loop {
        let state = match queue.write() {
            Ok(mut rw) => match rw.pop_front() {
                Some(state) => {
                    *sync_flex.lock().unwrap() = 0;

                    state
                }
                None => {
                    *sync_flex.lock().unwrap() += 1;
                    let a = *sync_flex.lock().unwrap();
                    if a >= 15 {
                        return;
                    }
                    continue;
                }
            },
            Err(_) => continue,
        };
        let next_states = state.next_states();

        let min_state = next_states
            .clone()
            .into_iter()
            .filter(|s| s.at_end(end))
            .reduce(|min, c| if c < min { c } else { min });
        let mut states_to_push: VecDeque<State> = next_states
            .into_iter()
            .filter(|state| !state.at_end(end))
            .collect();
        match min_state {
            Some(s) => match min_walk.lock() {
                Ok(mut min_s) => {
                    let len = match &(*min_s) {
                        Some(ms) => ms.len(),
                        None => usize::MAX,
                    };
                    if len > s.walk.len() {
                        std::mem::replace::<Option<Vec<usize>>>(&mut *min_s, Some(s.walk));
                        println!("{:?}", min_s.clone().unwrap());
                    }
                }
                Err(_) => unreachable!(),
            },
            None => {}
        }
        let len;
        match min_walk.lock() {
            Ok(min_s) => {
                len = match &(*min_s) {
                    Some(ms) => ms.len(),
                    None => usize::MAX,
                };
            }
            Err(_) => unreachable!(),
        }
        states_to_push = states_to_push
            .into_iter()
            .filter(|st| st.walk.len() < len)
            .collect();

        match queue.write() {
            Ok(mut queue) => queue.append(&mut states_to_push),
            Err(_) => {}
        }
    }
}

const NUM_THREADS: isize = 16;
#[allow(unused)]
pub fn parallel_backtrack(start: usize, end: usize, graph: &[Vec<u8>], keys: &[bool]) {
    let inital_state = State {
        walk: vec![start],
        doors_opened: HashSet::new(),
        keys_pickedup: vec![false; graph.len()],
        graph: Arc::new(graph.to_owned()),
        keys: Arc::new(keys.to_owned()),
    };
    let queue = Arc::new(RwLock::new(VecDeque::<State>::new()));
    let min_walk: Arc<Mutex<Option<Vec<usize>>>> = Arc::new(Mutex::new(None));
    let sync_flex: Arc<Mutex<isize>> = Arc::new(Mutex::new(0));
    queue.write().unwrap().push_front(inital_state);
    let mut handles = vec![];
    for i in 0..NUM_THREADS {
        let queue = queue.clone();
        let min_walk = min_walk.clone();
        let sync_flex = sync_flex.clone();
        handles.push(thread::spawn(move || {
            worker(end, queue, min_walk, sync_flex)
        }));
    }
    for thread in handles.into_iter() {
        thread.join().unwrap();
    }
}

#[allow(unused)]
pub fn key_cumsum(path: &[usize], consumed_key: &[bool], keys: &[bool]) -> Vec<isize> {
    let mut required_keys = vec![0isize; path.len()];
    for f in 0..path.len() - 1 {
        if consumed_key[path[f + 1]] {
            required_keys[f] += 1;
        }
        if keys[path[f]] {
            required_keys[f] -= 1;
        }
    }

    // println!("csmd {:2?}", &required_keys);

    let mut required_keys_cumsum = vec![0; required_keys.len()];
    required_keys_cumsum[required_keys.len() - 1] = required_keys[required_keys.len() - 1];

    for k in (0..required_keys.len() - 1).rev() {
        required_keys_cumsum[k] = required_keys_cumsum[k + 1] + required_keys[k];
        if required_keys_cumsum[k] < 0 {
            required_keys_cumsum[k] = 0;
        }
    }

    required_keys_cumsum
}

#[allow(unused)]
pub fn key_pickup(
    path: &[usize],
    cumsum: &[isize],
    graph: &mut [Vec<u8>],
    keys: &mut [bool],
    inventory: isize,
) -> Option<(Vec<usize>, isize)> {
    let mut pikcup_path = vec![];
    let mut key_inventory = inventory;

    for f in 0..path.len() {
        if keys[path[f]] {
            key_inventory += 1;
            keys[path[f]] = false;
        }
        if key_inventory >= cumsum[f] {
            break;
        }
        let mut current = path[f];
        'inner: for (key, _, doors) in bfs_closest_keys(current, graph, keys, true) {
            if doors > key_inventory as usize {
                continue 'inner;
            }
            if let Some(key_path) = bfs_shortest_path(current, key, graph, &[], true) {
                pikcup_path.extend_from_slice(&key_path);
                // println!("kp{:?}", &key_path);
                // println!("doors {:?}", &doors);
            } else {
                return None;
            }
            key_inventory += 1;
            keys[key] = false;
            current = key;
            if key_inventory >= cumsum[f] {
                break 'inner;
            }
        }
        if key_inventory < cumsum[f] && graph[path[f]][path[f + 1]] == 255 {
            return None;
        }
    }
    // println!("pkup {:2?}", &pikcup_path);
    // println!("invt {:?}", &key_inventory);

    Some((pikcup_path, key_inventory))
}

enum BfsActionResult<T> {
    Accumulate(T),
    Return(Vec<T>),
}

type BfsMatching = dyn Fn(usize, usize, &[Vec<u8>], &[bool]) -> bool;
type BfsAction<T> = dyn Fn(usize, &[Vec<u8>], &[Option<usize>], bool) -> BfsActionResult<T>;

fn bfs<T>(
    start: usize,
    end: usize,
    graph: &[Vec<u8>],
    keys: &[bool],
    matching: Box<BfsMatching>,
    action: Box<BfsAction<T>>,
    ignore_doors: bool,
) -> Option<Vec<T>> {
    let mut visited = vec![false; graph.len()];
    let mut queue = VecDeque::<usize>::with_capacity(graph.len());
    visited[start] = true;
    queue.push_back(start);

    let mut came_from: Vec<Option<usize>> = vec![None; graph.len()];
    let mut goal_accumulator: Vec<T> = Vec::new();

    while let Some(current) = queue.pop_front() {
        if matching(current, end, graph, keys) {
            match action(current, graph, &came_from, ignore_doors) {
                BfsActionResult::Accumulate(val) => goal_accumulator.push(val),
                BfsActionResult::Return(val) => return Some(val),
            }
        }
        for neighbour in 0..graph[current].len() {
            if graph[current][neighbour] == 0 {
                continue;
            }
            if !ignore_doors && graph[current][neighbour] == 255 {
                continue;
            }
            if !visited[neighbour] {
                visited[neighbour] = true;
                queue.push_back(neighbour);
                came_from[neighbour] = Some(current);
            }
        }
    }
    if goal_accumulator.is_empty() {
        None
    } else {
        Some(goal_accumulator)
    }
}

#[allow(unused)]
pub fn bfs_closest_keys(
    start: usize,
    graph: &[Vec<u8>],
    keys: &[bool],
    ignore_doors: bool,
) -> Vec<(usize, usize, usize)> {
    let matching = |current: usize, _: usize, _: &[Vec<u8>], keys: &[bool]| keys[current];
    let action = |current: usize, graph: &[Vec<u8>], came_from: &[Option<usize>], _: bool| {
        BfsActionResult::Accumulate((
            current,
            traverse_path(came_from, current).len() - 1,
            count_doors(graph, came_from, current),
        ))
    };
    let mut res = bfs(
        start,
        0,
        graph,
        keys,
        Box::new(matching),
        Box::new(action),
        ignore_doors,
    );
    res.get_or_insert(vec![]).to_owned()
}

#[allow(unused)]
pub fn bfs_closest_key(
    start: usize,
    graph: &[Vec<u8>],
    keys: &[bool],
    ignore_doors: bool,
) -> Option<(usize, usize, usize)> {
    let matching = |current: usize, _: usize, _: &[Vec<u8>], keys: &[bool]| keys[current];
    let action = |current: usize, graph: &[Vec<u8>], came_from: &[Option<usize>], _: bool| {
        BfsActionResult::Return(vec![(
            current,
            traverse_path(came_from, current).len() - 1,
            count_doors(graph, came_from, current),
        )])
    };
    match bfs(
        start,
        0,
        graph,
        keys,
        Box::new(matching),
        Box::new(action),
        ignore_doors,
    ) {
        Some(vec) => vec.into_iter().next(),
        None => None,
    }
}

#[allow(unused)]
pub fn bfs_shortest_path(
    start: usize,
    end: usize,
    graph: &[Vec<u8>],
    _: &[bool],
    ignore_doors: bool,
) -> Option<Vec<usize>> {
    let matching = |current: usize, end: usize, _: &[Vec<u8>], _: &[bool]| current == end;
    let action = |current: usize, _: &[Vec<u8>], came_from: &[Option<usize>], _: bool| {
        BfsActionResult::Return(traverse_path(came_from, current))
    };
    bfs(
        start,
        end,
        graph,
        &[],
        Box::new(matching),
        Box::new(action),
        ignore_doors,
    )
}

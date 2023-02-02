use super::COLUMN_SIZE;
use std::collections::{BinaryHeap, VecDeque};

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

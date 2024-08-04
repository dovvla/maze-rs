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

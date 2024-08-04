
# Maze-rs

## Overview

Maze-rs is a Rust-based application for parallelized maze solving. It leverages Rust's concurrency capabilities to efficiently solve mazes, making it a useful tool for exploring parallel algorithms and Rust programming.

Features:

- Parallelized Solving: Utilizes Rust's concurrency features to solve mazes in parallel.
    
- Customizable Mazes: Allows users to input their own maze designs for solving.
    
- Rust Implementation: Provides a practical example of Rust's power and efficiency in handling concurrent tasks.

## Installation

Clone the repository:

```sh
git clone https://github.com/dovvla/maze-rs.git
cd maze-rs
```

Build the project:

```sh
cargo build --release
```
Run the maze solver with the included example maze:
```sh
cargo run --release
```
To use a custom maze, modify the labyrinth.txt file with your maze design and then run the solver.

This project is licensed under the MIT License. See the LICENSE file for details.

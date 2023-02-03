use std::{error::Error, str::FromStr};

use pathfinder::parallel_backtrack;
const COLUMN_SIZE: usize = 9;

mod pathfinder;

struct Labyrinth(Vec<Vec<Field>>);

#[derive(Debug, Default, Clone, Copy)]
#[allow(unused)]
struct Directions<T> {
    west: T,
    east: T,
    north: T,
    south: T,
}

#[derive(Debug, Default, Clone, Copy)]
#[allow(unused)]
struct Field {
    paths: Directions<bool>,
    doors: Directions<bool>,
    contains_key: bool,
    is_end: bool,
}

impl FromStr for Directions<bool> {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! bit_match {
            ($slice:ident[$index:literal] -> $var:expr) => {
                match $slice.chars().nth($index) {
                    Some('0') => $var = false,
                    Some('1') => $var = true,
                    _ => return Err(()),
                }
            };
        }
        if s.len() > 4 {
            Err(())
        } else {
            let mut dir = Self::default();
            bit_match!(s[0] -> dir.west);
            bit_match!(s[1] -> dir.east);
            bit_match!(s[2] -> dir.north);
            bit_match!(s[3] -> dir.south);
            Ok(dir)
        }
    }
}

fn str_bitwise_and(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c == '1')
}

impl FromStr for Field {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 14 {
            return Err(());
        }
        Ok(Self {
            paths: Directions::from_str(&s[0..4])?,
            doors: Directions::from_str(&s[5..9])?,
            contains_key: str_bitwise_and(&s[10..12]),
            is_end: str_bitwise_and(&s[12..14]),
        })
    }
}

fn read_file(path: &str) -> Result<Labyrinth, Box<dyn Error>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file = File::open(path)?;
    let reader = BufReader::new(file).lines();
    let lines: Vec<String> = reader.map(|l| l.expect("Error reading line")).collect();
    let fields: Vec<Field> = lines
        .into_iter()
        .map(|l| Field::from_str(&l).expect("Error parsing line"))
        .collect();
    let row_count = fields.len() / COLUMN_SIZE;
    let mut labyrinth = vec![vec![Field::default(); COLUMN_SIZE]; row_count];
    for r in 0..row_count {
        for c in 0..COLUMN_SIZE {
            labyrinth[r][c] = fields[r * COLUMN_SIZE + c];
        }
    }
    Ok(Labyrinth(labyrinth))
}

fn display_labyrinth(lab: &Labyrinth) {
    let fields = &lab.0;
    #[allow(clippy::needless_range_loop)]
    for row in 0..fields.len() {
        for line in 1..=4 {
            for col in 0..COLUMN_SIZE {
                if (!fields[row][col].paths.north)
                    && !fields[row][col].paths.south
                    && !fields[row][col].paths.east
                    && !fields[row][col].paths.west
                    && !fields[row][col].doors.north
                    && !fields[row][col].doors.south
                    && !fields[row][col].doors.east
                    && !fields[row][col].doors.west
                {
                    print!("      ");
                    continue;
                }
                match line {
                    1 => print!(
                        "┏━{}━┓",
                        if fields[row][col].doors.north {
                            "╩╩"
                        } else if fields[row][col].paths.north {
                            "┛┗"
                        } else {
                            "━━"
                        }
                    ),
                    2 => print!(
                        "{} {} {}",
                        if fields[row][col].doors.west {
                            "╣"
                        } else if fields[row][col].paths.west {
                            "┛"
                        } else {
                            "┃"
                        },
                        if fields[row][col].is_end {
                            "🚩"
                        } else if fields[row][col].contains_key {
                            "🗝️ "
                        } else {
                            "  "
                        },
                        if fields[row][col].doors.east {
                            "╠"
                        } else if fields[row][col].paths.east {
                            "┗"
                        } else {
                            "┃"
                        },
                    ),
                    3 => print!(
                        "{} {:2} {}",
                        if fields[row][col].doors.west {
                            "╣"
                        } else if fields[row][col].paths.west {
                            "┓"
                        } else {
                            "┃"
                        },
                        row * COLUMN_SIZE + col,
                        if fields[row][col].doors.east {
                            "╠"
                        } else if fields[row][col].paths.east {
                            "┏"
                        } else {
                            "┃"
                        },
                    ),
                    4 => print!(
                        "┗━{}━┛",
                        if fields[row][col].doors.south {
                            "╦╦"
                        } else if fields[row][col].paths.south {
                            "┓┏"
                        } else {
                            "━━"
                        }
                    ),
                    _ => {}
                }
            }
            println!()
        }
    }
}

impl Labyrinth {
    fn pathfind_matrix(&self) -> (Vec<Vec<u8>>, Vec<bool>, Vec<bool>) {
        macro_rules! dim {
            ($row:expr, $col:expr) => {
                $row * COLUMN_SIZE + $col
            };
            (len) => {
                self.0.len() * COLUMN_SIZE
            };
        }
        let mut path_matrix = vec![vec![0u8; dim![len]]; dim![len]];
        let mut key_vector = vec![false; dim![len]];
        let mut end_vector = vec![false; dim![len]];
        for r in 0..self.0.len() {
            for c in 0..COLUMN_SIZE {
                if self.0[r][c].paths.north && r > 0 {
                    path_matrix[dim![r, c]][dim![r - 1, c]] += 2;
                    path_matrix[dim![r - 1, c]][dim![r, c]] += 2;
                }
                if self.0[r][c].paths.south && r < self.0.len() - 1 {
                    path_matrix[dim![r, c]][dim![r + 1, c]] += 2;
                    path_matrix[dim![r + 1, c]][dim![r, c]] += 2;
                }
                if self.0[r][c].paths.west && c > 0 {
                    path_matrix[dim![r, c]][dim![r, c - 1]] += 2;
                    path_matrix[dim![r, c - 1]][dim![r, c]] += 2;
                }
                if self.0[r][c].paths.east && c < COLUMN_SIZE - 1 {
                    path_matrix[dim![r, c]][dim![r, c + 1]] += 2;
                    path_matrix[dim![r, c + 1]][dim![r, c]] += 2;
                }
                /* */
                if self.0[r][c].doors.north && r > 0 {
                    path_matrix[dim![r, c]][dim![r - 1, c]] += 3;
                    path_matrix[dim![r - 1, c]][dim![r, c]] += 3;
                }
                if self.0[r][c].doors.south && r < self.0.len() - 1 {
                    path_matrix[dim![r, c]][dim![r + 1, c]] += 3;
                    path_matrix[dim![r + 1, c]][dim![r, c]] += 3;
                }
                if self.0[r][c].doors.west && c > 0 {
                    path_matrix[dim![r, c]][dim![r, c - 1]] += 3;
                    path_matrix[dim![r, c - 1]][dim![r, c]] += 3;
                }
                if self.0[r][c].doors.east && c < COLUMN_SIZE - 1 {
                    path_matrix[dim![r, c]][dim![r, c + 1]] += 3;
                    path_matrix[dim![r, c + 1]][dim![r, c]] += 3;
                }
                /* */
                if self.0[r][c].contains_key {
                    key_vector[dim![r, c]] = true;
                }
                if self.0[r][c].is_end {
                    end_vector[dim![r, c]] = true;
                }
            }
        }

        // print!["   "];
        // for i in 0..dim![len] {
        //     print!["{:2} ", i + 1];
        // }
        // println![];
        #[allow(clippy::needless_range_loop)]
        for i in 0..dim![len] {
            // print!["{:2} ", i + 1];
            for j in 0..dim![len] {
                path_matrix[i][j] = match path_matrix[i][j] {
                    4 => 1,   // path
                    7 => 255, // door
                    _ => 0,   // wall
                };
                // print![
                //     " {} ",
                //     match path_matrix[i][j] {
                //         1 => {
                //             "#"
                //         }
                //         255 => {
                //             "D"
                //         }
                //         _ => {
                //             "·"
                //         }
                //     }
                // ];
            }
            // println![];
        }

        (path_matrix, key_vector, end_vector)
    }

    fn pathfind(&self, start: usize, end: usize) -> Option<Vec<usize>> {
        let (mut maze, mut keys, _) = self.pathfind_matrix();
        let mut whole_path: Vec<usize> = vec![];
        let mut key_inventory = 0isize;

        let mut start = start;
        loop {
            let (ideal_path, consumed) = pathfinder::a_star(start, end, &maze)?;
            let cumsum = pathfinder::key_cumsum(&ideal_path, &consumed, &keys);
            let (pickup_path, kinv) =
                pathfinder::key_pickup(&ideal_path, &cumsum, &mut maze, &mut keys, key_inventory)?;
            key_inventory = kinv;
            if pickup_path.is_empty() {
                whole_path.extend(ideal_path);
                break;
            } else {
                start = pickup_path[pickup_path.len() - 1];
                whole_path.extend(pickup_path);
            }
        }
        Some(pathfinder::deduplicate_path(&whole_path))
    }

    #[allow(unused)]
    fn pathfind_parallel(&self, start: usize, end: usize) -> Option<Vec<usize>> {
        let (mut maze, mut keys, _) = self.pathfind_matrix();
        let mut start = start;
        pathfinder::parallel_backtrack(start, end, &maze, &keys);
        None
    }
}

use std::time::Instant;
fn main() -> Result<(), Box<dyn Error>> {
    let l = read_file("./labyrinth.txt")?;
    display_labyrinth(&l);
    let now = Instant::now();

    let pp = l.pathfind_parallel(0, 47);

    let elapsed = now.elapsed();
    println!("Time for parallel exec {:?}", elapsed);
    let now = Instant::now();

    let p = l.pathfind(0, 47);
    let elapsed = now.elapsed();

    println!("Time for normal exec {:?}", elapsed);

    println!["{:?}", p.unwrap()];
    println!["{pp:?}"];

    Ok(())
}

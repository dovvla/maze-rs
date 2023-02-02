use std::{error::Error, str::FromStr};
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

fn main() -> Result<(), Box<dyn Error>> {
    let l = read_file("./labyrinth.txt")?;
    display_labyrinth(&l);
    Ok(())
}

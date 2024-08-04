use std::{error::Error, str::FromStr};
const COLUMN_SIZE: usize = 9;
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
fn main() -> Result<(), Box<dyn Error>> {
    let l = read_file("./labyrinth.txt")?;
    Ok(())
}

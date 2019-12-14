use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::error::Error;
use std::collections::HashMap;
use std::fmt;
use std::cmp::min;

#[derive(Clone, Debug, Hash, PartialEq)]
struct Position {
    pub x: i32,
    pub y: i32,
}

impl Eq for Position {}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    // used for debugging
    fn grid_char(&self) -> char {
        match self {
            Direction::Up => '|',
            Direction::Down => '|',
            Direction::Left => '-',
            Direction::Right => '-',
        }
    }
}

impl From<char> for Direction {
    fn from(c: char) -> Self {
        match c {
            'U' => Direction::Up,
            'D' => Direction::Down,
            'L' => Direction::Left,
            'R' => Direction::Right,
            _ => panic!("Invalid direction {}", c)
        }
    }
}

struct Grid {
    origin: Position,
    values: HashMap<Position, char>,
    line_count: u32,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

impl Grid {
    fn new() -> Self {
       let mut rv = Grid {
           origin: Position { x:0, y:0},
           values: HashMap::new(),
           line_count: 1,
           min_x: 0,
           min_y: 0,
           max_x: 0,
           max_y: 0,
       };

        rv.values.insert(rv.origin.clone(), 'O');
        return rv;
    }

    fn add_line(&mut self, line: &Vec<&str>) {
        let mut current_position = self.origin.clone();
        let mut current_direction: Direction = line[0].chars().nth(0).unwrap().into();

        for movement in line.iter() {
            let (direction_char, length) = movement.split_at(1);
            let length: u32 = length.parse().unwrap();
            let new_direction: Direction = direction_char.chars().nth(0).unwrap().into();
            if new_direction != current_direction {
                self.insert(&current_position, '+');
            }

            current_direction = new_direction;
            for _ in 0..length {
                let next_position = next_position(&current_position, &current_direction);
                self.insert(&next_position, current_direction.grid_char());
                current_position = next_position;
            }
            //println!("New grid:\n{:}", self);
        }
        self.line_count += 1;
    }

    fn insert(&mut self, position: &Position, value: char) {
        if position.x < self.min_x {
            self.min_x = position.x.clone();
        } else if position.x > self.max_x {
            self.max_x = position.x.clone();
        }

        if position.y < self.min_y {
            self.min_y = position.y.clone();
        } else if position.y > self.max_y {
            self.max_y = position.y.clone();
        }

        self.values.insert(position.clone(), value);
    }

    fn get_char_at_position(&self, position: &Position) -> char {
        match self.values.get(position) {
            Some(&v) => v,
            _ => '.',
        }
    }

    fn get_intersections(&mut self, line: &Vec<&str>) -> Vec<Position> {
        let mut rv: Vec<Position> = Vec::new();

        let mut current_position = self.origin.clone();
        let mut current_direction;

        for movement in line.iter() {
            let (direction_char, length) = movement.split_at(1);
            let length: u32 = length.parse().unwrap();
            let new_direction: Direction = direction_char.chars().nth(0).unwrap().into();

            current_direction = new_direction;
            for _ in 0..length {
                let next_position = next_position(&current_position, &current_direction);
                let value_at_next_position = self.get_char_at_position(&next_position);
                if value_at_next_position != '.' {
                    self.insert(&next_position, 'X');
                    rv.push(next_position.clone());
                } else {
                    self.insert(&next_position, current_direction.grid_char());
                }
                current_position = next_position;
            }
        }

        rv
    }
}

// used for debugging
impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in self.min_y..=self.max_y {
            for x in self.min_x..=self.max_x {
                let p = Position{x,y};
                write!(f, "{}", self.get_char_at_position(&p))?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}

fn next_position(current_position: &Position, direction: &Direction) -> Position {
    match direction {
        Direction::Up => Position{x: current_position.x.clone(), y:current_position.y.clone() + 1},
        Direction::Down => Position{x: current_position.x.clone(), y:current_position.y.clone() - 1},
        Direction::Left => Position{x: current_position.x.clone() - 1, y:current_position.y.clone()},
        Direction::Right => Position{x: current_position.x.clone() + 1, y:current_position.y.clone()},
    }
}

fn get_manhattan_dist_from_origin(pos: &Position) -> u32 {
    return pos.x.abs() as u32 + pos.y.abs() as u32;
}

fn main() -> Result<(),Box<dyn Error>> {
    let input = File::open("../input.txt")?;
    let mut reader = BufReader::new(input);

    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    let first_line: Vec<&str> = first_line.trim_end().split(',').collect::<Vec<&str>>();

    let mut second_line = String::new();
    reader.read_line(&mut second_line)?;
    let second_line: Vec<&str> = second_line.trim_end().split(',').collect::<Vec<&str>>();

    let mut grid = Grid::new();

    grid.add_line(&first_line);
    let intersections = grid.get_intersections(&second_line);

    let mut min_dist = get_manhattan_dist_from_origin(&intersections[0]);

    for intersection_point in intersections.iter().skip(1) {
       let curr_dist = get_manhattan_dist_from_origin(&intersection_point);
        min_dist = min(min_dist, curr_dist);
    }

    println!("{}", min_dist);

    return Ok(());
}
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::error::Error;
use std::collections::HashSet;
use std::iter::FromIterator;
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

fn generate_path(line: &Vec<&str>) -> Vec<Position> {
    let mut rv = Vec::new();

    let mut current_position = Position{x:0, y:0};
    let mut current_direction;

    for movement in line.iter() {
        let (direction_char, length) = movement.split_at(1);
        let length: u32 = length.parse().unwrap();
        let new_direction: Direction = direction_char.chars().nth(0).unwrap().into();

        current_direction = new_direction;
        for _ in 0..length {
            let next_position = next_position(&current_position, &current_direction);
            rv.push(next_position.clone());
            current_position = next_position;
        }
    }

    rv
}

fn get_intersections(first: &Vec<Position>, second: &Vec<Position>) -> HashSet<Position> {
    let mut rv = HashSet::new();
    let first_positions: HashSet<Position> = HashSet::from_iter(first.iter().cloned());

    for position in second.iter() {
        if let Some(p) = first_positions.get(position) {
            rv.insert(p.clone());
        }
    }

    rv
}

fn next_position(current_position: &Position, direction: &Direction) -> Position {
    match direction {
        Direction::Up => Position{x: current_position.x.clone(), y:current_position.y.clone() + 1},
        Direction::Down => Position{x: current_position.x.clone(), y:current_position.y.clone() - 1},
        Direction::Left => Position{x: current_position.x.clone() - 1, y:current_position.y.clone()},
        Direction::Right => Position{x: current_position.x.clone() + 1, y:current_position.y.clone()},
    }
}

fn get_path_dist_to_position(path: &Vec<Position>, target: &Position) -> usize {
    for(step, position) in path.iter().enumerate() {
        if position == target {
            return step + 1; // + 1 because origin skipped when generating path
        }
    }

    panic!("Didn't find target position in path")
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

    let first_path = generate_path(&first_line);
    let second_path = generate_path(&second_line);
    let intersections = get_intersections(&first_path, &second_path);

    let first_intersection = intersections.iter().next().unwrap();

    let mut min_dist = get_path_dist_to_position(&first_path, &first_intersection) +
                           get_path_dist_to_position(&second_path, &first_intersection);

    for intersection in intersections.iter().skip(1) {
        let curr_dist = get_path_dist_to_position(&first_path, intersection) +
            get_path_dist_to_position(&second_path, intersection);
        min_dist = min(min_dist, curr_dist);
    }

    println!("{}", min_dist);

    return Ok(());
}
use std::fs::File;
use std::io::{BufRead,BufReader};
use std::error::Error;
use std::collections::HashSet;
use std::cmp::{min,max};

#[derive(Debug, Hash, PartialEq, Clone)]
struct Position(i32, i32);
impl Eq for Position {}

#[derive(Debug, PartialEq)]
struct Slope(i32, i32);

#[derive(Debug)]
struct AsteroidField {
    grid: Vec<Vec<char>>,
    asteroid_locations: HashSet<Position>,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let tmp = a;
        a = b;
        b = tmp % b;
    }
    a
}

impl AsteroidField {
    fn new(input: Vec<Vec<char>>) -> Self {
        let mut min_x = (input.len()*input[0].len()) as i32;
        let mut max_x = 0;
        let mut min_y = (input.len()*input[0].len()) as i32;
        let mut max_y = 0;
        AsteroidField {
            asteroid_locations: input.iter().flat_map(|row| row.iter()).enumerate().filter(|(_, c)| c != &&'.').map(|(i, _)| {
                let i = i as i32;
                let x = i%input.len() as i32;
                let y = i/input.len() as i32;
                min_x = min(min_x, x);
                min_y = min(min_y, y);

                max_x = max(max_x, x);
                max_y = max(max_y, y);

                Position(i/input.len() as i32, i%input.len() as i32)
            }).collect(),
            grid: input,
            min_x, min_y, max_x, max_y
        }
    }

    fn first_asteroid_following_slope(&self, start: &Position, slope: &Slope) -> &Position {
        let mut current_pos = start.next_position(&slope);

        while current_pos.0 <= self.max_x && current_pos.0 >= self.min_x && current_pos.1 <= self.max_y && current_pos.1 >= self.min_y {
            if let Some(pos) = self.asteroid_locations.get(&current_pos) {
                return pos;
            } else {
                current_pos = current_pos.next_position(&slope);
            }
        }

        // in this code, the slopes are always computed using another asteroid as the second
        // point, so we should always hit another asteroid
        unreachable!();
    }
}

impl Position {
    fn compute_slope(&self, other: &Position) -> Slope {
        let numerator = other.0 - self.0;
        let denominator = other.1 - self.1;
        let gcd = gcd(numerator.abs(),denominator.abs()) as i32;
        // return simplified slope
        Slope(numerator/gcd, denominator/gcd)
    }

    fn next_position(&self, slope: &Slope) -> Position {
        Position(self.0+slope.0, self.1+slope.1)
    }
}

fn main() -> Result<(), Box<dyn Error>>{
    let f = File::open("../input.txt").expect("couldn't open input");
    let f = BufReader::new(f);

    let mut grid = Vec::new();

    for line in f.lines() {
        let mut row = Vec::new();
        for c in line?.chars() {
            row.push(c);
        }
        grid.push(row);
    }

    let af = AsteroidField::new(grid);

    let mut max_asteroids_seen = 0;

    for asteroid_pos1 in af.asteroid_locations.iter() {
        let mut asteroids_seen_from_here = HashSet::new();
        for asteroid_pos2 in af.asteroid_locations.iter() {
            if asteroid_pos1 != asteroid_pos2 {
                let slope = asteroid_pos1.compute_slope(asteroid_pos2);
                let next_asteroid = af.first_asteroid_following_slope(&asteroid_pos1, &slope);
                asteroids_seen_from_here.insert(next_asteroid.clone());
            }
        }
        max_asteroids_seen = max(max_asteroids_seen, asteroids_seen_from_here.len());
    }

    println!("{}", max_asteroids_seen);

    Ok(())
}

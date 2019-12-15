use std::fs::File;
use std::io::{BufRead,BufReader};
use std::error::Error;
use std::collections::{BinaryHeap,HashSet};
use std::cmp::{min,max,Reverse};
use std::f64::consts::PI;
use std::cmp::Ordering;

#[derive(Debug, Hash, PartialEq, Clone)]
struct Position {
    x: i32,
    y: i32,
}
impl Eq for Position {}

#[derive(Debug, PartialEq)]
struct Slope {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct AsteroidField {
    grid: Vec<Vec<char>>,
    asteroid_locations: HashSet<Position>,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

#[derive(PartialEq, Debug)]
struct AsteroidWithAngle(f64, Position);
impl Eq for AsteroidWithAngle {}
impl PartialOrd for AsteroidWithAngle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 < other.0 {
            Some(Ordering::Less)
        } else if self.0 > other.0 {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}
impl Ord for AsteroidWithAngle {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 < other.0 {
            Ordering::Less
        } else if self.0 > other.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
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
        let mut min_x = max(input.len(),input[0].len()) as i32;
        let mut max_x = 0;
        let mut min_y = max(input.len(),input[0].len()) as i32;
        let mut max_y = 0;
        AsteroidField {
            asteroid_locations: input.iter().flat_map(|row| row.iter()).enumerate().filter(|(_, c)| c != &&'.').map(|(i, _)| {
                let i = i as i32;
                let x = i%input[0].len() as i32;
                let y = i/input[0].len() as i32;
                min_x = min(min_x, x);
                min_y = min(min_y, y);

                max_x = max(max_x, x);
                max_y = max(max_y, y);

                Position{y, x}
            }).collect(),
            grid: input,
            min_x, min_y, max_x, max_y,
        }
    }

    fn first_asteroid_following_slope(&self, start: &Position, slope: &Slope) -> &Position {
        let mut current_pos = start.next_position(&slope);

        while current_pos.x <= self.max_x && current_pos.x >= self.min_x && current_pos.y <= self.max_y && current_pos.y >= self.min_y {
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

    fn get_asteroids_seen_from_position(&self, anchor: &Position) -> HashSet<Position> {
        let mut asteroids_seen_from_here = HashSet::new();

        for other_pos in self.asteroid_locations.iter() {
            if other_pos != anchor {
                let slope = anchor.compute_slope(other_pos);
                let next_asteroid = self.first_asteroid_following_slope(&anchor, &slope);
                asteroids_seen_from_here.insert(next_asteroid.clone());
            }
        }

        return asteroids_seen_from_here;
    }

    fn get_best_position(&self) -> Position {
        let mut max_asteroids_seen = 0;
        let mut best_position = Position{y:0, x:0};

        for pos in self.asteroid_locations.iter() {
            let asteroids_seen = self.get_asteroids_seen_from_position(pos);
            if asteroids_seen.len() > max_asteroids_seen {
                max_asteroids_seen = asteroids_seen.len();
                best_position = pos.clone();
            }
        }

        best_position
    }

    // just blast all asteroids so don't have to keep track of blaster angle
    fn blast_all_visible_asteroids(&mut self, anchor: &Position) -> Vec::<Position> {
        let blastable_asteroids = self.get_asteroids_seen_from_position(anchor);
        let mut sorted_asteroids_with_angles = BinaryHeap::new();
        let mut destroyed_asteroids = Vec::new();

        for asteroid in blastable_asteroids.iter() {
            let x = f64::from(asteroid.x - anchor.x);
            let y = -f64::from(asteroid.y - anchor.y);
            let mut angle = y.atan2(x) - PI/2f64; // subtract pi/2 because blaster will point directly up after every iteration
            // ensure angle positive
            if angle < 0f64 {
                angle += 2f64*PI;
            }
            // 2PI - current angle because blaster rotates clockwise
            angle = 2f64*PI - angle;
            // 2PI - current angle doesn't work if angle is 0.  If angle should be 0, set to 0
            angle = if asteroid.x == anchor.x {
                        0f64
                    } else {
                        angle
            };
            let angle = AsteroidWithAngle(angle, asteroid.clone());
            sorted_asteroids_with_angles.push(Reverse(angle)); // reverse so it will be min heap
        }

        while sorted_asteroids_with_angles.len() > 0 {
            let Reverse(asteroid_with_angle) = sorted_asteroids_with_angles.pop().unwrap();
            destroyed_asteroids.push(asteroid_with_angle.1.clone());
            self.asteroid_locations.remove(&asteroid_with_angle.1);
        }

       destroyed_asteroids
    }
}

impl Position {
    fn compute_slope(&self, other: &Position) -> Slope {
        let delta_x = other.x - self.x;
        let delta_y = other.y - self.y;
        let gcd = gcd(delta_x .abs(),delta_y.abs()) as i32;
        // return simplified slope
        Slope{y:delta_y /gcd, x: delta_x/gcd}
    }

    fn next_position(&self, slope: &Slope) -> Position {
        Position{x: self.x+slope.x, y: self.y+slope.y}
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

    let mut af = AsteroidField::new(grid);

    let best_pos = af.get_best_position();
    let mut destroyed_asteroids_count = 0;
    loop {
        let destroyed_asteroids = af.blast_all_visible_asteroids(&best_pos);
        for asteroid in destroyed_asteroids.iter() {
            destroyed_asteroids_count += 1;
            if destroyed_asteroids_count == 200 {
                println!("{}", asteroid.x*100+asteroid.y);
                return Ok(());
            }
        }
    }
}

use std::fs::File;
use std::io::Read;
use std::collections::{HashMap,HashSet};
use std::thread;
use std::sync::mpsc;
use std::cmp::{min,max};
use std::fmt;

extern crate pathfinding;
use pathfinding::utils::absdiff;
use pathfinding::prelude::{astar, bfs};

#[derive(Debug)]
enum ParameterMode {
    Position,
    Immediate,
    Relative
}

impl From<char> for ParameterMode {
    fn from(num: char) -> ParameterMode {
        match num {
            '0' => ParameterMode::Position,
            '1' => ParameterMode::Immediate,
            '2' => ParameterMode::Relative,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Hash, Clone, Copy)]
struct Position(i64,i64);

impl Position {
    fn distance(&self, other: &Self) -> u32 {
        absdiff(self.0, other.0) as u32 + absdiff(self.1, other.1) as u32
    }

    fn move_to_other(&self, other: &Self) -> MovementCommand {
        if *self == Position(other.0 - 1, other.1) {
            return MovementCommand::East;
        } else if *self == Position(other.0 + 1, other.1) {
            return MovementCommand::West;
        } else if *self == Position(other.0, other.1 + 1) {
            return MovementCommand::South;
        } else if *self == Position(other.0, other.1 - 1) {
            return MovementCommand::North;
        }

        // should only be called with adjacent positions
        unreachable!()
    }
}

struct Grid {
    known_positions: HashMap<Position, StatusCode>,
    robot_position: Position,
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64
}

impl Grid {
    fn new() -> Self {
        let mut g = Grid {
            known_positions: HashMap::new(),
            robot_position: Position(0,0),
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
        };
        g.known_positions.insert(Position(0,0), StatusCode::Ok);
        return g;
    }

    fn successors(&self, p: &Position) -> Vec<(Position, u32)> {
        let mut rv = Vec::new();
        // check 4 directions
        // each direction can be a successor as long as no wall present

        { // scopes to prevent accidentally using same position twice when copy/pasting
            let up_pos = Position( p.0, p.1 + 1 );
            if *self.known_positions.get(&up_pos).or(Some(&StatusCode::Unknown)).unwrap() != StatusCode::Wall {
                rv.push((up_pos, 1));
            }
        }

        {
            let down_pos = Position( p.0, p.1 - 1 );
            if *self.known_positions.get(&down_pos).or(Some(&StatusCode::Unknown)).unwrap() != StatusCode::Wall {
                rv.push((down_pos, 1));
            }
        }

        {
            let left_pos = Position( p.0 - 1, p.1 );
            if *self.known_positions.get(&left_pos).or(Some(&StatusCode::Unknown)).unwrap() != StatusCode::Wall {
                rv.push((left_pos, 1));
            }
        }

        {
            let right_pos = Position( p.0 + 1, p.1 );
            if *self.known_positions.get(&right_pos).or(Some(&StatusCode::Unknown)).unwrap() != StatusCode::Wall {
                rv.push((right_pos, 1));
            }
        }

        rv
    }

    fn move_robot_to_target(&mut self, target: &Position, robot_tx: &mpsc::Sender<i64>, robot_rx: &mpsc::Receiver<i64>, draw: bool) -> StatusCode {
        let mut s: StatusCode;

        'find_target: loop {
            let path = astar(&self.robot_position, |p| self.successors(p), |p| p.distance(&target), |p| p == target).unwrap();

            // if there are any unknowns, can only move to first one before recalculating path
            // because revealing can show a path is no longer valid
            for path_pos in path.0.iter().skip(1) {
                let movement = self.robot_position.move_to_other(path_pos);
                robot_tx.send(movement.into()).unwrap();
                s = robot_rx.recv().unwrap().into();

                if draw {
                    self.draw();
                }

                if s != StatusCode::Wall {
                    self.robot_position = path_pos.clone();
                }

                if self.known_positions.get(&path_pos).is_none() {
                    self.known_positions.insert(path_pos.clone(), s.clone());
                    self.min_x = min(self.min_x, path_pos.0);
                    self.min_y = min(self.min_y, path_pos.1);
                    self.max_x = max(self.max_x, path_pos.0);
                    self.max_y = max(self.max_y, path_pos.1);

                    // first time finding this target
                    if path_pos == target {
                        break 'find_target;
                    } else {
                        // found an unknown that isn't target
                        // recompute path
                        continue 'find_target;
                    }
                } else if path_pos == target {
                    break 'find_target;
                }
            }
        }

        s
    }

    fn simulate_oxygen(&mut self, draw: bool) -> bool {
        let mut get_oxygen_this_tick = HashSet::new();

        for x in self.min_x..=self.max_x {
            for y in self.min_y..=self.max_y {
                let s = self.known_positions.get(&Position(x,y));
                if let Some(status) = s {
                    if *status == StatusCode::Oxygen {
                        // send oxygen to adjacent cells that are empty

                        {
                            let up_pos = Position(x, y + 1);
                            if *self.known_positions.get(&up_pos).or(Some(&StatusCode::Unknown)).unwrap() == StatusCode::Ok {
                                get_oxygen_this_tick.insert(up_pos);
                            }
                        }

                        {
                            let down_pos = Position(x, y - 1);
                            if *self.known_positions.get(&down_pos).or(Some(&StatusCode::Unknown)).unwrap() == StatusCode::Ok {
                                get_oxygen_this_tick.insert(down_pos);
                            }
                        }

                        {
                            let left_pos = Position(x - 1, y);
                            if *self.known_positions.get(&left_pos).or(Some(&StatusCode::Unknown)).unwrap() == StatusCode::Ok {
                                get_oxygen_this_tick.insert(left_pos);
                            }
                        }

                        {
                            let right_pos = Position(x + 1, y);
                            if *self.known_positions.get(&right_pos).or(Some(&StatusCode::Unknown)).unwrap() == StatusCode::Ok {
                                get_oxygen_this_tick.insert(right_pos);
                            }
                        }
                    }
                }
            }
        }

        for pos in get_oxygen_this_tick.iter() {
            self.known_positions.insert(pos.clone(), StatusCode::Oxygen);
        }

        if draw {
            self.draw();
//            std::io::stdin().read(&mut [0u8]).unwrap();
        }

        get_oxygen_this_tick.len() == 0
    }

    fn draw(&self) {
        println!("-----------------------------------------------------");
        for y in (self.min_y..=self.max_y).rev() {
            for x in self.min_x..=self.max_x {
                if x == self.robot_position.0 && y == self.robot_position.1 {
                    print!("D");
                } else if x == 0 && y == 0 {
                    print!("X")
                } else {
                    print!("{:?}", self.known_positions.get(&Position(x, y)).or(Some(&StatusCode::Unknown)).unwrap());
                }
            }
            println!();
        }
        std::thread::sleep(std::time::Duration::from_millis(50));

    }
}

#[derive(Debug)]
enum MovementCommand {
    North,
    South,
    East,
    West
}

impl Into<i64> for MovementCommand {
    fn into(self) -> i64 {
        match self {
            MovementCommand::North => 1,
            MovementCommand::South => 2,
            MovementCommand::West => 3,
            MovementCommand::East => 4,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum StatusCode {
    Unknown,
    Wall,
    Ok,
    Oxygen
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusCode::Wall => write!(f, "#"),
            StatusCode::Oxygen => write!(f, "O"),
            StatusCode::Unknown => write!(f, " "),
            StatusCode::Ok => write!(f, "."),
        }
    }
}

impl From<i64> for StatusCode {
    fn from(i: i64) -> Self {
        match i {
            -1 => {
                StatusCode::Unknown
            },
            0 => {
                StatusCode::Wall
            },
            1 => {
                StatusCode::Ok
            },
            2 => {
                StatusCode::Oxygen
            },
            _ => {
                unreachable!()
            }
        }
    }
}

fn get_value(h: &HashMap<i64, i64>, i: i64, p: &ParameterMode, relative_base: i64) -> i64 {
    match p {
        ParameterMode::Position => {
            *h.get(h.get(&i).unwrap()).or(Some(&0)).unwrap()
        },
        ParameterMode::Immediate => {
            *h.get(&i).or(Some(&0)).unwrap()
        },
        ParameterMode::Relative => {
            *h.get(&(*h.get(&i).or(Some(&0)).unwrap() + relative_base)).or(Some(&0)).unwrap()
        }
    }
}

fn store_value(h: &mut HashMap<i64, i64>, i: i64, value: i64, p: &ParameterMode, relative_base: i64) {
    match p {
        ParameterMode::Position => {
            h.insert(*h.get(&i).or(Some(&0)).unwrap(), value);
        },
        ParameterMode::Relative => {
            h.insert(*h.get(&i).or(Some(&0)).unwrap() + relative_base, value);
        },
        _ => unreachable!() // immediate never given for storing values
    }
}

fn run_robot(h: &mut HashMap<i64, i64>, input: mpsc::Receiver<i64>, output: mpsc::Sender<i64>) {
    let mut current_op_index: i64 = 0;
    let mut relative_base = 0;

    while current_op_index < h.len() as i64 {
        let op  = *h.get(&current_op_index).unwrap();
        let mut op_digits: Vec<char> = op.to_string().chars().collect();

        while op_digits.len() < 5 {
            op_digits.insert(0,'0');
        }


        let mut op_code = String::new();
        op_code.push(op_digits[3]);
        op_code.push(op_digits[4]);

        let param_first = ParameterMode::from(op_digits[2]);
        let param_second = ParameterMode::from(op_digits[1]);
        let param_third = ParameterMode::from(op_digits[0]);

        if op_code == "01" {
            let first_value = get_value(&h, current_op_index+1, &param_first, relative_base);
            let second_value = get_value(&h, current_op_index+2, &param_second, relative_base);

            store_value(h, current_op_index+3, first_value+second_value, &param_third, relative_base);
            current_op_index += 4;
        } else if op_code == "02" {
            let first_value = get_value(&h, current_op_index+1, &param_first, relative_base);
            let second_value = get_value(&h, current_op_index+2, &param_second, relative_base);

            store_value(h, current_op_index+3, first_value*second_value, &param_third, relative_base);
            current_op_index += 4;
        } else if op_code == "03" {
            let value = input.recv();
            if let Err(_) = value {
                // program over, kill robot sim thread
                return;
            } else {
                store_value(h, current_op_index + 1, value.unwrap(), &param_first, relative_base);
                current_op_index += 2;
            }
        } else if op_code == "04" {
            let value = get_value(&h, current_op_index+1, &param_first, relative_base);
            let _ = output.send(value);
            current_op_index += 2;
        } else if op_code == "05" {
            let first_value = get_value(&h, current_op_index+1, &param_first, relative_base);
            let second_value = get_value(&h, current_op_index+2, &param_second, relative_base);

            if first_value != 0 {
                current_op_index = second_value;
            } else {
                current_op_index += 3;
            }
        } else if op_code == "06" {
            let first_value = get_value(&h, current_op_index+1, &param_first, relative_base);
            let second_value = get_value(&h, current_op_index+2, &param_second, relative_base);

            if first_value == 0 {
                current_op_index = second_value;
            } else {
                current_op_index += 3;
            }
        } else if op_code == "07" {
            let first_value = get_value(&h, current_op_index+1, &param_first, relative_base);
            let second_value = get_value(&h, current_op_index+2, &param_second, relative_base);

            if first_value < second_value {
                store_value(h, current_op_index+3, 1, &param_third, relative_base);
            } else {
                store_value(h, current_op_index+3, 0, &param_third, relative_base);
            }

            current_op_index += 4;
        } else if op_code == "08" {
            let first_value = get_value(&h, current_op_index+1, &param_first, relative_base);
            let second_value = get_value(&h, current_op_index+2, &param_second, relative_base);

            if first_value == second_value {
                store_value(h, current_op_index+3, 1, &param_third, relative_base);
            } else {
                store_value(h, current_op_index+3, 0, &param_third, relative_base);
            }

            current_op_index += 4;
        } else if op_code == "09" {
            let value = get_value(&h, current_op_index+1, &param_first, relative_base);
            relative_base += value;
            current_op_index += 2;
        } else if op_code == "99" {
            return;
        }
    }
    panic!("Didn't get halt opcode 99");
}

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s = String::new();
    let _ = input.read_to_string(&mut s);

    let mut h: HashMap<i64, i64> = HashMap::new();
    for (i, s) in s.trim_end().split(',').collect::<Vec<&str>>().iter().enumerate() {
        h.insert(i as i64, s.parse().unwrap());
    }

    let (robot_command_tx, robot_command_rx) = mpsc::channel();
    let (robot_status_tx, robot_status_rx) = mpsc::channel();

    thread::spawn(move || {
        run_robot(&mut h, robot_command_rx, robot_status_tx);
    });

    let mut grid = Grid::new();

    // explore mode
    loop {
        // search for an unknown position

        // get shortest path to unknown position
        let path = bfs(&grid.robot_position,
                       |p| grid.successors(p).iter().map(|p2| p2.0).collect::<Vec<Position>>(),
                       |p| *grid.known_positions.get(&p).or(Some(&StatusCode::Unknown)).unwrap() == StatusCode::Unknown);

        if let Some(path) = path {
            // send movement commands to reach unknown
            for path_pos in path.iter().skip(1) {
                grid.move_robot_to_target(path_pos, &robot_command_tx, &robot_status_rx, false);
            }
        } else {
            // all positions found
            break;
        }
    }

    // start simulating oxygen flow
    let mut oxygen_tick_counter = 0;
    loop {
        if grid.simulate_oxygen(true) {
            println!("{}", oxygen_tick_counter);
            break;
        } else {
            oxygen_tick_counter += 1;
        }
    }
}
use std::fs::File;
use std::io::{Read, Write};
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc;
use std::cmp::{min,max};
use std::fmt;

extern crate pathfinding;
use pathfinding::utils::absdiff;
use pathfinding::prelude::{astar, bfs};

extern crate crossterm;

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

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Hash, Clone)]
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
    start: Position,
    robot_position: Position,
}

impl Grid {
    fn new() -> Self {
        let mut g = Grid {
            known_positions: HashMap::new(),
            start: Position(0,0),
            robot_position: Position(0,0)
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

    fn bfs_successors(&self, p: &Position) -> Vec<Position> {
        let mut rv = Vec::new();
        // check 4 directions
        // each direction can be a successor as long as no wall present

        { // scopes to prevent accidentally using same position twice when copy/pasting
            let up_pos = Position( p.0, p.1 + 1 );
            if *self.known_positions.get(&up_pos).or(Some(&StatusCode::Unknown)).unwrap() != StatusCode::Wall {
                rv.push(up_pos);
            }
        }

        {
            let down_pos = Position( p.0, p.1 - 1 );
            if *self.known_positions.get(&down_pos).or(Some(&StatusCode::Unknown)).unwrap() != StatusCode::Wall {
                rv.push(down_pos);
            }
        }

        {
            let left_pos = Position( p.0 - 1, p.1 );
            if *self.known_positions.get(&left_pos).or(Some(&StatusCode::Unknown)).unwrap() != StatusCode::Wall {
                rv.push(left_pos);
            }
        }

        {
            let right_pos = Position( p.0 + 1, p.1 );
            if *self.known_positions.get(&right_pos).or(Some(&StatusCode::Unknown)).unwrap() != StatusCode::Wall {
                rv.push(right_pos);
            }
        }

        rv
    }

    fn move_robot_to_target(&mut self, target: &Position, robot_tx: &mpsc::Sender<i64>, robot_rx: &mpsc::Receiver<i64>, draw: bool) -> StatusCode {
//        println!("at {:?}, moving to {:?}", self.robot_position, target);
        let mut s = self.known_positions.get(target).or(Some(&StatusCode::Unknown)).unwrap().clone();

        'find_target: loop {
//            println!("starting loop");
            let path = astar(&self.robot_position, |p| self.successors(p), |p| p.distance(&target), |p| p == target).unwrap();
//            println!("{:?}", path);

            // if there are any unknowns, can only move to first one before recalculating path
            // because revealing can show a path is no longer valid
            for path_pos in path.0.iter().skip(1) {
//                println!("at {:?}, want {:?}", self.robot_position, path_pos);
                let movement = self.robot_position.move_to_other(path_pos);
//                println!("moving {:?}", movement);
                robot_tx.send(movement.into()).unwrap();
                s = robot_rx.recv().unwrap().into();

                if s != StatusCode::Wall {
                    self.robot_position = path_pos.clone();
                }

                if draw {
                    self.draw();
//                    std::io::stdin().read(&mut [0u8]).unwrap();
                }

                if self.known_positions.get(&path_pos).is_none() {
                    self.known_positions.insert(path_pos.clone(), s.clone());

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

    fn draw(&self) {
        let mut min_x = 0;
        let mut min_y = 0;
        let mut max_x = 0;
        let mut max_y = 0;

        for (pos, _) in self.known_positions.iter() {
            min_x = min(min_x, pos.0);
            min_y = min(min_y, pos.1);
            max_x = max(max_x, pos.0);
            max_y = max(max_y, pos.1);
        }

        crossterm::execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
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
        std::thread::sleep(std::time::Duration::from_millis(250));

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
    Goal
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusCode::Wall => write!(f, "#"),
            StatusCode::Goal => write!(f, "O"),
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
                StatusCode::Goal
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
    let mut found_goal = false;
    loop {
        // search for an unknown position

        // get shortest path to unknown position
        let path = bfs(&grid.robot_position,
                         |p| grid.bfs_successors(p),
                         |p| *grid.known_positions.get(&p).or(Some(&StatusCode::Unknown)).unwrap() == StatusCode::Unknown).unwrap();

        // send movement commands to reach unknown
        for path_pos in path.iter().skip(1) {
            let s = grid.move_robot_to_target(path_pos, &robot_command_tx, &robot_status_rx, false);
//            grid.draw();
//            std::io::stdin().read(&mut [0u8]).unwrap();
            if s == StatusCode::Goal {
                found_goal = true;
            }
        }
        if found_goal {
            break;
        }
    }

    let oxygen_position = grid.robot_position.clone();

    // robot_position is now position of oxygen system
    // find shortest path from there back to start at 0, 0
    // move robot from oxygen system back to start to ensure entire path is revealed
    'find_best_to_start: loop {
        // always need to move to only the first position that is unknown or the goal
        // this is because moving into an unknown space might reveal the potential path is not viable
        let path = astar(&oxygen_position, |p| grid.successors(p), |p| p.distance(&Position(0, 0)), |p| *p == Position(0, 0)).unwrap();
        let mut first_unknown = None;
        for path_pos in path.0.iter().skip(1) {
            if *path_pos == grid.start {
//                grid.move_robot_to_target(&oxygen_position, &robot_command_tx, &robot_status_rx, true);
//                grid.move_robot_to_target(&grid.start.clone(), &robot_command_tx, &robot_status_rx, true);
                println!("{}", path.1);
                break 'find_best_to_start;
            }

            let s = grid.known_positions.get(&path_pos).or(Some(&StatusCode::Unknown)).unwrap();
            if *s == StatusCode::Unknown {
               first_unknown = Some(path_pos.clone());
                break;
            }
        }

        let first_unknown = first_unknown.unwrap();
        grid.move_robot_to_target(&first_unknown, &robot_command_tx, &robot_status_rx, false);

    }
}
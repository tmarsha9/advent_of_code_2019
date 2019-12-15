use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc;

#[derive(Debug)]
enum ParameterMode {
    Position,
    Immediate,
    Relative
}

#[derive(PartialEq, Debug)]
enum Turn {
    Clockwise,
    CounterClockwise
}

impl From<i64> for Turn {
    fn from(n: i64) -> Self {
        match n {
            0i64 => {
                Turn::CounterClockwise
            },
            1i64 => {
               Turn::Clockwise
            },
            _ => unreachable!()

        }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn turn(&self, turn: &Turn) -> Direction {
        match *self {
            Direction::Up => {
                match *turn {
                    Turn::Clockwise => Direction::Right,
                    Turn::CounterClockwise => Direction::Left,
                }
            },
            Direction::Down => {
                match *turn {
                    Turn::Clockwise => Direction::Left,
                    Turn::CounterClockwise => Direction::Right,
                }
            },
            Direction::Left => {
                match *turn {
                    Turn::Clockwise => Direction::Up,
                    Turn::CounterClockwise => Direction::Down,
                }
            },
            Direction::Right => {
                match *turn {
                    Turn::Clockwise => Direction::Down,
                    Turn::CounterClockwise => Direction::Up,
                }
            },
        }
    }
}

#[derive(PartialEq, Hash, Clone)]
struct Position{
    x: i64,
    y: i64
}
impl Eq for Position {}

impl Position {
    fn move_in_direction(&self, d: &Direction) -> Self {
        match *d {
            Direction::Up => {
                Position{x:self.x,y: self.y+1}
            },
            Direction::Down => {
                Position{x:self.x, y:self.y-1}
            },
            Direction::Left => {
                Position{x:self.x-1, y:self.y}
            },
            Direction::Right => {
                Position{x:self.x+1, y:self.y}
            }
        }
    }
}

#[derive(PartialEq, Debug)]
enum CameraCommand {
    Read,
    Write
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

fn run_robot(h: &mut HashMap<i64, i64>, input: mpsc::Receiver<i64>, output: mpsc::Sender<i64>, camera_command_output: mpsc::Sender<CameraCommand>) {
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
            let _ = camera_command_output.send(CameraCommand::Read);

            let value = input.recv().unwrap();
            store_value(h, current_op_index+1, value, &param_first, relative_base);
            current_op_index += 2;
        } else if op_code == "04" {
            let _ = camera_command_output.send(CameraCommand::Write);

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

    let (command_tx, command_rx) = mpsc::channel();
    let (camera_to_robot_tx, camera_to_robot_rx) = mpsc::channel();
    let (robot_to_camera_tx, robot_to_camera_rx) = mpsc::channel();

    thread::spawn(move || {
        run_robot(&mut h, camera_to_robot_rx, robot_to_camera_tx, command_tx);
    });

    let mut grid: HashMap<Position, i64> = HashMap::new();
    let mut robot_position = Position{x:0, y:0};
    let mut robot_direction = Direction::Up;
    let mut panels_colored = 0;
    loop {
        match command_rx.recv() {
            Ok(command) => {
                match command {
                    CameraCommand::Write => {
                        let value = robot_to_camera_rx.recv().unwrap();
                        if let None = grid.get(&robot_position) {
                            panels_colored += 1;
                        }
                        grid.insert(robot_position.clone(), value);
                        assert_eq!(CameraCommand::Write, command_rx.recv().unwrap());
                        let direction_to_turn: Turn = robot_to_camera_rx.recv().unwrap().into();
                        robot_direction = robot_direction.turn(&direction_to_turn);
                        robot_position = robot_position.move_in_direction(&robot_direction);
                    },
                    CameraCommand::Read => {
                        let color = *grid.get(&robot_position).or(Some(&0i64)).unwrap();
                        let _ = camera_to_robot_tx.send(color);
                    }
                }
            },
            Err(_) => {
                println!("{}", panels_colored);
                return;
            }
        }
    }
}
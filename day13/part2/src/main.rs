use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc;
use std::cmp::{max,min};
use std::fmt;
use std::str;

#[derive(PartialEq, Debug)]
enum CommandType {
    ScreenOutput,
    JoyStickRequest
}

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

#[derive(PartialEq, Hash, Clone, Debug)]
struct Position{
    x: i64,
    y: i64
}
impl Eq for Position {}

#[derive(Debug)]
enum TileType {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball,
}

impl From<i64> for TileType {
    fn from(i: i64) -> Self {
        match i {
            0 => {
                TileType::Empty
            },
            1 => {
                TileType::Wall
            },
            2 => {
                TileType::Block
            },
            3 => {
                TileType::HorizontalPaddle
            },
            4 => {
                TileType::Ball
            },
            _ => unreachable!()
        }
    }
}

// trying rust UTF-8 strings
// use alternative write calls for ascii screen characters
impl fmt::Display for TileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TileType::Empty => {
                write!(f, " ")
            },
            TileType::Wall => {
//                write!(f, "{}", str::from_utf8(&[0xe2, 0x8f, 0xb9]).unwrap())
                write!(f, "#")
            },
            TileType::Block => {
//                write!(f, "{}", str::from_utf8(&[0xe2, 0x96, 0xa2]).unwrap())
                write!(f, "O")
            },
            TileType::HorizontalPaddle => {
//                write!(f, "{}", str::from_utf8(&[0xe2, 0x96, 0x82]).unwrap())
                write!(f, "_")
            },
            TileType::Ball => {
//                write!(f, "{}", str::from_utf8(&[0xe2, 0x9a, 0xbd]).unwrap())
                write!(f, "o")
            },
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

fn run_game(h: &mut HashMap<i64, i64>, output_channel: mpsc::Sender<i64>, input_channel: mpsc::Receiver<i64>, command_channel: mpsc::Sender<CommandType>) {
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
            command_channel.send(CommandType::JoyStickRequest).unwrap();
            let joystick_value = input_channel.recv().unwrap();

            store_value(h, current_op_index+1, joystick_value, &param_first, relative_base);
            current_op_index += 2;
        } else if op_code == "04" {
            command_channel.send(CommandType::ScreenOutput).unwrap();

            let value = get_value(&h, current_op_index+1, &param_first, relative_base);
            let _ = output_channel.send(value);
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

fn display_image(i: &HashMap<Position, TileType>, max_x: i64, max_y: i64) {
    for y in 0..max_y {
        for x in 0..max_x {
            let t = i.get(&Position{x,y}).or(Some(&TileType::Empty)).unwrap();
            print!("{}", t);
        }
        println!();
    }
}

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s = String::new();
    let _ = input.read_to_string(&mut s);

    let mut h: HashMap<i64, i64> = HashMap::new();
    for (i, s) in s.trim_end().split(',').collect::<Vec<&str>>().iter().enumerate() {
        h.insert(i as i64, s.parse().unwrap());
    }
    h.insert(0i64, 2); // initial value to start part 2

    let (screen_tx, screen_rx) = mpsc::channel();
    let (joystick_tx, joystick_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();

    thread::spawn(move || {
        run_game(&mut h, screen_tx, joystick_rx, command_tx);
    });

    let mut game_state: HashMap<Position, TileType> = HashMap::new();
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut last_score = 0;

    loop {
        match command_rx.recv() {
            Ok(command) => match command {
                CommandType::ScreenOutput => {
                    let x_value = screen_rx.recv().unwrap();
                    assert_eq!(command_rx.recv().unwrap(), CommandType::ScreenOutput);
                    let y_value = screen_rx.recv().unwrap();
                    assert_eq!(command_rx.recv().unwrap(), CommandType::ScreenOutput);

                    if x_value == -1 && y_value == 0 {
                        let score = screen_rx.recv().unwrap();
                        last_score = score;
//                        println!("Score: {}", score);
                    } else {
                        let tile_type: TileType = screen_rx.recv().unwrap().into();
                        game_state.insert(Position { x: x_value, y: y_value }, tile_type);

                        min_x = min(min_x, x_value);
                        min_y = min(min_y, y_value);
                        max_x = max(max_x, x_value);
                        max_y = max(max_y, y_value);
                    }
                },
                CommandType::JoyStickRequest => {
                    // draw screen and send joystick input
//                    display_image(&game_state, max_x, max_y); // uncomment to see game in console
//                    std::thread::sleep(std::time::Duration::from_millis(250)); // uncomment to have game progress at constant pace automatically
//                    std::io::stdin().read(&mut [0u8]).unwrap(); // uncomment to pause for user // input to see screen
                    let mut paddle_pos = None;
                    let mut ball_pos = None;
                    for (pos, t) in &game_state {
                        if let TileType::Ball = t {
                            ball_pos = Some(pos.clone());
                        } else if let TileType::HorizontalPaddle = t {
                            paddle_pos = Some(pos.clone());
                        }

                        if ball_pos.is_some() && paddle_pos.is_some() {
                            break;
                        }
                    }

                    let paddle_pos = paddle_pos.unwrap();
                    let ball_pos = ball_pos.unwrap();

                    if paddle_pos.x < ball_pos.x {
                        // move right
                        let _ = joystick_tx.send(1);
                    } else if paddle_pos.x > ball_pos.x {
                        // move left
                        let _ = joystick_tx.send(-1);
                    } else {
                        // don't move
                        let _ = joystick_tx.send(0);
                    }
                }
            },
            Err(_) => {
                break;
            }
        }
    }

    println!("{}", last_score);
}
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc;
use std::cmp::{max,min};

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

#[derive(PartialEq, Hash, Clone)]
struct Position{
    x: i64,
    y: i64
}
impl Eq for Position {}

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

fn run_game(h: &mut HashMap<i64, i64>, output: mpsc::Sender<i64>) {
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
            unreachable!();
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

    let (game_tx, game_rx) = mpsc::channel();

    thread::spawn(move || {
        run_game(&mut h, game_tx);
    });

    let mut image: HashMap<Position, i64> = HashMap::new();
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut block_tile_counter = 0;

    loop {
        match game_rx.recv() {
            Ok(x_value) => {
                let y_value = game_rx.recv().unwrap();
                let tile_type: TileType = game_rx.recv().unwrap().into();

                if let TileType::Block = tile_type {
                    block_tile_counter += 1;
                }

                min_x = min(min_x, x_value);
                min_y = min(min_y, y_value);
                max_x = max(max_x, x_value);
                max_y = max(max_y, y_value);
            },
            Err(_) => {
                println!("{}", block_tile_counter);
                break;
            }
        }
    }
}
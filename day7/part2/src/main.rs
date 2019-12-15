use std::fs::File;
use std::io::Read;
use std::cmp::max;
use std::thread;
use std::sync::mpsc;

#[derive(Debug)]
enum ParameterMode {
    Position,
    Immediate
}

impl From<char> for ParameterMode {
    fn from(num: char) -> ParameterMode {
        match num {
            '0' => ParameterMode::Position,
            '1' => ParameterMode::Immediate,
            _ => unreachable!()
        }
    }
}

fn get_value(intcode: &Vec<i32>, i: usize, p: &ParameterMode) -> i32 {
    match p {
        ParameterMode::Position => {
            *intcode.get(*intcode.get(i).unwrap() as usize).unwrap()
        },
        ParameterMode::Immediate => {
            *intcode.get(i).unwrap()
        }
    }
}

fn run_amplifier(mut modifiable_intcode: Vec<i32>, phase_input: i32, input_channel: mpsc::Receiver<i32>, output_channel: mpsc::Sender<i32>, amplifier_number: i32) -> Option<i32> {
    let mut current_op_index: usize = 0;
    let mut first_input = true;
    let mut last_output = None;

    while current_op_index < modifiable_intcode.len() {
        let op  = *modifiable_intcode.get(current_op_index).unwrap();
        let mut op_digits: Vec<char> = op.to_string().chars().collect();

        while op_digits.len() < 5 {
            op_digits.insert(0,'0');
        }


        let mut op_code = String::new();
        op_code.push(op_digits[3]);
        op_code.push(op_digits[4]);

        let param_first = ParameterMode::from(op_digits[2]);
        let param_second = ParameterMode::from(op_digits[1]);

        if op_code == "01" {
            let first_value = get_value(&modifiable_intcode, current_op_index+1, &param_first);
            let second_value = get_value(&modifiable_intcode, current_op_index+2, &param_second);
            let third_value = *modifiable_intcode.get(current_op_index+3).unwrap() as usize;

            modifiable_intcode[third_value] = first_value+second_value;
            current_op_index += 4;
        } else if op_code == "02" {
            let first_value = get_value(&modifiable_intcode, current_op_index+1, &param_first);
            let second_value = get_value(&modifiable_intcode, current_op_index+2, &param_second);
            let third_value = *modifiable_intcode.get(current_op_index+3).unwrap() as usize;

            modifiable_intcode[third_value] = first_value*second_value;
            current_op_index += 4;
        } else if op_code == "03" {
            let position = *modifiable_intcode.get(current_op_index+1).unwrap() as usize;

            if first_input {
                modifiable_intcode[position] = phase_input;
                first_input = false;
            } else {
                let input = input_channel.recv().unwrap();
                modifiable_intcode[position] = input;
            }
            current_op_index += 2;
        } else if op_code == "04" {
            let output = get_value(&modifiable_intcode, current_op_index+1, &param_first);

            if amplifier_number == 4 {
                last_output = Some(output);
            }

            if let Err(_error) = output_channel.send(output) {
                // as soon as one thread halts, they should all halt
                if amplifier_number == 4 {
                    return last_output;
                } else {
                    return None;
                }
            }
            current_op_index += 2;
        } else if op_code == "05" {
            let first_value = get_value(&modifiable_intcode, current_op_index+1, &param_first);
            let second_value = get_value(&modifiable_intcode, current_op_index+2, &param_second) as usize;

            if first_value != 0 {
                current_op_index = second_value;
            } else {
                current_op_index += 3;
            }
        } else if op_code == "06" {
            let first_value = get_value(&modifiable_intcode, current_op_index+1, &param_first);
            let second_value = get_value(&modifiable_intcode, current_op_index+2, &param_second) as usize;

            if first_value == 0 {
                current_op_index = second_value;
            } else {
                current_op_index += 3;
            }
        } else if op_code == "07" {
            let first_value = get_value(&modifiable_intcode, current_op_index+1, &param_first);
            let second_value = get_value(&modifiable_intcode, current_op_index+2, &param_second);
            let third_value = *modifiable_intcode.get(current_op_index+3).unwrap() as usize;

            if first_value < second_value {
                modifiable_intcode[third_value] = 1;
            } else {
                modifiable_intcode[third_value] = 0;
            }

            current_op_index += 4;
        } else if op_code == "08" {
            let first_value = get_value(&modifiable_intcode, current_op_index+1, &param_first);
            let second_value = get_value(&modifiable_intcode, current_op_index+2, &param_second);
            let third_value = *modifiable_intcode.get(current_op_index+3).unwrap() as usize;

            if first_value == second_value {
                modifiable_intcode[third_value] = 1;
            } else {
                modifiable_intcode[third_value] = 0;
            }

            current_op_index += 4;
        } else if op_code == "99" {
            if amplifier_number == 4 {
                if let Some(value) = last_output {
                    return Some(value);
                } else {
                    unreachable!();
                }
            } else {
                return None;
            }
        }
    }
    unreachable!();
}

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s = String::new();
    let _ = input.read_to_string(&mut s);
    let intcode: Vec<i32> = s.trim_end().split(',').collect::<Vec<&str>>().iter().map(|s| s.parse().unwrap()).collect();

    let mut max_thrust = 0;

    for p1 in 5..=9 {
        for p2 in 5..=9 {
            if p2 == p1 { continue; }
            for p3 in 5..=9 {
                if p3 == p2 || p3 == p1 { continue; }
                for p4 in 5..=9 {
                    if p4 == p3 || p4 == p2 || p4 == p1 { continue; }
                    for p5 in 5..=9 {
                        if p5 == p4 || p5 == p3 || p5 == p2 || p5 == p1 { continue; }

                        // start the 5 amplifiers
                        // c1 = channel 1
                        // tx, rx for transmit and receive

                        let (c1tx, c1rx) = mpsc::channel();
                        let (c2tx, c2rx) = mpsc::channel();
                        let (c3tx, c3rx) = mpsc::channel();
                        let (c4tx, c4rx) = mpsc::channel();
                        let (c5tx, c5rx) = mpsc::channel();

                        let intcode1 = intcode.clone();
                        let intcode2 = intcode.clone();
                        let intcode3 = intcode.clone();
                        let intcode4 = intcode.clone();
                        let intcode5 = intcode.clone();

                        thread::spawn(move || {
                            run_amplifier(intcode1, p1, c1rx, c2tx, 0);
                        });

                        thread::spawn(move || {
                            run_amplifier(intcode2, p2, c2rx, c3tx, 1);
                        });

                        thread::spawn(move || {
                            run_amplifier(intcode3, p3, c3rx, c4tx, 2);
                        });
                        thread::spawn(move || {
                            run_amplifier(intcode4, p4, c4rx, c5tx, 3);
                        });

                        // send initial input to first amplifier
                        c1tx.send(0).unwrap();

                        // main thread will run the 5th amplifier
                        let thrust = run_amplifier(intcode5, p5, c5rx, c1tx, 4).unwrap();
                        max_thrust = max(max_thrust, thrust);
                    }
                }
            }
        }
    }

    println!("{}", max_thrust);
}
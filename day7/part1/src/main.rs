use std::fs::File;
use std::io::Read;
use std::cmp::max;

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

fn get_value(v: &Vec<i32>, i: usize, p: &ParameterMode) -> i32 {
    match p {
        ParameterMode::Position => {
            *v.get(*v.get(i).unwrap() as usize).unwrap()
        },
        ParameterMode::Immediate => {
            *v.get(i).unwrap()
        }
    }
}

fn get_thrust(original: &Vec<i32>, phase_inputs: [i32; 5]) -> i32 {
    let mut current_op_index: usize = 0;

    let mut input = 0;
    let mut amplifier_counter = 0;
    let mut first_input = true;

    let mut v = original.clone();
    while current_op_index < v.len() {
        let op  = *v.get(current_op_index).unwrap();
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
            let first_value = get_value(&v, current_op_index+1, &param_first);
            let second_value = get_value(&v, current_op_index+2, &param_second);
            let third_value = *v.get(current_op_index+3).unwrap() as usize;

            v[third_value] = first_value+second_value;
            current_op_index += 4;
        } else if op_code == "02" {
            let first_value = get_value(&v, current_op_index+1, &param_first);
            let second_value = get_value(&v, current_op_index+2, &param_second);
            let third_value = *v.get(current_op_index+3).unwrap() as usize;

            v[third_value] = first_value*second_value;
            current_op_index += 4;
        } else if op_code == "03" {
            let position = *v.get(current_op_index+1).unwrap() as usize;

            if first_input {
                v[position] = phase_inputs[amplifier_counter];
            } else {
                v[position] = input;
            }
            first_input = !first_input;
            current_op_index += 2;
        } else if op_code == "04" {
            let output = get_value(&v, current_op_index+1, &param_first);
            if amplifier_counter < 4 {
                input = output;
                current_op_index = 0;
                v = original.clone();
                amplifier_counter += 1;
            } else {
                return output;
            }
        } else if op_code == "05" {
            let first_value = get_value(&v, current_op_index+1, &param_first);
            let second_value = get_value(&v, current_op_index+2, &param_second) as usize;

            if first_value != 0 {
                current_op_index = second_value;
            } else {
                current_op_index += 3;
            }
        } else if op_code == "06" {
            let first_value = get_value(&v, current_op_index+1, &param_first);
            let second_value = get_value(&v, current_op_index+2, &param_second) as usize;

            if first_value == 0 {
                current_op_index = second_value;
            } else {
                current_op_index += 3;
            }
        } else if op_code == "07" {
            let first_value = get_value(&v, current_op_index+1, &param_first);
            let second_value = get_value(&v, current_op_index+2, &param_second);
            let third_value = *v.get(current_op_index+3).unwrap() as usize;

            if first_value < second_value {
                v[third_value] = 1;
            } else {
                v[third_value] = 0;
            }

            current_op_index += 4;
        } else if op_code == "08" {
            let first_value = get_value(&v, current_op_index+1, &param_first);
            let second_value = get_value(&v, current_op_index+2, &param_second);
            let third_value = *v.get(current_op_index+3).unwrap() as usize;

            if first_value == second_value {
                v[third_value] = 1;
            } else {
                v[third_value] = 0;
            }

            current_op_index += 4;
        } else if op_code == "99" {
            panic!();
        }
    }
    panic!();
}

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s = String::new();
    let _ = input.read_to_string(&mut s);
    let v: Vec<i32> = s.trim_end().split(',').collect::<Vec<&str>>().iter().map(|s| s.parse().unwrap()).collect();

    let mut max_thrust = 0;

    for p1 in 0..=4 {
        for p2 in 0..=4 {
            if p2 == p1 { continue; }
            for p3 in 0..=4 {
                if p3 == p2 || p3 == p1 { continue; }
                for p4 in 0..=4 {
                    if p4 == p3 || p4 == p2 || p4 == p1 { continue; }
                    for p5 in 0..=4 {
                        if p5 == p4 || p5 == p3 || p5 == p2 || p5 == p1 { continue; }
                        let thrust = get_thrust(&v.clone(), [p1, p2, p3, p4, p5]);
                        max_thrust = max(max_thrust, thrust);
                    }
                }
            }
        }
    }

    println!("{}", max_thrust);
}
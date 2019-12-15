use std::fs::File;
use std::io::Read;

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

fn run_intcode(v: &mut Vec<i32>) {
    let mut current_op_index: usize = 0;

    let first_input_dest = *v.get(current_op_index+1).unwrap() as usize;
    v[first_input_dest] = 5;
    println!("storing {} at {}", 5, first_input_dest);
    current_op_index += 2;

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
        } else if op_code == "04" {
            let value = get_value(&v, current_op_index+1, &param_first);
            println!("{}", value);
            current_op_index += 2;
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
            return;
        }
    }
    panic!("Didn't get halt opcode 99");
}

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s = String::new();
    let _ = input.read_to_string(&mut s);
    let mut v: Vec<i32> = s.trim_end().split(',').collect::<Vec<&str>>().iter().map(|s| s.parse().unwrap()).collect();

    run_intcode(&mut v);
}
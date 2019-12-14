use std::fs::File;
use std::io::Read;

fn get_output(v: &mut Vec<usize>) -> usize {
    let mut current_op_index: usize = 0;
    while current_op_index < v.len() {
        let op = *v.get(current_op_index).unwrap();

        if op == 1 {
            let first = *v.get(*v.get(current_op_index+1).unwrap()).unwrap();
            let second = *v.get(*v.get(current_op_index+2).unwrap()).unwrap();
            let third_index = *v.get(current_op_index+3).unwrap();
            v[third_index] = first+second;
        } else if op == 2 {
            let first = *v.get(*v.get(current_op_index+1).unwrap()).unwrap();
            let second = *v.get(*v.get(current_op_index+2).unwrap()).unwrap();
            let third_index = *v.get(current_op_index+3).unwrap();
            v[third_index] = first*second;
        }

        current_op_index += 4;
    }
    *v.get(0).unwrap()
}

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s = String::new();
    let _ = input.read_to_string(&mut s);
    let v: Vec<usize> = s.trim_end().split(',').collect::<Vec<&str>>().iter().map(|s| s.parse().unwrap()).collect();

    for noun in 0..=99 {
        for verb in 0..=99 {
            let mut v_clone = v.clone();
            v_clone[1] = noun;
            v_clone[2] = verb;
            let output = get_output(&mut v_clone);
            if output == 19690720 {
                // noun and verb found
                println!("{}", 100*noun+verb);
                return;
            }
        }
    }
    unreachable!();
}
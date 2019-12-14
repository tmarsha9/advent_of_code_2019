use std::fs::File;
use std::io::Read;

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s = String::new();
    let _ = input.read_to_string(&mut s);
    let mut v: Vec<usize> = s.trim_end().split(',').collect::<Vec<&str>>().iter().map(|s| s.parse().unwrap()).collect();

    v[1] = 12;
    v[2] = 2;

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
    println!("Output:  {}", *v.get(0).unwrap());
}

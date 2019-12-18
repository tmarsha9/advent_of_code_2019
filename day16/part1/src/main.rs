use std::fs::File;
use std::io::Read;

const PATTERN: [i32; 4] = [0, 1, 0, -1];

struct Pattern {
    output_element_index: usize,
    curr_index: usize,
}
impl Iterator for Pattern {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        let rv = PATTERN[(self.curr_index/(self.output_element_index+1))%PATTERN.len()];
        self.curr_index += 1;
        Some(rv)
    }
}

impl Pattern {
    fn new(output_index: usize) -> Self {
        let mut p = Pattern{
            output_element_index: output_index,
            curr_index: 0
        };
        p.next(); // always skip first
        p
    }
}

fn get_new_signal(input_signal: Vec<i32>) -> Vec<i32> {
    let mut output_signal = Vec::with_capacity(input_signal.len());

    for (i, _) in input_signal.iter().enumerate() {
        let mut acc = 0;
        for (j, p) in Pattern::new(i).take(input_signal.len()).enumerate() {
            acc += input_signal[j] * p;
        }
        output_signal.push(acc.abs()%10);
    }

    output_signal
}

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s = String::new();
    let _ = input.read_to_string(&mut s);


    let mut signal = Vec::new();
    for c in s.trim_end().chars() {
        signal.push(c.to_digit(10).unwrap() as i32);
    }

    for _ in 0..100 {
        signal = get_new_signal(signal);
    }

    println!("{}", signal.iter().take(8).map(|v| v.to_string()).collect::<Vec<_>>().join(""));

}

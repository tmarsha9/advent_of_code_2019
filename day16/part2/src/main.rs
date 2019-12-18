use std::fs::File;
use std::io::Read;

fn get_new_signal(input_signal: &mut Vec<i32>) {
    // create in-place

    // sum over input, then build output using one digit from at a time
    let mut sum: i32 = input_signal.iter().sum();
    for i in 0..input_signal.len() {
        let before = input_signal[i];
        input_signal[i] = sum%10;
        sum -= before;
    }
}

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s = String::new();
    let _ = input.read_to_string(&mut s);


    let mut signal_piece = Vec::new();
    for c in s.trim_end().chars() {
        signal_piece.push(c.to_digit(10).unwrap() as i32);
    }

    let mut offset = 0;
    for d in signal_piece.iter().take(7) {
        offset *= 10;
        offset += d;
    }
    let offset = offset;

    let mut signal = Vec::new();
    for _ in 0..10000 {
        signal.append(&mut signal_piece.clone());
    }

    // since offset > halfway through signal, last half of output signal is sum over last of input
    assert!(offset as usize > signal.len()/2);

    let mut signal = Vec::from(&signal[offset as usize..]);
    for _ in 0..100 {
        get_new_signal(&mut signal);
    }

    println!("{}", signal.iter().take(8).map(|v| v.to_string()).collect::<Vec<_>>().join(""));

}
use std::fs::File;
use std::io::Read;

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s= String::new();
    let _ = input.read_to_string(&mut s);
    let mut total = 0;
    for line in s.lines() {
        let mass: u32 = line.parse().unwrap();
        total += (mass/3)-2;
    }
    println!("{}", total);
}

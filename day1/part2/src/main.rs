use std::fs::File;
use std::io::Read;

fn calc_fuel(m: i32) -> i32 {
    if m/3 > 2 {
        m/3-2
    } else {
        0
    }
}

fn calc_all_fuel(m: i32) -> i32 {
    let fuel_for_mass = calc_fuel(m);

    let mut total = fuel_for_mass;
    let mut fuel = fuel_for_mass;

    while fuel > 0 {
        let new_fuel = calc_fuel(fuel);
        total += new_fuel;
        fuel = new_fuel;
    }

    total
}

fn main() {
    let mut input = File::open("../input.txt").expect("couldn't open input");
    let mut s= String::new();
    let _ = input.read_to_string(&mut s);

    let mut total_fuel = 0;

    for line in s.lines() {
        let mass: i32 = line.parse().unwrap();
        total_fuel += calc_all_fuel(mass);
    }

    println!("{}", total_fuel);
}

#[cfg(test)]
mod tests {
    use crate::{calc_all_fuel};

    #[test]
    fn test1() {
        assert_eq!(calc_all_fuel(14), 2);
    }

    #[test]
    fn test2() {
        assert_eq!(calc_all_fuel(1969), 966);
    }

    #[test]
    fn test3() {
        assert_eq!(calc_all_fuel(100756), 50346);
    }
}

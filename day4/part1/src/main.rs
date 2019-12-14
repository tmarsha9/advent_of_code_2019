use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::error::Error;

fn check_has_double_digits(s: &str) -> bool {
    let mut i = s.chars().skip(1);
    let mut prev = s.chars().next().unwrap();

    while let Some(current) = i.next() {
        if prev == current {
            return true;
        }
        prev = current;
    }

    return false;
}

fn check_no_decrease(s: &str) -> bool {
    let mut i = s.chars().skip(1);
    let mut prev = s.chars().next().unwrap().to_digit(10).unwrap();

    while let Some(current) = i.next() {
        let current = current.to_digit(10).unwrap();

        if current < prev {
            return false;
        }

        prev = current;
    }

    return true;
}

fn main() -> Result<(),Box<dyn Error>> {
    let input = File::open("../input.txt")?;
    let mut reader = BufReader::new(input);

    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    let first_line: Vec<&str> = first_line.trim_end().split('-').collect::<Vec<&str>>();

    let lower_bound: u32 = first_line[0].parse()?;
    let upper_bound: u32 = first_line[1].parse()?;

    let mut num_passwords = 0;
    for value in lower_bound..=upper_bound {
       let value_as_string = value.to_string();
        if check_has_double_digits(&value_as_string) && check_no_decrease(&value_as_string) {
            num_passwords += 1;
        }
    }

    println!("{}", num_passwords);

    Ok(())
}
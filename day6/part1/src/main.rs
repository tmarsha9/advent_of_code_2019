use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::error::Error;
use std::collections::HashMap;

fn get_distance(h: &HashMap<String, String>, s: &String) -> u32{
    let mut rv = 0;

    let mut search_string = s;

    while let Some(next_string) = h.get(search_string) {
        search_string = next_string;
        rv += 1;
    }

    rv
}

fn main() -> Result<(), Box<dyn Error>>{
    let f = File::open("../input.txt")?;
    let f = BufReader::new(f);

    let mut m = HashMap::new();

    for line in f.lines() {
        let line = line?.split(')').collect::<Vec<&str>>().iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let a: String = line[0].to_string();
        let b: String = line[1].to_string();

        // b is in orbit around a
        println!("{} is in orbit around {}", b, a);

        m.insert(b.clone(), a.clone());
    }

    let mut total = 0;
    for s in m.keys() {
        total += get_distance(&m, s);
    }

    println!("{}", total);

    Ok(())
}

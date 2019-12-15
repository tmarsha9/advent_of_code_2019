use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::error::Error;
use std::collections::HashMap;

fn get_distance_to_orbit(m: &HashMap<String, String>, start: &String, target: &String) -> u32{
    let mut rv = 0;

    let mut search_string = m.get(start).unwrap();

    while search_string != target {
        search_string = m.get(search_string).unwrap();
        rv += 1;
    }

    rv
}

fn generate_path_to_com<'a>(m: &'a HashMap<String, String>, start: &String) -> Vec<&'a String> {
    let mut rv = Vec::new();

    let mut search_string = start;

    while let Some(next_string) = m.get(search_string) {
        rv.push(next_string);
        search_string = next_string;
    }

    rv
}

fn find_first_common_orbit<'a>(p1: &Vec<&'a String>, p2: &Vec<&'a String>) -> &'a String {
    for orbit1 in p1.iter() {
        for orbit2 in p2.iter() {
            if orbit1 == orbit2 {
                return orbit1;
            }
        }
    }

    unreachable!()
}

fn main() -> Result<(), Box<dyn Error>>{
    let san_string = String::from("SAN");
    let my_string = String::from("YOU");

    let f = File::open("../input.txt")?;
    let f = BufReader::new(f);

    let mut m = HashMap::new();

    for line in f.lines() {
        let line = line?.split(')').collect::<Vec<&str>>().iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let a: String = line[0].to_string();
        let b: String = line[1].to_string();

        // b is in orbit around a
        m.insert(b.clone(), a.clone());
    }

    let my_path = generate_path_to_com(&m, &my_string);
    let san_path = generate_path_to_com(&m, &san_string);
    let first_common = find_first_common_orbit(&my_path, &san_path);

    let min_hops = get_distance_to_orbit(&m, &my_string, first_common) +
    get_distance_to_orbit(&m, &san_string, first_common);

    println!("{}", min_hops);

    Ok(())
}
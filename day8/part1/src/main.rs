use std::fs::File;
use std::io::Read;
use std::error::Error;

const WIDTH: usize = 25;
const HEIGHT: usize = 6;

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("../input.txt")?;

    let mut layers: Vec<[[u32; HEIGHT]; WIDTH]> = Vec::new();

    let mut s = String::new();
    f.read_to_string(&mut s)?;

    let mut min_zeroes = WIDTH*HEIGHT;
    let mut min_zeroes_ones = 0;
    let mut min_zeroes_twos = 0;
    let mut it = s.trim_end().chars().peekable();

    while it.peek().is_some() {
        let mut layer_zeroes = 0;
        let mut layer_ones = 0;
        let mut layer_twos = 0;
        let mut layer = [[0; HEIGHT]; WIDTH];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let value = it.next().unwrap().to_digit(10).unwrap();
                layer[x][y] = value;

                if value == 0 {
                    layer_zeroes += 1;
                } else if value == 1 {
                    layer_ones += 1;
                } else if value == 2 {
                    layer_twos += 1;
                }
            }
        }
        layers.push(layer);

        if layer_zeroes < min_zeroes {
            min_zeroes = layer_zeroes;
            min_zeroes_ones = layer_ones;
            min_zeroes_twos = layer_twos;
        }
    }

    println!("{}", min_zeroes_ones*min_zeroes_twos);

    Ok(())
}

use std::fs::File;
use std::io::Read;
use std::error::Error;

extern crate image;

const WIDTH: usize = 25;
const HEIGHT: usize = 6;

const TRANSPARENT_PIXEL: image::Bgra<u8> = image::Bgra([0, 0, 0, 0]);

fn apply_layer(i: &mut [image::Bgra<u8>; HEIGHT*WIDTH], l: &[image::Bgra<u8>; HEIGHT*WIDTH]) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if i[y*WIDTH+x] == TRANSPARENT_PIXEL && l[y*WIDTH+x] != TRANSPARENT_PIXEL {
                i[y*WIDTH+x] = l[y*WIDTH+x];
            }
        }
    }
}

fn pixel_from_input(n: u8) -> image::Bgra<u8> {
    match n {
        0 => image::Bgra([0,0,0,255]),
        1 => image::Bgra([255, 255, 255, 255]),
        2 => image::Bgra([0, 0, 0, 0]),
        _ => unreachable!()
    }
}

fn into_bytes(i: &[image::Bgra<u8>; WIDTH*HEIGHT]) -> [u8; WIDTH*HEIGHT*4] {
    let mut rv = [0; WIDTH*HEIGHT*4];

    let mut offset = 0;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let bgra = i[y*WIDTH+x].0;
            rv[y*WIDTH+x + offset + 0] = bgra[0];
            rv[y*WIDTH+x + offset + 1] = bgra[1];
            rv[y*WIDTH+x + offset + 2] = bgra[2];
            rv[y*WIDTH+x + offset + 3] = bgra[3];
            offset += 3;
        }
    }

    rv
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("../input.txt")?;

    let mut s = String::new();
    f.read_to_string(&mut s)?;

    let mut it = s.trim_end().chars().peekable();

    let mut image = [TRANSPARENT_PIXEL; WIDTH*HEIGHT];

    while it.peek().is_some() {
        let mut layer = [TRANSPARENT_PIXEL; HEIGHT*WIDTH];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let value = it.next().unwrap().to_digit(10).unwrap() as u8;
                layer[y*WIDTH+x] = pixel_from_input(value);
            }
        }
        apply_layer(&mut image, &layer);
    }

    let image = into_bytes(&image);
    image::save_buffer("output.png", &image, WIDTH as u32, HEIGHT as u32, image::BGRA(8)).unwrap();

    Ok(())
}

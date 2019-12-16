use std::fs::File;
use std::io::{BufRead,BufReader};
use std::error::Error;

#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
    z: i32,
}

impl Position {
    fn add_velocity(&mut self, v: &Velocity) {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
    }
}

#[derive(Debug)]
struct Velocity {
    x: i32,
    y: i32,
    z: i32,
}

impl Velocity {
    fn add_acceleration(&mut self, a: &Acceleration) {
        self.x += a.x;
        self.y += a.y;
        self.z += a.z;
    }
}

#[derive(Debug)]
struct Acceleration {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug)]
struct Moon {
    p: Position,
    v: Velocity,
}

struct System {
    moons: Vec<Moon>,
}

impl System {
    fn new() -> Self{
        System {
            moons: Vec::new()
        }
    }

    fn tick(&mut self) {
        // update velocities

        // need the accelerations vector because can't have a mutable iter and an immutable
        // iterator to moons vec
        let mut accelerations = Vec::new();

        for (i, moon) in self.moons.iter().enumerate() {
            let mut a = Acceleration {x: 0, y: 0, z: 0};

            for (j, moon2) in self.moons.iter().enumerate() {
                 if i == j {
                     // skip self
                     continue;
                 } else {
                     // compute acceleration that affects moon, NOT moon2
                     // moon2 will be handled by other loop and it will be labeled just moon then
                     let dx = moon2.p.x-moon.p.x;
                     let dy = moon2.p.y-moon.p.y;
                     let dz = moon2.p.z-moon.p.z;

                     // acceleration only affected by at most +/- 1
                     if dx < 0 {
                         a.x += -1;
                     } else if dx > 0 {
                         a.x += 1;
                     }
                     if dy < 0 {
                         a.y += -1;
                     } else if dy > 0 {
                         a.y += 1;
                     }
                     if dz < 0 {
                         a.z += -1;
                     } else if dz > 0 {
                         a.z += 1;
                     }
                 }
            }
            accelerations.push(a);
        }

        // modify velocities after computing acceleration for each
        for (a, moon) in accelerations.iter().zip(self.moons.iter_mut()) {
            moon.v.add_acceleration(&a);
        }

        // update positions
        for moon in self.moons.iter_mut() {
            moon.p.add_velocity(&moon.v);
        }
    }

    fn compute_energy(&self) -> i32 {
        let mut energy = 0;

        for moon in self.moons.iter() {
            let pot = moon.p.x.abs() + moon.p.y.abs() + moon.p.z.abs();
            let kin = moon.v.x.abs() + moon.v.y.abs() + moon.v.z.abs();
            energy += pot*kin;
        }

        energy
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let f = File::open("../input.txt")?;
    let f = BufReader::new(f);

    let mut s = System::new();

    for line in f.lines() {
        let line = line?.split(',').map(|mut s| {
                                                s = s.trim_start_matches("<");
                                                s = s.trim_start_matches("x=");
                                                s = s.trim_start_matches(" y=");
                                                s = s.trim_start_matches(" z=");
                                                s = s.trim_end_matches(">");
                                                s.parse().unwrap()
                                            }).collect::<Vec<i32>>();
        let x = line[0];
        let y = line[1];
        let z = line[2];

        let p = Position {x,y,z};
        let v = Velocity {x: 0, y: 0, z: 0};
        let m = Moon{p,v};

        s.moons.push(m);
    }

    for _ in 0..1000 {
        s.tick();
    }

    println!("{}", s.compute_energy());

    Ok(())
}

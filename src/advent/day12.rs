use crate::advent::AdventSolver;
use anyhow::{Error, format_err};
use lazy_static::lazy_static;
use num::integer::lcm;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufRead, BufReader};

#[derive(Default)]
pub struct Solver;

lazy_static! {
    static ref POSITION_REGEX: Regex =
        Regex::new(r"<x=(?P<x>-?\d+), y=(?P<y>-?\d+), z=(?P<z>-?\d+)>")
              .unwrap();
}

#[derive(Clone,Copy)]
enum Dim {
    X,
    Y,
    Z,
}

#[derive(Debug,Default)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug,Default)]
struct Moon {
    pos: Vec3,
    vel: Vec3,
}

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let mut moons = Self::read_moon_data("input/day12.txt")?;
        let mut xstates: HashSet<u64> = HashSet::new();
        let mut ystates: HashSet<u64> = HashSet::new();
        let mut zstates: HashSet<u64> = HashSet::new();
        for i in 0.. {
            let xstate = Moon::state_on(&moons, Dim::X);
            let ystate = Moon::state_on(&moons, Dim::Y);
            let zstate = Moon::state_on(&moons, Dim::Z);
            if xstates.contains(&xstate)
                && ystates.contains(&ystate)
                && zstates.contains(&zstate) {
                break;
            }
            xstates.insert(xstate);
            ystates.insert(ystate);
            zstates.insert(zstate);
            if i == 1000 {
                println!("Total energy after 1000 steps: {}",
                         moons.iter()
                              .map(Moon::energy)
                              .fold(0, |sum, e| sum + e));
            }
            Self::step_system(&mut moons);
        }
        println!("Total cycle length: {}",
                 lcm(lcm(xstates.len(), ystates.len()), zstates.len()));

        Ok(())
    }
}

impl Solver {
    fn read_moon_data(path: &str) -> Result<Vec<Moon>, Error> {
        BufReader::new(File::open(path)?)
                  .lines()
                  .map(|l| {
                      Vec3::from_string(&l?)
                           .and_then(|p| Ok(Moon::with_pos(p)))
                  })
                  .collect::<Result<Vec<Moon>, _>>()
    }

    fn apply_gravity(moon1: &mut Moon, moon2: &mut Moon, dim: Dim) {
        match moon1.pos_on(dim).cmp(&moon2.pos_on(dim)) {
            Ordering::Less => {
                moon1.set_vel_on(dim, moon1.vel_on(dim) + 1);
                moon2.set_vel_on(dim, moon2.vel_on(dim) - 1);
            },
            Ordering::Equal => {},
            Ordering::Greater => {
                moon1.set_vel_on(dim, moon1.vel_on(dim) - 1);
                moon2.set_vel_on(dim, moon2.vel_on(dim) + 1);
            }
        }
    }

    fn step_system(moons: &mut Vec<Moon>) {
        // Update velocities
        for m1 in 0..moons.len() {
            for m2 in m1+1..moons.len() {
                let (part1, part2) = moons.split_at_mut(m2);
                let moon1 = &mut part1[m1];
                let moon2 = &mut part2[0];
                Self::apply_gravity(moon1, moon2, Dim::X);
                Self::apply_gravity(moon1, moon2, Dim::Y);
                Self::apply_gravity(moon1, moon2, Dim::Z);
            }
        }
        // Update positions
        for moon in moons.iter_mut() {
            moon.step();
        }
    }
}

impl Vec3 {
    fn from_string(s: &str) -> Result<Vec3, Error> {
        match POSITION_REGEX.captures(s) {
            Some(caps) => {
                Ok(Vec3 {
                    x: caps["x"].parse::<i32>()?,
                    y: caps["y"].parse::<i32>()?,
                    z: caps["z"].parse::<i32>()?,
                })
            },
            None => {
                Err(format_err!("Couldn't parse position: {}", s))
            }
        }
    }
}

impl Moon {
    fn with_pos(p: Vec3) -> Moon {
        Moon {
            pos: p,
            ..Default::default()
        }
    }

    fn pos_on(&self, dim: Dim) -> i32 {
        match dim {
            Dim::X => self.pos.x,
            Dim::Y => self.pos.y,
            Dim::Z => self.pos.z,
        }
    }

    fn vel_on(&self, dim: Dim) -> i32 {
        match dim {
            Dim::X => self.vel.x,
            Dim::Y => self.vel.y,
            Dim::Z => self.vel.z,
        }
    }

    fn set_vel_on(&mut self, dim: Dim, vel: i32) {
        match dim {
            Dim::X => self.vel.x = vel,
            Dim::Y => self.vel.y = vel,
            Dim::Z => self.vel.z = vel,
        }
    }

    fn state_on(moons: &Vec<Moon>, dim: Dim) -> u64 {
        let mut hasher = DefaultHasher::new();
        for moon in moons.iter() {
            hasher.write_i32(moon.pos_on(dim));
            hasher.write_i32(moon.vel_on(dim));
        }
        hasher.finish()
    }

    fn energy(&self) -> i32 {
        (self.pos.x.abs() + self.pos.y.abs() + self.pos.z.abs())
        * (self.vel.x.abs() + self.vel.y.abs() + self.vel.z.abs())
    }

    fn step(&mut self) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
        self.pos.z += self.vel.z;
    }
}

use crate::advent::AdventSolver;
use anyhow::{format_err, Error};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Default)]
pub struct Solver;

type OrbitMap = HashMap<String, String>;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let orbits = Self::read_orbits_from_file("input/day06.txt")?;
        println!("Total orbits: {}", Self::count_orbits(&orbits));
        println!("Transfers to Santa: {}", Self::count_transfers(&orbits));
        Ok(())
    }
}

impl Solver {
    fn count_orbits(orbits: &OrbitMap) -> usize {
        let mut result = 0;
        for orbit in orbits.keys() {
            let mut orbit: &str = &orbit;
            let mut body: &str = &orbits[orbit];
            result += 1;
            while body != "COM" {
                orbit = body;
                body = &orbits[orbit];
                result += 1;
            }
        }
        result
    }

    fn count_transfers(orbits: &OrbitMap) -> usize {
        let mut my_path = Self::path_to(orbits, "YOU", "COM");
        let mut his_path = Self::path_to(orbits, "SAN", "COM");

        while !my_path.is_empty() && !his_path.is_empty()
            && my_path.last() == his_path.last() {
            my_path.pop();
            his_path.pop();
        }
        my_path.len() + his_path.len()
    }

    fn path_to(orbits: &OrbitMap, start: &str, dest: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut body: &str = &orbits[start];
        result.push(body.to_string());
        while body != dest {
            body = &orbits[body];
            result.push(body.to_string());
        }
        result
    }

    fn read_orbits_from_file(path: &str) -> Result<OrbitMap, Error> {
        let lines: Vec<String> =
            BufReader::new(File::open(path)?)
                      .lines()
                      .collect::<Result<Vec<String>, _>>()?;
        Self::read_orbits_from_lines(&lines)
    }

    fn read_orbits_from_lines<T>(lines: &Vec<T>) -> Result<OrbitMap, Error>
        where T: AsRef<str>
    {
        let mut orbits = HashMap::new();
        let orbit_re = Regex::new(r"^(?P<lhs>[^)]+)\)(?P<rhs>.*)$")?;
        for line in lines {
            match orbit_re.captures(line.as_ref()) {
                Some(caps) => {
                    orbits.insert(caps["rhs"].to_string(),
                                  caps["lhs"].to_string());
                },
                None => return Err(format_err!("Bad orbit: {}", line.as_ref())),
            }
        }
        Ok(orbits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part2_example() {
        let lines: Vec<&str> = vec![
            "COM)B",
            "B)C",
            "C)D",
            "D)E",
            "E)F",
            "B)G",
            "G)H",
            "D)I",
            "E)J",
            "J)K",
            "K)L",
            "K)YOU",
            "I)SAN",
        ];
        let orbits = Solver::read_orbits_from_lines(&lines).unwrap();
        assert_eq!(4, Solver::count_transfers(&orbits));
    }
}

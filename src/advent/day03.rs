use crate::advent::AdventSolver;
use crate::shared::grid::{Dir, Pos};
use anyhow::{format_err, Error};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Default)]
pub struct Solver;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let wire_paths: Vec<String> =
            BufReader::new(File::open("input/day03.txt")?)
                      .lines()
                      .collect::<Result<Vec<String>, _>>()?;
        let path1 = Solver::parse_path(&wire_paths[0])?;
        let path2 = Solver::parse_path(&wire_paths[1])?;

        // Part 1
        match Solver::closest_intersection(&path1, &path2) {
            Some(pos) => {
                println!("Closest intersection: {:?} (dist: {})",
                         pos, pos.manhattan_distance(&Pos::origin()));
            },
            None => {
                println!("No intersections found.");
            }
        }

        // Part 2
        match Solver::minimum_steps_to_intersection(&path1, &path2) {
            Some(distance) => {
                println!("Minimum steps to an intersection: {}", distance);
            },
            None => {
                println!("No intersections found.");
            }
        }
        Ok(())
    }
}

impl Solver {
    fn all_intersections<'a>(path1: &Vec<Pos>,
                             path2: &Vec<Pos>) -> Vec<Pos> {
        let visited1: HashSet<Pos> = path1.iter().cloned().collect();
        let visited2: HashSet<Pos> = path2.iter().cloned().collect();
        visited1.intersection(&visited2).cloned().collect()
    }

    fn closest_intersection(path1: &Vec<Pos>, path2: &Vec<Pos>) -> Option<Pos> {
        let common = Self::all_intersections(path1, path2);
        let mut closest: Option<Pos> = None;
        for pos in common {
            if pos == Pos::origin() {
                continue;
            }
            match closest {
                None => closest = Some(pos),
                Some(c) => {
                    if pos.manhattan_distance(&Pos::origin()) <
                         c.manhattan_distance(&Pos::origin()) {
                        closest = Some(pos);
                    }
                }
            }
        }
        closest
    }

    fn minimum_steps_to_intersection(path1: &Vec<Pos>,
                                     path2: &Vec<Pos>) -> Option<usize> {
        let common = Self::all_intersections(path1, path2);
        let mut closest: Option<usize> = None;
        for pos in common {
            if pos == Pos::origin() {
                continue;
            }
            let dist = Self::steps_to_point(path1, pos).unwrap() +
                       Self::steps_to_point(path2, pos).unwrap();
            match closest {
                None => closest = Some(dist),
                Some(closest_dist) => {
                    if dist < closest_dist {
                        closest = Some(dist);
                    }
                },
            }
        }
        closest
    }

    fn parse_path(text: &str) -> Result<Vec<Pos>, Error> {
        let mut path = Vec::new();
        let mut cur_pos = Pos::origin();

        path.push(cur_pos);
        let moves: Vec<&str> = text.split(",").collect();
        for m in moves {
            let dir = match m.chars().next().unwrap() {
                'U' => Dir::Up,
                'D' => Dir::Down,
                'L' => Dir::Left,
                'R' => Dir::Right,
                _ => return Err(format_err!("Invalid direction: {}", m)),
            };
            let amount = m[1..].parse::<isize>()?;
            for _ in 0..amount {
                cur_pos = cur_pos.neighbor(dir);
                path.push(cur_pos);
            }
        }

        Ok(path)
    }

    fn steps_to_point(path: &Vec<Pos>, point: Pos) -> Option<usize> {
        let mut steps = 0;
        for &p in path {
            if p == point {
                return Some(steps);
            }
            steps += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1_ex1() {
        let path1 = Solver::parse_path("R8,U5,L5,D3").unwrap();
        let path2 = Solver::parse_path("U7,R6,D4,L4").unwrap();
        let pos = Solver::closest_intersection(&path1, &path2).unwrap();
        assert_eq!(6, pos.manhattan_distance(&Pos::origin()));
    }

    #[test]
    fn test_part1_ex2() {
        let path1 = Solver::parse_path("R75,D30,R83,U83,L12,D49,R71,U7,L72")
                           .unwrap();
        let path2 = Solver::parse_path("U62,R66,U55,R34,D71,R55,D58,R83")
                           .unwrap();
        let pos = Solver::closest_intersection(&path1, &path2).unwrap();
        assert_eq!(159, pos.manhattan_distance(&Pos::origin()));
    }

    #[test]
    fn test_part1_ex3() {
        let path1 =
            Solver::parse_path("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51")
                   .unwrap();
        let path2 =
            Solver::parse_path("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7")
                   .unwrap();
        let pos = Solver::closest_intersection(&path1, &path2).unwrap();
        assert_eq!(135, pos.manhattan_distance(&Pos::origin()));
    }

    #[test]
    fn test_part2_ex1() {
        let path1 = Solver::parse_path("R8,U5,L5,D3").unwrap();
        let path2 = Solver::parse_path("U7,R6,D4,L4").unwrap();
        assert_eq!(
            30,
            Solver::minimum_steps_to_intersection(&path1, &path2).unwrap()
        );
    }

    #[test]
    fn test_part2_ex2() {
        let path1 = Solver::parse_path("R75,D30,R83,U83,L12,D49,R71,U7,L72")
                           .unwrap();
        let path2 = Solver::parse_path("U62,R66,U55,R34,D71,R55,D58,R83")
                           .unwrap();
        assert_eq!(
            610,
            Solver::minimum_steps_to_intersection(&path1, &path2).unwrap()
        );
    }

    #[test]
    fn test_part2_ex3() {
        let path1 =
            Solver::parse_path("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51")
                   .unwrap();
        let path2 =
            Solver::parse_path("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7")
                   .unwrap();
        assert_eq!(
            410,
            Solver::minimum_steps_to_intersection(&path1, &path2).unwrap()
        );
    }
}

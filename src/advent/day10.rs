use crate::advent::AdventSolver;
use anyhow::{Error, format_err};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

#[derive(Default)]
pub struct Solver;

#[derive(Copy,Clone,PartialEq)]
enum Space {
    Empty,
    Asteroid,
}

#[derive(Copy,Clone,Debug,Default,PartialEq)]
struct Pos {
    x: usize,
    y: usize,
}

// (is_right, canonicalized slope)
#[derive(Copy,Clone,Eq,Hash,PartialEq)]
struct Direction {
    right: bool,
    slope: isize,
}

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let mut map_data = String::new();
        File::open("input/day10.txt")?
             .read_to_string(&mut map_data)?;

        let map = Solver::read_space_string(&map_data)?;
        let base = Solver::find_best_station_location(&map);
        Solver::animate_vaporization(&map, base);

        println!("Best monitoring station location: {:?} (asteroids: {})",
                 base, Solver::count_asteroids_seen(&map, base));

        println!("200th asteroid to vaporize: {:?}",
                 Solver::find_nth_vaporized(&map, base, 200).unwrap());
        Ok(())
    }
}

impl Solver {
    fn animate_vaporization(map: &Vec<Vec<Space>>, base: Pos) {
        let mut map = map.clone();
        let order = Self::vaporization_order(&map, base);
        print!("\x1B[2J\x1B[?25l"); // Clear screen, hide cursor
        Self::display_map(&map);
        for asteroid in order {
            thread::sleep(Duration::from_millis(20));
            map[asteroid.y][asteroid.x] = Space::Empty;
            Self::display_map(&map);
        }
        print!("\x1B[?25h"); // Show cursor
    }

    fn display_map(map: &Vec<Vec<Space>>) {
        print!("\x1B[H"); // move cursor to top-left
        for row in map.iter() {
            for &col in row.iter() {
                print!("{}", if col == Space::Empty { '.' } else { '#' });
            }
            println!("");
        }
        println!("");
    }
    fn read_space_string(text: &str) -> Result<Vec<Vec<Space>>, Error> {
        let lines = text.split("\n")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>();
        let mut result = Vec::new();
        for line in lines {
            let mut row = Vec::new();
            for c in line.chars() {
                row.push(match c {
                    '.' => Space::Empty,
                    '#' => Space::Asteroid,
                    _ => return Err(format_err!("Invalid char: {}", c)),
                });
            }
            result.push(row);
        }

        Ok(result)
    }

    fn find_best_station_location(map: &Vec<Vec<Space>>) -> Pos {
        let mut best_location = Pos::default();
        let mut best_asteroids_seen = 0;
        for y in 0..map.len() {
            for x in 0..map[y].len() {
                let location = Pos { x, y };
                if map[y][x] == Space::Asteroid {
                    let count = Self::count_asteroids_seen(map, location);
                    if count > best_asteroids_seen {
                        best_location = location;
                        best_asteroids_seen = count;
                    }
                }
            }
        }
        best_location
    }

    fn count_asteroids_seen(map: &Vec<Vec<Space>>, base: Pos) -> usize {
        Self::build_direction_map(map, base).len()
    }

    // Returns a map of directions to list of asteroids found in that
    // direction, in distance order.
    fn build_direction_map(map: &Vec<Vec<Space>>,
                           base: Pos) -> HashMap<Direction, Vec<Pos>> {
        let mut direction_map: HashMap<Direction, Vec<Pos>> = HashMap::new();
        for y in 0..map.len() {
            for x in 0..map[y].len() {
                // Ignore empty spaces
                if map[y][x] == Space::Empty {
                    continue;
                }

                // Ignore the base itself
                let asteroid = Pos { x, y };
                if asteroid == base {
                    continue;
                }

                // Trace path from base to asteroid, and stop if we are
                // blocked by another asteroid first.
                let direction = Direction::from_points(base, asteroid);
                if direction_map.contains_key(&direction) {
                    let asteroids = direction_map.get_mut(&direction).unwrap();
                    asteroids.push(asteroid);
                } else {
                    direction_map.insert(direction, vec![asteroid]);
                }
            }
        }
        // Sort by distance from base
        for (_, asteroids) in direction_map.iter_mut() {
            asteroids.sort_by_key(|a| a.distance_from(base));
        }
        direction_map
    }

    fn vaporization_order(map: &Vec<Vec<Space>>, base: Pos) -> Vec<Pos> {
        let mut direction_map = Self::build_direction_map(map, base);
        let mut result = Vec::new();

        let mut dirs = direction_map.keys()
                                    .map(|&d| d.clone())
                                    .collect::<Vec<Direction>>();
        dirs.sort();

        loop {
            // Sweep clockwise and hit asteroids in those directions
            for dir in dirs.iter() {
                let ref mut asteroids = direction_map.get_mut(dir).unwrap();
                if !asteroids.is_empty() {
                    result.push(asteroids.remove(0));
                }
            }
            // When no asteroids are left, we're done.
            if direction_map.values().all(|v| v.is_empty()) {
                break;
            }
        }

        result
    }

    fn find_nth_vaporized(map: &Vec<Vec<Space>>,
                          base: Pos, n: usize) -> Option<Pos> {
        // Using natural counting order here n = 1, 2, 3...
        let n = n - 1;

        let order = Self::vaporization_order(map, base);
        if order.len() > n {
            Some(order[n])
        } else {
            None
        }
    }
}

impl Direction {
    fn from_points(from: Pos, to: Pos) -> Direction {
        let from_x = from.x as isize;
        let from_y = from.y as isize;
        let to_x = to.x as isize;
        let to_y = to.y as isize;

        let xd = to_x - from_x;
        let yd = from_y - to_y; // Reversed because I want "up" to be y--

        let mut right = xd > 0;
        let slope = if xd == 0 {
            if yd < 0 {
                right = false;
            } else if yd > 0 {
                right = true;
            } else {
                // Tried to create a Direction with the same from/to point.
                panic!();
            }
            std::isize::MAX
        } else {
            // Hacky slope computation using integers since we can't count on
            // equality between floats. Hopefully this is enough precision.
            yd * 1_000_000 / xd * 1_000_000
        };
        Direction { right, slope }
    }
}

// Implement clockwise ordering of directions
impl Ord for Direction {
    fn cmp(&self, other: &Direction) -> Ordering {
        if self == other {
            Ordering::Equal
        } else if self.right && !other.right {
            // Right < Left
            Ordering::Less
        } else if !self.right && other.right {
            // Left > Right
            Ordering::Greater
        } else {
            // Descending order of slope
            other.slope.cmp(&self.slope)
        }
    }
}

// PartialOrd is required, but it just calls my Ord#cmp() function.
impl PartialOrd for Direction {
    fn partial_cmp(&self, other: &Direction) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Pos {
    // Return manhattan distance to other point
    fn distance_from(&self, other: Pos) -> usize {
        let (x1, y1) = (self.x as isize, self.y as isize);
        let (x2, y2) = (other.x as isize, other.y as isize);
        ((x1 - x2).abs() + (y1 - y2).abs()) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() {
        let text = r"
            .#..#
            .....
            #####
            ....#
            ...##
        ";
        let map = Solver::read_space_string(text).unwrap();
        assert_eq!(Pos {x: 3, y: 4}, Solver::find_best_station_location(&map));
    }

    #[test]
    fn test_example_2() {
        let text = r"
            ......#.#.
            #..#.#....
            ..#######.
            .#.#.###..
            .#..#.....
            ..#....#.#
            #..#....#.
            .##.#..###
            ##...#..#.
            .#....####
        ";
        let map = Solver::read_space_string(text).unwrap();
        assert_eq!(Pos {x: 5, y: 8}, Solver::find_best_station_location(&map));
    }

    #[test]
    fn test_example_3() {
        let text = r"
            #.#...#.#.
            .###....#.
            .#....#...
            ##.#.#.#.#
            ....#.#.#.
            .##..###.#
            ..#...##..
            ..##....##
            ......#...
            .####.###.
        ";
        let map = Solver::read_space_string(text).unwrap();
        assert_eq!(Pos {x: 1, y: 2}, Solver::find_best_station_location(&map));
    }

    #[test]
    fn test_example_4() {
        let text = r"
            .#..#..###
            ####.###.#
            ....###.#.
            ..###.##.#
            ##.##.#.#.
            ....###..#
            ..#.#..#.#
            #..#.#.###
            .##...##.#
            .....#.#..
        ";
        let map = Solver::read_space_string(text).unwrap();
        assert_eq!(Pos {x: 6, y: 3}, Solver::find_best_station_location(&map));
    }

    #[test]
    fn test_example_5() {
        let text = r"
            .#..##.###...#######
            ##.############..##.
            .#.######.########.#
            .###.#######.####.#.
            #####.##.#.##.###.##
            ..#####..#.#########
            ####################
            #.####....###.#.#.##
            ##.#################
            #####.##.###..####..
            ..######..##.#######
            ####.##.####...##..#
            .#####..#.######.###
            ##...#.##########...
            #.##########.#######
            .####.#.###.###.#.##
            ....##.##.###..#####
            .#.#.###########.###
            #.#.#.#####.####.###
            ###.##.####.##.#..##
        ";
        let map = Solver::read_space_string(text).unwrap();
        assert_eq!(Pos {x: 11, y: 13},
                   Solver::find_best_station_location(&map));

        // Part two, vaporization order
        let base = Pos {x: 11, y: 13};
        assert_eq!(Some(Pos {x: 11, y: 12}),
                   Solver::find_nth_vaporized(&map, base, 1));
        assert_eq!(Some(Pos {x: 12, y: 1}),
                   Solver::find_nth_vaporized(&map, base, 2));
        assert_eq!(Some(Pos {x: 12, y: 2}),
                   Solver::find_nth_vaporized(&map, base, 3));
        assert_eq!(Some(Pos {x: 12, y: 8}),
                   Solver::find_nth_vaporized(&map, base, 10));
        assert_eq!(Some(Pos {x: 16, y: 0}),
                   Solver::find_nth_vaporized(&map, base, 20));
        assert_eq!(Some(Pos {x: 16, y: 9}),
                   Solver::find_nth_vaporized(&map, base, 50));
        assert_eq!(Some(Pos {x: 10, y: 16}),
                   Solver::find_nth_vaporized(&map, base, 100));
        assert_eq!(Some(Pos {x: 9, y: 6}),
                   Solver::find_nth_vaporized(&map, base, 199));
        assert_eq!(Some(Pos {x: 8, y: 2}),
                   Solver::find_nth_vaporized(&map, base, 200));
        assert_eq!(Some(Pos {x: 10, y: 9}),
                   Solver::find_nth_vaporized(&map, base, 201));
        assert_eq!(Some(Pos {x: 11, y: 1}),
                   Solver::find_nth_vaporized(&map, base, 299));
    }
}

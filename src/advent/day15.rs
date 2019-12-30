use crate::advent::AdventSolver;
use crate::shared::intcode::{Program, Simulator, Sender, Receiver};
use crate::shared::grid::{InfiniteGrid, Dir, Pos};
use anyhow::{Error, format_err};
use num_derive::FromPrimitive;
use num_traits::cast::FromPrimitive;
use std::collections::VecDeque;

pub struct Solver {
    sim: Simulator,
    grid: InfiniteGrid<ShipSpace>,
    robot: Pos,
}

#[derive(Copy,Clone,Eq,PartialEq)]
enum ShipSpace {
    Unknown,
    Start,
    Empty,
    Oxygen,
    Wall,
}

#[derive(FromPrimitive)]
enum MovementResponse {
    HitWall,
    Success,
    FoundOxygenSystem,
}

impl Default for Solver {
    fn default() -> Self {
        Solver {
            sim: Simulator::new(),
            grid: InfiniteGrid::new(ShipSpace::Unknown),
            robot: Pos::origin(),
        }
    }
}

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let program = Program::from_path("input/day15.txt")?;
        self.sim.load_program(&program);

        // Robot does not start inside a wall
        self.grid[self.robot] = ShipSpace::Start;

        // Perform a depth-first search to map the ship
        let input = self.sim.create_input_channel();
        let output = self.sim.create_output_channel();
        self.map_ship(&input, &output)?;
        assert_eq!(Pos::origin(), self.robot);

        self.show_grid();
        println!("Steps to oxygen system: {}",
                 self.steps_to_oxygen(Pos::origin()));

        println!("Time to fill with oxygen: {}",
                 self.fill_with_oxygen());

        Ok(())
    }
}

impl Solver {
    // Problem doesn't use the same order as my enum, hence:
    fn dir_to_int(dir: &Dir) -> isize {
        match dir {
            Dir::Up => 1,
            Dir::Down => 2,
            Dir::Left => 3,
            Dir::Right => 4,
        }
    }

    // Recursively explore any unmapped neighbors of robot
    fn map_ship(&mut self, input: &Sender,
                output: &Receiver) -> Result<(), Error> {
        for &dir in [Dir::Up, Dir::Down, Dir::Left, Dir::Right].iter() {
            let neighbor = self.robot.neighbor(dir);
            if self.grid[neighbor] == ShipSpace::Unknown {
                input.send(Self::dir_to_int(&dir))?;
                self.sim.run()?;
                let mut moved = false;
                let o = output.recv()?;
                let r = MovementResponse::from_isize(o)
                            .ok_or(format_err!("Bad response: {}", o))?;
                match r {
                    MovementResponse::HitWall => {
                        self.grid[neighbor] = ShipSpace::Wall;
                    },
                    MovementResponse::Success => {
                        moved = true;
                        self.grid[neighbor] = ShipSpace::Empty;
                        self.robot = neighbor;
                    },
                    MovementResponse::FoundOxygenSystem => {
                        moved = true;
                        self.grid[neighbor] = ShipSpace::Oxygen;
                        self.robot = neighbor;
                    },
                }
                if moved {
                    let reverse = dir.reverse();
                    self.map_ship(input, output)?;
                    input.send(Self::dir_to_int(&reverse))?;
                    self.robot = self.robot.neighbor(reverse); 
                    self.sim.run()?;
                    output.recv()?;
                }
            }
        }
        Ok(())
    }

    // BFS
    fn steps_to_oxygen(&self, start: Pos) -> usize {
        let mut queue: VecDeque<(Pos, Dir, usize)> = VecDeque::new();
        queue.push_back((start, Dir::Up, 0));

        loop {
            let (pos, orig_dir, dist) = queue.pop_front().unwrap();
            if self.grid[pos] == ShipSpace::Oxygen {
                return dist;
            }
            for &dir in [Dir::Up, Dir::Down, Dir::Left, Dir::Right].iter() {
                if dir == orig_dir && pos != start {
                    continue;
                }
                let neighbor = pos.neighbor(dir);
                match self.grid[neighbor] {
                    ShipSpace::Wall => {},
                    ShipSpace::Oxygen => return dist+1,
                    _ => queue.push_back((neighbor, dir.reverse(), dist+1)),
                }
            }
        }
    }

    // This is pretty inefficient but whatevs
    fn fill_with_oxygen(&self) -> usize {
        let mut grid = self.grid.crop();
        let mut time = 0;

        while !Self::is_filled(&grid) {
            let mut to_fill = Vec::new();
            for row in 0..grid.len() {
                for col in 0..grid[row].len() {
                    if grid[row][col] == ShipSpace::Oxygen {
                        if row > 0 {
                            to_fill.push((row-1, col));
                        }
                        if col > 0 {
                            to_fill.push((row, col-1));
                        }
                        if row < grid.len()-1 {
                            to_fill.push((row+1, col));
                        }
                        if col < grid[row].len()-1 {
                            to_fill.push((row, col+1));
                        }
                    }
                }
            }
            for (row, col) in to_fill {
                match grid[row][col] {
                    ShipSpace::Empty | ShipSpace::Start => {
                        grid[row][col] = ShipSpace::Oxygen;
                    },
                    _ => {},
                }
            }
            time += 1;
        }
        time
    }

    fn is_filled(grid: &Vec<Vec<ShipSpace>>) -> bool {
        for row in grid {
            for &col in row {
                if col == ShipSpace::Empty || col == ShipSpace::Start {
                    return false;
                }
            }
        }
        true
    }

    fn show_grid(&self) {
        let grid = self.grid.crop();
        for row in 0..grid.len() {
            for col in 0..grid[row].len() {
                print!("{}", match grid[row][col] {
                    ShipSpace::Unknown => "?",
                    ShipSpace::Empty => " ",
                    ShipSpace::Start => "S",
                    ShipSpace::Oxygen => "O",
                    ShipSpace::Wall => "#",
                });
            }
            println!("");
        }
    }
}

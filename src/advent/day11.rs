use crate::advent::AdventSolver;
use crate::shared::grid::{Dir, InfiniteGrid, Pos};
use crate::shared::intcode::{Program, Simulator};
use anyhow::{Error, format_err};
use std::collections::HashSet;

#[derive(Default)]
pub struct Solver;

#[derive(Copy,Clone,PartialEq)]
enum Color {
    Black = 0,
    White = 1,
}

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let program = Program::from_path("input/day11.txt")?;

        // Part 1
        Solver::paint_registration(&program, Color::Black)?;
        // This one is garbage, don't bother displaying it.

        // Part 2
        let grid = Solver::paint_registration(&program, Color::White)?;
        Solver::display_grid(&grid);

        Ok(())
    }
}

impl Solver {
    fn paint_registration(program: &Program,
                          start_color: Color) ->
                              Result<Vec<Vec<Color>>, Error> {
        let mut grid: InfiniteGrid<Color> = InfiniteGrid::new(Color::Black);
        let mut robot_pos = Pos::origin();
        let mut robot_dir = Dir::Up;
        let mut panels_painted: HashSet<Pos> = HashSet::new();
        let mut sim = Simulator::with_program(program);
        let input = sim.create_input_channel();
        let output = sim.create_output_channel();
        sim.set_blocking_input(false);

        grid[robot_pos] = start_color;

        loop {
            input.send(grid[robot_pos] as isize)?;
            sim.run()?;
            if !sim.is_running() {
                break;
            }

            grid[robot_pos] = match output.recv()? {
                0 => Color::Black,
                1 => Color::White,
                x => return Err(format_err!("Unrecognized color: {}", x)),
            };
            panels_painted.insert(robot_pos);
            match output.recv()? {
                0 => robot_dir = robot_dir.turn_left(),
                1 => robot_dir = robot_dir.turn_right(),
                x => return Err(format_err!("Unrecognized turn: {}", x)),
            }
            robot_pos = robot_pos.neighbor(robot_dir);
        }

        println!("Panels painted: {}", panels_painted.len());
        Ok(grid.crop())
    }

    fn display_grid(grid: &Vec<Vec<Color>>) {
        for row in grid {
            for col in row {
                print!("{}", match col {
                    Color::Black => ' ',
                    Color::White => '\u{2588}',
                });
            }
            println!("");
        }
        println!("");
    }
}

use crate::advent::AdventSolver;
use crate::shared::grid::{InfiniteGrid, Pos};
use crate::shared::intcode::{Program, Simulator};
use anyhow::{Error, format_err};
use num_derive::FromPrimitive;
use num_traits::cast::FromPrimitive;
use std::thread;
use std::time;

#[derive(Default)]
pub struct Solver;

#[derive(Clone,Copy,FromPrimitive,Eq,PartialEq)]
enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let program = Program::from_path("input/day13.txt")?;
        let mut screen: InfiniteGrid<Tile> = InfiniteGrid::new(Tile::Empty);
        let mut sim = Simulator::with_program(&program);
        let input = sim.create_input_channel();
        let output = sim.create_output_channel();
        let mut score: isize = 0;

        print!("\x1B[2J\x1B[?25l"); // Clear screen, hide cursor
        sim.poke(0, 2); // Insert quarter

        loop {
            sim.run()?;

            // Read any buffered output and then render screen
            while let Ok(x) = output.try_recv() {
                if x == -1 {
                    output.recv()?;
                    score = output.recv()?;
                } else {
                    let pos = Pos::new(output.recv()?, x);
                    let tile = output.recv()?;
                    match Tile::from_isize(tile) {
                        Some(tile) => screen[pos] = tile,
                        None => return Err(format_err!("Bad tile: {}", tile)),
                    }
                }
                Self::display_screen(&screen, score);
            }
            if !sim.is_running() {
                break;
            }

            // Move paddle toward ball
            let ball_col = Self::find_tile(&screen, Tile::Ball);
            let paddle_col = Self::find_tile(&screen, Tile::Paddle);
            if ball_col.is_some() && paddle_col.is_some() {
                if ball_col.unwrap() < paddle_col.unwrap() {
                    input.send(-1)?;
                } else if ball_col.unwrap() == paddle_col.unwrap() {
                    input.send(0)?;
                } else {
                    input.send(1)?;
                }
            }
        }
        print!("\x1B[?25h"); // Show cursor

        Ok(())
    }
}

impl Solver {
    // Used for part 1
    #[allow(dead_code)]
    fn block_count(screen: &InfiniteGrid<Tile>) -> usize {
        let cropped = screen.crop();
        cropped.iter()
               .map(|row| row.iter().filter(|&t| t == &Tile::Block).count())
               .fold(0, |sum, c| sum + c)
    }

    fn display_screen(screen: &InfiniteGrid<Tile>, score: isize) {
        // Hack: Only display screen if ball & paddle are present (reduce
        // flicker when ball erased and speeds up initial screen paint).
        if Self::find_tile(&screen, Tile::Ball).is_none() ||
           Self::find_tile(&screen, Tile::Paddle).is_none() {
            return;
        }
        let cropped = screen.crop();
        print!("\x1B[H"); // move cursor to top left
        println!("\x1B[36;1mScore: {}", score);
        for row in cropped {
            for tile in row {
                print!("{}", match tile {
                    Tile::Empty => " ",
                    Tile::Wall => "\x1B[37;1m\u{2588}",
                    Tile::Block => "\x1B[34m\u{2584}",
                    Tile::Paddle => "\x1B[32;1m\u{2501}",
                    Tile::Ball => "\x1B[31;1m\u{2b24}",
                });
            }
            print!("\n");
        }
        thread::sleep(time::Duration::from_millis(10));
    }

    fn find_tile(screen: &InfiniteGrid<Tile>, to_find: Tile) -> Option<isize> {
        let screen = screen.crop();
        for row in screen {
            for col in 0..row.len() {
                if row[col] == to_find {
                    return Some(col as isize);
                }
            }
        }
        None
    }
}

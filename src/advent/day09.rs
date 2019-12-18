use crate::advent::AdventSolver;
use crate::shared::intcode::{Program, Simulator};
use anyhow::Error;

#[derive(Default)]
pub struct Solver;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let program = Program::from_path("input/day09.txt")?;

        println!("Part 1, running BOOST test:");
        Self::run_boost_program(&program, 1)?;

        println!("Part 2, running sensor boost:");
        Self::run_boost_program(&program, 2)?;

        Ok(())
    }
}

impl Solver {
    fn run_boost_program(program: &Program,
                         input_value: isize) -> Result<(), Error> {
        let mut sim = Simulator::with_program(program);
        let input = sim.create_input_channel();
        let output = sim.create_output_channel();
        input.send(input_value)?;
        sim.run()?;
        loop {
            match output.try_recv() {
                Ok(v) => { println!("Read: {}", v); }
                Err(_) => { break; },
            }
        }
        Ok(())
    }
}


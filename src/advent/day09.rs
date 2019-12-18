use crate::advent::AdventSolver;
use crate::shared::intcode::IntcodeProgram;
use anyhow::Error;

#[derive(Default)]
pub struct Solver;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let program = IntcodeProgram::from_path("input/day09.txt")?;

        println!("Part 1, running BOOST test:");
        Self::run_boost_program(program.clone(), 1)?;

        println!("Part 2, running sensor boost:");
        Self::run_boost_program(program.clone(), 2)?;

        Ok(())
    }
}

impl Solver {
    fn run_boost_program(mut program: IntcodeProgram,
                         input_value: isize) -> Result<(), Error> {
        let input = program.create_input_channel();
        let output = program.create_output_channel();
        input.send(input_value)?;
        program.run()?;
        loop {
            match output.try_recv() {
                Ok(v) => { println!("Read: {}", v); }
                Err(_) => { break; },
            }
        }
        Ok(())
    }
}


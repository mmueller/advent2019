use crate::advent::AdventSolver;
use crate::shared::intcode::{Program, Simulator};
use anyhow::Error;

#[derive(Default)]
pub struct Solver;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let program = Program::from_path("input/day02.txt")?;

        Self::run_1202(&program)?;
        Self::find_inputs(&program, 19690720)?;
        Ok(())
    }
}

impl Solver {

    // Run the given program with the specified input values, returns output.
    // (Here, "input" means addresses 1 & 2, "output" means address 0.)
    fn run(program: &Program, input1: isize,
           input2: isize) -> Result<isize, Error> {
        let mut sim = Simulator::with_program(&program);
        sim.poke(1, input1);
        sim.poke(2, input2);
        sim.run()?;
        Ok(sim.peek(0))
    }

    fn run_1202(program: &Program) -> Result<(), Error> {
        println!("Output for 1202 input: {}",
                 Self::run(program, 12, 2)?);
        Ok(())
    }

    fn find_inputs(program: &Program, output: isize) -> Result<(), Error> {
        'outer: for input1 in 0..=99 {
            for input2 in 0..=99 {
                if Self::run(program, input1, input2)? == output {
                    println!("Inputs {} and {} produce output {}.",
                             input1, input2, output);
                    break 'outer;
                }
            }
        }
        Ok(())
    }
}

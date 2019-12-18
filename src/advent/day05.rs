use crate::advent::AdventSolver;
use crate::shared::intcode::{Program, Simulator};
use anyhow::Error;

#[derive(Default)]
pub struct Solver;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        println!("Diagnostic test of system 1: {}",
                 Self::run_diagnostic_test(1)?);
        println!("Diagnostic test of system 5: {}",
                 Self::run_diagnostic_test(5)?);
        Ok(())
    }
}

impl Solver {
    fn run_diagnostic_test(input: isize) -> Result<isize, Error> {
        let program = Program::from_path("input/day05.txt")?;
        let mut sim = Simulator::with_program(&program);
        let input_sender = sim.create_input_channel();
        let output_receiver = sim.create_output_channel();
        input_sender.send(input)?;
        sim.run()?;
        let mut result = 0;
        while result == 0 {
            result = output_receiver.recv()?;
        }
        Ok(result)
    }
}

use crate::advent::AdventSolver;
use crate::shared::intcode::IntcodeProgram;
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
        let mut program = IntcodeProgram::from_path("input/day05.txt")?;
        let mut result = 0;
        let input = || input;
        let mut output = |v| {
            match v {
                0 => Ok(()),
                code @ _ => { result = code; Ok(()) },
            }
        };
        program.connect_input(&input);
        program.connect_output(&mut output);
        program.run()?;
        Ok(result)
    }
}

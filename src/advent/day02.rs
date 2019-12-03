use crate::advent::AdventSolver;
use crate::shared::intcode::IntcodeProgram;
use anyhow::Error;

#[derive(Default)]
pub struct Solver;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let program = IntcodeProgram::from_path("input/day02.txt")?;

        Self::run_1202(program.clone())?;
        Self::find_inputs(program.clone(), 19690720)?;
        Ok(())
    }
}

impl Solver {
    fn run_1202(mut program: IntcodeProgram) -> Result<(), Error> {
        program.set_inputs(12, 2);
        let output = program.run()?;

        println!("Output for 1202 input: {}", output);
        Ok(())
    }

    fn find_inputs(program: IntcodeProgram,
                   output: usize) -> Result<(), Error> {
        'outer: for input1 in 0..=99 {
            for input2 in 0..=99 {
                let mut tmp_prog = program.clone();
                tmp_prog.set_inputs(input1, input2);
                tmp_prog.run()?;
                if tmp_prog.output() == output {
                    println!("Inputs {} and {} produce output {}.",
                             input1, input2, output);
                    break 'outer;
                }
            }
        }
        Ok(())
    }
}

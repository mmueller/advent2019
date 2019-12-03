use anyhow::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;

#[derive(Clone)]
pub struct IntcodeProgram {
    inst: Vec<usize>,
    pc: usize,
    state: IntcodeProgramState,
}

#[derive(Clone,PartialEq)]
pub enum IntcodeProgramState {
    Running,
    Halted,
}

impl IntcodeProgram {
    pub fn from_path(path: &str) -> Result<Self, Error> {
        let mut text = String::new();
        File::open(path)?.read_to_string(&mut text)?;
        Self::from_string(text.trim())
    }

    pub fn from_string(text: &str) -> Result<Self, Error> {
        let inst: Vec<usize> = text.split(',')
                                   .map(|s| s.parse::<usize>())
                                   .collect::<Result<Vec<usize>, _>>()?;
        Ok(Self {
            inst: inst,
            pc: 0,
            state: IntcodeProgramState::Running,
        })
    }

    // Same as run, but displays program state at each step.
    #[allow(dead_code)]
    pub fn debug(&mut self) -> Result<usize, Error> {
        self.run_program(true)
    }

    pub fn is_running(&self) -> bool {
        self.state == IntcodeProgramState::Running
    }

    pub fn output(&self) -> usize {
        self.inst[0]
    }

    pub fn run(&mut self) -> Result<usize, Error> {
        self.run_program(false)
    }

    // Run implementation, with optional debug output.
    fn run_program(&mut self, debug: bool) -> Result<usize, Error> {
        while self.is_running() {
            if debug {
                println!("{:?}", self);
            }
            self.step()?;
        }
        Ok(self.output())
    }

    pub fn set_inputs(&mut self, input1: usize, input2: usize) {
        self.inst[1] = input1;
        self.inst[2] = input2;
    }

    pub fn step(&mut self) -> Result<(), Error> {
        let op = self.inst[self.pc];
        match op {
            // Binary operations
            1 | 2 => {
                let operand1 = self.inst[self.inst[self.pc+1]];
                let operand2 = self.inst[self.inst[self.pc+2]];
                let target = self.inst[self.pc+3];
                self.inst[target] = match op {
                    1 => operand1 + operand2,
                    2 => operand1 * operand2,
                    _ => unreachable!(),
                };
                self.pc += 4;
            },
            99 => {
                self.state = IntcodeProgramState::Halted;
            },
            _ => {
                return Err(anyhow::format_err!("Invalid opcode: {}",
                                               self.inst[self.pc]));
            },
        }
        Ok(())
    }
}

impl fmt::Debug for IntcodeProgram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inst.iter()
                                 .map(|i| i.to_string())
                                 .collect::<Vec<String>>()
                                 .join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to run a program given in the string and return the output value.
    fn run_program(text: &str) -> usize {
        let mut program = IntcodeProgram::from_string(text).unwrap();
        program.run().unwrap()
    }

    // Helper to debug a program (useful if a test is failing)
    #[allow(dead_code)]
    fn debug_program(text: &str) -> usize {
        let mut program = IntcodeProgram::from_string(text).unwrap();
        program.debug().unwrap()
    }

    #[test]
    fn test_empty_program() {
        assert_eq!(99, run_program("99"));
    }

    #[test]
    fn test_add() {
        // 1 + 1 = 2
        assert_eq!(2, run_program("1,0,0,0,99"));

        // 1 + 1 + 1 = 3
        assert_eq!(3, run_program("1,0,0,1,1,0,1,0,99"));
    }

    #[test]
    fn test_multiply() {
        // 2 + 3 = 6
        assert_eq!(6, run_program("2,0,5,0,99,3"));

        // 2 * 3 * 4 = 24
        assert_eq!(24, run_program("2,0,9,1,2,10,1,0,99,3,4"));
    }

    #[test]
    fn test_day2_examples() {
        assert_eq!(3500, run_program("1,9,10,3,2,3,11,0,99,30,40,50"));
        assert_eq!(30, run_program("1,1,1,4,99,5,6,0,99"));
    }
}

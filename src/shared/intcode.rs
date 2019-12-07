use anyhow::{Error, format_err};
use std::fmt;
use std::fs::File;
use std::io::Read;

type InputCallback<'a> = dyn Fn()->isize + 'a;
type OutputCallback<'a> = dyn FnMut(isize)->Result<(), Error> + 'a;

pub struct IntcodeProgram<'a> {
    inst: Vec<isize>,
    pc: usize,
    state: IntcodeProgramState,
    input: Option<&'a InputCallback<'a>>,
    output: Option<&'a mut OutputCallback<'a>>,
}

#[derive(Clone,PartialEq)]
pub enum IntcodeProgramState {
    Running,
    Halted,
}

#[derive(Clone,PartialEq)]
pub enum ParameterMode {
    Position,
    Immediate,
}

impl<'a> IntcodeProgram<'a> {
    pub fn from_path(path: &str) -> Result<Self, Error> {
        let mut text = String::new();
        File::open(path)?.read_to_string(&mut text)?;
        Self::from_string(text.trim())
    }

    pub fn from_string(text: &str) -> Result<Self, Error> {
        let inst: Vec<isize> = text.split(',')
                                   .map(|s| s.parse::<isize>())
                                   .collect::<Result<Vec<isize>, _>>()?;
        Ok(Self {
            inst: inst,
            pc: 0,
            state: IntcodeProgramState::Running,
            input: None,
            output: None,
        })
    }

    pub fn connect_input(&mut self, input: &'a InputCallback<'a>) {
        self.input = Some(input);
    }

    #[allow(dead_code)]
    pub fn disconnect_input(&mut self) {
        self.input = None;
    }

    pub fn connect_output(&mut self, output: &'a mut OutputCallback<'a>) {
        self.output = Some(output);
    }

    #[allow(dead_code)]
    pub fn disconnect_output(&mut self) {
        self.output = None;
    }

    // Same as run, but displays program state at each step.
    #[allow(dead_code)]
    pub fn debug(&mut self) -> Result<isize, Error> {
        self.run_program(true)
    }

    pub fn is_running(&self) -> bool {
        self.state == IntcodeProgramState::Running
    }

    pub fn output(&self) -> isize {
        self.inst[0]
    }

    pub fn run(&mut self) -> Result<isize, Error> {
        self.run_program(false)
    }

    // Set "inputs" as described in day 2, which means the values as positions 1
    // and 2 in the program. Not the same as the input op.
    pub fn set_inputs(&mut self, input1: isize, input2: isize) {
        self.inst[1] = input1;
        self.inst[2] = input2;
    }

    pub fn step(&mut self) -> Result<(), Error> {
        let op = self.inst[self.pc];
        match op % 100 {
            // Binary operations
            1 | 2 | 7 | 8 => {
                let param0 = self.get_param(op, 0)?;
                let param1 = self.get_param(op, 1)?;
                let target = self.get_param(0, 2)?;
                self.inst[target as usize] = match op % 100 {
                    1 => param0 + param1,
                    2 => param0 * param1,
                    7 => if param0 < param1 { 1 } else { 0 },
                    8 => if param0 == param1 { 1 } else { 0 },
                    _ => unreachable!(),
                };
                self.pc += 4;
            },
            // Input
            3 => {
                let target = self.get_param(0, 0)?;
                match self.input {
                    Some(ref f) => {
                        self.inst[target as usize] = f();
                    },
                    None => {
                        return Err(format_err!(
                            "Input op called without input callback."));
                    },
                }
                self.pc += 2;
            },
            // Output
            4 => {
                let value = self.get_param(op, 0)?;
                match self.output {
                    Some(ref mut f) => {
                        f(value)?;
                    },
                    None => {
                        return Err(format_err!(
                                "Output op called without output callback."));
                    },
                }
                self.pc += 2;
            },
            // Jump-if-true and jump-if-false
            5 | 6 => {
                let val = self.get_param(op, 0)?;
                let addr = self.get_param(op, 1)?;
                if op % 100 == 5 && val != 0 || op  % 100== 6 && val == 0 {
                    self.pc = addr as usize;
                } else {
                    self.pc += 3;
                }
            },
            99 => {
                self.state = IntcodeProgramState::Halted;
            },
            _ => {
                return Err(format_err!("Invalid opcode: {}",
                                       self.inst[self.pc]));
            },
        }
        Ok(())
    }

    // Private

    fn get_param(&self, op: isize, param_no: usize) -> Result<isize, Error> {
        let mode = Self::parameter_mode(op, param_no)?;
        let value = self.inst[self.pc + param_no + 1];
        match mode {
            ParameterMode::Position => {
                Ok(self.inst[value as usize])
            },
            ParameterMode::Immediate => {
                Ok(value)
            },
        }
    }

    // Run implementation, with optional debug output.
    fn run_program(&mut self, debug: bool) -> Result<isize, Error> {
        while self.is_running() {
            if debug {
                println!("{:?}", self);
            }
            self.step()?;
        }
        Ok(self.output())
    }

    fn parameter_mode(op: isize, param: usize) -> Result<ParameterMode, Error> {
        // Hack: Use 0 op for write targets, which are addresses but we need
        // their value.
        if op == 0 {
            return Ok(ParameterMode::Immediate);
        }
        let mut op = op as usize;
        op /= 100 * 10_usize.pow(param as u32);
        match op % 10 {
            0 => Ok(ParameterMode::Position),
            1 => Ok(ParameterMode::Immediate),
            _ => Err(format_err!("Invalid parameter mode: {}", op % 10)),
        }
    }
}

// Cloning a program does not include the input/output callbacks.
impl<'a> Clone for IntcodeProgram<'a> {
    fn clone(&self) -> Self {
        Self {
            inst: self.inst.clone(),
            pc: self.pc,
            state: self.state.clone(),
            input: None,
            output: None,
        }
    }
}

impl<'a> fmt::Debug for IntcodeProgram<'a> {
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

    // Helper to run a program given in the string and return the value that
    // ends up in position 0.
    fn run_program(text: &str) -> isize {
        let mut program = IntcodeProgram::from_string(text).unwrap();
        program.run().unwrap()
    }

    // Helper to debug a program (useful if a test is failing)
    #[allow(dead_code)]
    fn debug_program(text: &str) -> isize {
        let mut program = IntcodeProgram::from_string(text).unwrap();
        program.debug().unwrap()
    }

    // Helper to run a program with a particular input value and return the
    // output value it yields (using the callbacks).
    fn run_program_io(text: &str, input: isize) -> Result<isize, Error> {
        let mut program = IntcodeProgram::from_string(text).unwrap();
        let mut result = 0;
        let input = || input;
        let mut output = |v| {
            result = v;
            Ok(())
        };
        program.connect_input(&input);
        program.connect_output(&mut output);
        program.run()?;
        Ok(result)
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

    #[test]
    fn test_day5_example0() {
        match run_program_io("3,9,8,9,10,9,4,9,99,-1,8", 7) {
            Ok(v) => assert_eq!(0, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io("3,9,8,9,10,9,4,9,99,-1,8", 8) {
            Ok(v) => assert_eq!(1, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io("3,9,7,9,10,9,4,9,99,-1,8", 5) {
            Ok(v) => assert_eq!(1, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io("3,9,7,9,10,9,4,9,99,-1,8", 8) {
            Ok(v) => assert_eq!(0, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io("3,3,1108,-1,8,3,4,3,99", 7) {
            Ok(v) => assert_eq!(0, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io("3,3,1108,-1,8,3,4,3,99", 8) {
            Ok(v) => assert_eq!(1, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io("3,3,1107,-1,8,3,4,3,99", 5) {
            Ok(v) => assert_eq!(1, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io("3,3,1107,-1,8,3,4,3,99", 8) {
            Ok(v) => assert_eq!(0, v),
            Err(e) => assert!(false, e.to_string()),
        }
    }

    #[test]
    fn test_day5_example1() {
        match run_program_io("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9", 0) {
            Ok(v) => assert_eq!(0, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9", 555) {
            Ok(v) => assert_eq!(1, v),
            Err(e) => assert!(false, e.to_string()),
        }
    }

    #[test]
    fn test_day5_example2a() {
        match run_program_io(
            concat!("3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,",
                    "1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,",
                    "999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99"),
            5)
        {
            Ok(v) => assert_eq!(999, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io(
            concat!("3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,",
                    "1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,",
                    "999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99"),
            8)
        {
            Ok(v) => assert_eq!(1000, v),
            Err(e) => assert!(false, e.to_string()),
        }
        match run_program_io(
            concat!("3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,",
                    "1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,",
                    "999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99"),
            529)
        {
            Ok(v) => assert_eq!(1001, v),
            Err(e) => assert!(false, e.to_string()),
        }
    }

    #[test]
    fn test_negative_values_ok() {
        assert_eq!(-1, run_program("1,0,5,0,99,-2"));
    }

    #[test]
    fn test_immediate_mode() {
        assert_eq!(1005, run_program("1001,0,4,0,99"));
    }

    #[test]
    fn test_input() {
        let mut program = IntcodeProgram::from_string("3,0,99").unwrap();
        program.connect_input(&|| 5);
        assert_eq!(5, program.run().unwrap());
    }

    #[test]
    fn test_output() {
        let mut result = 0;
        let mut save_output = |v| {
            result = v;
            Ok(())
        };
        let mut program = IntcodeProgram::from_string("4,0,99").unwrap();
        program.connect_output(&mut save_output);
        assert!(program.run().is_ok());
        assert_eq!(4, result);
    }

    #[test]
    fn test_output_error() {
        // Errors from the output callback should stop program execution
        let mut program = IntcodeProgram::from_string("4,0,99").unwrap();
        let mut output = |_| {
            Err(format_err!("whoops"))
        };
        program.connect_output(&mut output);
        assert!(program.run().is_err());
    }
}

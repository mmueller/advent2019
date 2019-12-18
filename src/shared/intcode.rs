use anyhow::{Error, format_err};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Receiver, Sender};

type IntcodeReceiver = Receiver<isize>;
type IntcodeSender = Sender<isize>;

pub struct IntcodeProgram {
    inst: Vec<isize>,
    pc: usize,
    state: IntcodeProgramState,
    relative_base: usize,
    input_reader: Option<IntcodeReceiver>,
    input_sender: Option<IntcodeSender>,
    output_sender: Option<IntcodeSender>,
    last_output: isize,
    blocking_input: bool,
    waiting_for_input: bool,
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
    Relative,
}

#[derive(Clone,Debug,PartialEq)]
pub enum Parameter {
    Address(usize),
    Value(isize),
}

impl IntcodeProgram {
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
            relative_base: 0,
            input_sender: None,
            input_reader: None,
            output_sender: None,
            last_output: 0,
            blocking_input: true,
            waiting_for_input: false,
        })
    }

    // Creates a channel for you and returns the sender half.
    pub fn create_input_channel(&mut self) -> IntcodeSender {
        let (sender, receiver) = channel();
        self.input_reader = Some(receiver);
        self.input_sender = Some(sender.clone());
        sender
    }

    pub fn connect_input(&mut self, input: IntcodeReceiver,
                         sender: IntcodeSender) {
        self.input_reader = Some(input);
        self.input_sender = Some(sender);
    }

    pub fn send_input(&self, value: isize) -> Result<(), Error> {
        match &self.input_sender {
            Some(ref sender) => {
                sender.send(value)?;
                Ok(())
            },
            None => {
                Err(format_err!("No input channel available."))
            }
        }
    }

    #[allow(dead_code)]
    pub fn disconnect_input(&mut self) {
        self.input_reader = None;
        self.input_sender = None;
    }

    // Creates a channel for you and returns the receiver half.
    pub fn create_output_channel(&mut self) -> IntcodeReceiver {
        let (sender, receiver) = channel();
        self.output_sender = Some(sender);
        receiver
    }

    pub fn connect_output(&mut self, output: IntcodeSender) {
        self.output_sender = Some(output);
    }

    #[allow(dead_code)]
    pub fn disconnect_output(&mut self) {
        self.output_sender = None;
    }

    pub fn last_output(&self) -> isize {
        self.last_output
    }

    // Same as run, but displays program state at each step.
    #[allow(dead_code)]
    pub fn debug(&mut self) -> Result<isize, Error> {
        self.run_program(true)
    }

    pub fn is_running(&self) -> bool {
        self.state == IntcodeProgramState::Running
    }

    // "output" as defined in day 2 (value at addr 0), not the channels
    pub fn output(&self) -> isize {
        self.inst[0]
    }

    // Set "inputs" as described in day 2, which means the values as positions 1
    // and 2 in the program. Not the same as the input op.
    pub fn set_inputs(&mut self, input1: isize, input2: isize) {
        self.inst[1] = input1;
        self.inst[2] = input2;
    }

    pub fn run(&mut self) -> Result<isize, Error> {
        self.run_program(false)
    }

    pub fn run_until_input_needed(&mut self) -> Result<(), Error> {
        while self.is_running() {
            self.step()?;
            if self.waiting_for_input() {
                break;
            }
        }
        Ok(())
    }

    pub fn set_blocking_input(&mut self, blocking: bool) {
        self.blocking_input = blocking;
    }

    pub fn waiting_for_input(&self) -> bool {
        self.waiting_for_input
    }

    pub fn step(&mut self) -> Result<(), Error> {
        let op = self.inst[self.pc];
        match op % 100 {
            // Binary operations
            1 | 2 | 7 | 8 => {
                let param0 = self.load(self.get_param(0)?);
                let param1 = self.load(self.get_param(1)?);
                let target = self.get_param(2)?;
                self.store(target, match op % 100 {
                    1 => param0 + param1,
                    2 => param0 * param1,
                    7 => if param0 < param1 { 1 } else { 0 },
                    8 => if param0 == param1 { 1 } else { 0 },
                    _ => unreachable!(),
                })?;
                self.pc += 4;
            },
            // Input
            3 => {
                let target = self.get_param(0)?;
                match self.input_reader {
                    Some(ref receiver) => {
                        if self.blocking_input {
                            let value = receiver.recv()?;
                            self.store(target, value)?;
                        } else {
                            match receiver.try_recv() {
                                Ok(value) => {
                                    self.store(target, value)?;
                                    self.waiting_for_input = false;
                                },
                                Err(_) => {
                                    self.waiting_for_input = true;
                                }
                            }
                        }
                    },
                    None => {
                        return Err(format_err!(
                            "Input op called without input channel."));
                    },
                }
                if !self.waiting_for_input {
                    self.pc += 2;
                }
            },
            // Output
            4 => {
                let value = self.load(self.get_param(0)?);
                match self.output_sender {
                    Some(ref sender) => {
                        sender.send(value)?;
                        self.last_output = value;
                    },
                    None => {
                        return Err(format_err!(
                            "Output op called without output channel."));
                    },
                }
                self.pc += 2;
            },
            // Jump-if-true and jump-if-false
            5 | 6 => {
                let val = self.load(self.get_param(0)?);
                let addr = self.load(self.get_param(1)?);
                if op % 100 == 5 && val != 0 || op % 100 == 6 && val == 0 {
                    self.pc = addr as usize;
                } else {
                    self.pc += 3;
                }
            },
            // Adjust relative base
            9 => {
                let val = self.load(self.get_param(0)?);
                let new_base = self.relative_base as isize + val;
                if new_base < 0 {
                    return Err(format_err!(
                        "Overflow in opcode 9: base would be {}", new_base));
                }
                self.relative_base = new_base as usize;
                self.pc += 2;
            },
            99 => {
                self.state = IntcodeProgramState::Halted;
            },
            _ => {
                return Err(format_err!("Invalid opcode: {} (pc: {})",
                                       self.inst[self.pc], self.pc));
            },
        }
        Ok(())
    }

    // Private

    // Get the ith parameter (0-based) for the current instruction.
    fn get_param(&self, i: usize) -> Result<Parameter, Error> {
        let mode = Self::parameter_mode(self.inst[self.pc], i)?;
        let raw_value = self.inst[self.pc + i + 1];
        match mode {
            ParameterMode::Position => {
                Parameter::to_address(raw_value)
            },
            ParameterMode::Immediate => {
                Ok(Parameter::Value(raw_value))
            },
            ParameterMode::Relative => {
                Parameter::to_address(self.relative_base as isize + raw_value)
            },
        }
    }

    // Read the value of the parameter, dereferencing if it is an address.
    fn load(&self, param: Parameter) -> isize {
        match param {
            Parameter::Address(addr) => {
                if addr >= self.inst.len() {
                    0
                } else {
                    self.inst[addr]
                }
            },
            Parameter::Value(value) => value,
        }
    }

    // Target must be an address
    fn store(&mut self, target: Parameter, value: isize) -> Result<(), Error> {
        match target {
            Parameter::Address(addr) => {
                if addr >= self.inst.len() {
                    self.inst.resize_with(addr+1, Default::default);
                }
                self.inst[addr] = value;
                Ok(())
            },
            Parameter::Value(_value) => {
                Err(format_err!(
                    "Cannot store using immediate parameter {:?}", target))
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
            2 => Ok(ParameterMode::Relative),
            _ => Err(format_err!("Invalid parameter mode: {}", op % 10)),
        }
    }
}

impl Clone for IntcodeProgram {
    fn clone(&self) -> Self {
        Self {
            inst: self.inst.clone(),
            pc: self.pc,
            state: self.state.clone(),
            relative_base: self.relative_base,
            input_reader: None,
            input_sender: None,
            output_sender: None,
            last_output: 0,
            blocking_input: true,
            waiting_for_input: false,
        }
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

impl Parameter {
    fn to_address(address: isize) -> Result<Parameter, Error> {
        if address < 0 {
            Err(format_err!("Invalid address: {}", address))
        } else {
            Ok(Parameter::Address(address as usize))
        }
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
        let input_sender = program.create_input_channel();
        let output_receiver = program.create_output_channel();
        input_sender.send(input)?;
        program.run()?;
        Ok(output_receiver.recv()?)
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
        let input_sender = program.create_input_channel();
        input_sender.send(5).unwrap();
        assert_eq!(5, program.run().unwrap());
    }

    #[test]
    fn test_output() {
        let mut program = IntcodeProgram::from_string("4,0,99").unwrap();
        let output_receiver = program.create_output_channel();
        assert!(program.run().is_ok());
        assert_eq!(4, output_receiver.recv().unwrap());
    }

    #[test]
    fn test_quine() {
        // Example from Day 9, a program that outputs itself.
        let quine: Vec<isize> = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101,
            1006, 101, 0, 99
        ];
        let mut program =
            IntcodeProgram::from_string(&quine.iter()
                                              .map(|v| v.to_string())
                                              .collect::<Vec<String>>()
                                              .join(",")).unwrap();
        let reader = program.create_output_channel();
        match program.run() {
            Ok(_) => {},
            Err(e) => { assert!(false, e.to_string()) },
        }
        assert!(program.run().is_ok());
        for &v in quine.iter() {
            assert_eq!(v, reader.recv().unwrap());
        }
    }

    #[test]
    fn test_large_multiply() {
        // Example from day 9
        let mut program = IntcodeProgram::from_string(
            "1102,34915192,34915192,7,4,7,99,0").unwrap();
        let reader = program.create_output_channel();
        assert!(program.run().is_ok());
        assert_eq!(1219070632396864, reader.recv().unwrap());
    }

    #[test]
    fn test_large_value() {
        // Example from day 9
        let mut program = IntcodeProgram::from_string(
            "104,1125899906842624,99").unwrap();
        let reader = program.create_output_channel();
        assert!(program.run().is_ok());
        assert_eq!(1125899906842624, reader.recv().unwrap());
    }

    #[test]
    fn test_storing_to_immediate_param_fails() {
        let mut program = IntcodeProgram::from_string("11101,1,1,1,99")
                                         .unwrap();
        let result = program.run();
        assert!(result.is_err());
        assert_eq!("Cannot store using immediate parameter Value(1)",
                   result.unwrap_err().to_string());
    }
}

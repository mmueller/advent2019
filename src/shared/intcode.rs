use anyhow::{Error, format_err};
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;

type Receiver = mpsc::Receiver<isize>;
type Sender = mpsc::Sender<isize>;

pub struct Program {
    instructions: Vec<isize>,
}

pub struct Simulator {
    state: ProgramState,
    mem: Vec<isize>,
    pc: usize,
    relative_base: usize,
    io: SimulatorIO,
}

struct SimulatorIO {
    input_reader: Option<Receiver>,
    input_sender: Option<Sender>,
    output_sender: Option<Sender>,
    last_output: isize,
    blocking_input: bool,
}

#[derive(Clone,PartialEq)]
enum ProgramState {
    Running,
    Wait,
    Halted,
}

#[derive(Clone,Debug,PartialEq)]
enum Parameter {
    Address(usize),
    Value(isize),
}

#[derive(Clone,PartialEq)]
enum ParameterMode {
    Position,
    Immediate,
    Relative,
}

impl Program {
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
            instructions: inst,
        })
    }
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            state: ProgramState::Halted,
            mem: Vec::new(),
            pc: 0,
            relative_base: 0,
            io: SimulatorIO::default(),
        }
    }

    // Construct Simulator with the given program loaded, for convenience.
    pub fn with_program(program: &Program) -> Self {
        let mut sim = Self::new();
        sim.load_program(program);
        sim
    }

    // Load the given program into memory. Implicitly resets all state to new
    // (I/O handlers are untouched).
    pub fn load_program(&mut self, program: &Program) {
        self.state = ProgramState::Halted;
        self.mem = program.instructions.clone();
        self.pc = 0;
        self.relative_base = 0;
    }

    // Creates a channel for you and returns the sender half.
    pub fn create_input_channel(&mut self) -> Sender {
        let (sender, receiver) = mpsc::channel();
        self.io.input_reader = Some(receiver);
        self.io.input_sender = Some(sender.clone());
        sender
    }

    pub fn connect_input(&mut self, input: Receiver, sender: Sender) {
        self.io.input_reader = Some(input);
        self.io.input_sender = Some(sender);
    }

    // TODO: Can we remove
    pub fn send_input(&self, value: isize) -> Result<(), Error> {
        match &self.io.input_sender {
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
        self.io.input_reader = None;
        self.io.input_sender = None;
    }

    // Creates a channel for you and returns the receiver half.
    pub fn create_output_channel(&mut self) -> Receiver {
        let (sender, receiver) = mpsc::channel();
        self.io.output_sender = Some(sender);
        receiver
    }

    pub fn connect_output(&mut self, output: Sender) {
        self.io.output_sender = Some(output);
    }

    #[allow(dead_code)]
    pub fn disconnect_output(&mut self) {
        self.io.output_sender = None;
    }

    // Feels hacky, but it's useful.
    pub fn last_output(&self) -> isize {
        self.io.last_output
    }

    // Return the value in memory at the given address.
    pub fn peek(&self, address: usize) -> isize {
        self.load(Parameter::Address(address))
    }

    // Overwrite memory at the given address.
    pub fn poke(&mut self, address: usize, value: isize) {
        self.store(Parameter::Address(address), value).ok();
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.state = ProgramState::Running;
        while self.is_running() {
            self.step()?;
            if self.waiting_for_input() {
                break;
            }
        }
        Ok(())
    }

    // True if running (even if blocked on input)
    pub fn is_running(&self) -> bool {
        self.state != ProgramState::Halted
    }

    pub fn set_blocking_input(&mut self, blocking: bool) {
        self.io.blocking_input = blocking;
    }

    pub fn waiting_for_input(&self) -> bool {
        self.state == ProgramState::Wait
    }

    pub fn step(&mut self) -> Result<(), Error> {
        let op = self.mem[self.pc];
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
                match self.io.input_reader {
                    Some(ref receiver) => {
                        if self.io.blocking_input {
                            let value = receiver.recv()?;
                            self.store(target, value)?;
                        } else {
                            match receiver.try_recv() {
                                Ok(value) => {
                                    self.store(target, value)?;
                                    self.state = ProgramState::Running;
                                },
                                Err(_) => {
                                    self.state = ProgramState::Wait;
                                }
                            }
                        }
                    },
                    None => {
                        return Err(format_err!(
                            "Input op called without input channel."));
                    },
                }
                if self.state != ProgramState::Wait {
                    self.pc += 2;
                }
            },
            // Output
            4 => {
                let value = self.load(self.get_param(0)?);
                match self.io.output_sender {
                    Some(ref sender) => {
                        sender.send(value)?;
                        self.io.last_output = value;
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
                self.state = ProgramState::Halted;
            },
            _ => {
                return Err(format_err!("Invalid opcode: {} (pc: {})",
                                       self.mem[self.pc], self.pc));
            },
        }
        Ok(())
    }

    // Private

    // Get the ith parameter (0-based) for the current instruction.
    fn get_param(&self, i: usize) -> Result<Parameter, Error> {
        let mode = Self::parameter_mode(self.mem[self.pc], i)?;
        let raw_value = self.mem[self.pc + i + 1];
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
                if addr >= self.mem.len() {
                    0
                } else {
                    self.mem[addr]
                }
            },
            Parameter::Value(value) => value,
        }
    }

    // Target must be an address
    fn store(&mut self, target: Parameter, value: isize) -> Result<(), Error> {
        match target {
            Parameter::Address(addr) => {
                if addr >= self.mem.len() {
                    self.mem.resize_with(addr+1, Default::default);
                }
                self.mem[addr] = value;
                Ok(())
            },
            Parameter::Value(_value) => {
                Err(format_err!(
                    "Cannot store using immediate parameter {:?}", target))
            },
        }
    }

    fn parameter_mode(op: isize, param: usize) -> Result<ParameterMode, Error> {
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

impl SimulatorIO {
    fn default() -> Self {
        Self {
            input_reader: None,
            input_sender: None,
            output_sender: None,
            last_output: 0,
            blocking_input: false,
        }
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

    // Helper to run a program given in the string. Returns the simulator in
    // case you want to inspect any state.
    fn run_program(text: &str) -> Result<Simulator, Error> {
        let program = Program::from_string(text).unwrap();
        let mut sim = Simulator::with_program(&program);
        sim.run()?;
        Ok(sim)
    }

    // Helper to run a program with a particular input value and return the
    // output value it yields.
    fn run_program_io(text: &str, input: isize) -> Result<isize, Error> {
        let program = Program::from_string(text).unwrap();
        let mut sim = Simulator::with_program(&program);
        let input_sender = sim.create_input_channel();
        let output_receiver = sim.create_output_channel();
        input_sender.send(input)?;
        sim.run()?;
        Ok(output_receiver.recv()?)
    }

    #[test]
    fn test_empty_program() {
        let sim = run_program("99").unwrap();
        assert_eq!(99, sim.peek(0));
    }

    #[test]
    fn test_add() {
        // 1 + 1 = 2
        let sim = run_program("1,0,0,0,99").unwrap();
        assert_eq!(2, sim.peek(0));

        // 1 + 1 + 1 = 3
        let sim = run_program("1,0,0,1,1,0,1,0,99").unwrap();
        assert_eq!(3, sim.peek(0));
    }

    #[test]
    fn test_multiply() {
        // 2 + 3 = 6
        let sim = run_program("2,0,5,0,99,3").unwrap();
        assert_eq!(6, sim.peek(0));

        // 2 * 3 * 4 = 24
        let sim = run_program("2,0,9,1,2,10,1,0,99,3,4").unwrap();
        assert_eq!(24, sim.peek(0));
    }

    #[test]
    fn test_day2_examples() {
        let sim = run_program("1,9,10,3,2,3,11,0,99,30,40,50").unwrap();
        assert_eq!(3500, sim.peek(0));
        let sim = run_program("1,1,1,4,99,5,6,0,99").unwrap();
        assert_eq!(30, sim.peek(0));
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
        let sim = run_program("1,0,5,0,99,-2").unwrap();
        assert_eq!(-1, sim.peek(0));
    }

    #[test]
    fn test_immediate_mode() {
        let sim = run_program("1001,0,4,0,99").unwrap();
        assert_eq!(1005, sim.peek(0));
    }

    #[test]
    fn test_input() {
        let program = Program::from_string("3,0,99").unwrap();
        let mut sim = Simulator::with_program(&program);
        let input_sender = sim.create_input_channel();
        input_sender.send(5).unwrap();
        assert!(sim.run().is_ok());
        assert_eq!(5, sim.peek(0));
    }

    #[test]
    fn test_output() {
        let program = Program::from_string("4,0,99").unwrap();
        let mut sim = Simulator::with_program(&program);
        let output_receiver = sim.create_output_channel();
        assert!(sim.run().is_ok());
        assert_eq!(4, output_receiver.recv().unwrap());
    }

    #[test]
    fn test_quine() {
        // Example from Day 9, a program that outputs itself.
        let quine: Vec<isize> = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101,
            1006, 101, 0, 99
        ];
        let program = Program::from_string(&quine.iter()
                              .map(|v| v.to_string())
                              .collect::<Vec<String>>()
                              .join(",")).unwrap();
        let mut sim = Simulator::with_program(&program);
        let reader = sim.create_output_channel();
        assert!(sim.run().is_ok());
        for &v in quine.iter() {
            assert_eq!(v, reader.recv().unwrap());
        }
    }

    #[test]
    fn test_large_multiply() {
        // Example from day 9
        let program = Program::from_string(
            "1102,34915192,34915192,7,4,7,99,0").unwrap();
        let mut sim = Simulator::with_program(&program);
        let reader = sim.create_output_channel();
        assert!(sim.run().is_ok());
        assert_eq!(1219070632396864, reader.recv().unwrap());
    }

    #[test]
    fn test_large_value() {
        // Example from day 9
        let program = Program::from_string("104,1125899906842624,99").unwrap();
        let mut sim = Simulator::with_program(&program);
        let reader = sim.create_output_channel();
        assert!(sim.run().is_ok());
        assert_eq!(1125899906842624, reader.recv().unwrap());
    }

    #[test]
    fn test_storing_to_immediate_param_fails() {
        let program = Program::from_string("11101,1,1,1,99").unwrap();
        let mut sim = Simulator::with_program(&program);
        let result = sim.run();
        assert!(result.is_err());
        assert_eq!("Cannot store using immediate parameter Value(1)",
                   result.unwrap_err().to_string());
    }
}

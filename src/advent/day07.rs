use crate::advent::AdventSolver;
use crate::shared::intcode::IntcodeProgram;
use anyhow::Error;
use permutohedron::LexicalPermutation;
use std::sync::mpsc::channel;

#[derive(Default)]
pub struct Solver;

const NUM_AMPLIFIERS: isize = 5;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let program = IntcodeProgram::from_path("input/day07.txt")?;
        let mut phases: Vec<isize> = (0..NUM_AMPLIFIERS).collect();
        let mut max_signal: isize = 0;
        loop {
            let signal = Self::run_amplifier_config(&program, &phases)?;
            if signal > max_signal {
                max_signal = signal;
            }
            if !phases.next_permutation() {
                break;
            }
        }
        println!("Max signal from serial config: {}", max_signal);

        max_signal = 0;
        phases = (NUM_AMPLIFIERS..NUM_AMPLIFIERS*2).collect();
        loop {
            let signal = Self::run_feedback_loop(&program, &phases)?;
            if signal > max_signal {
                max_signal = signal;
            }
            if !phases.next_permutation() {
                break;
            }
        }
        println!("Max signal from feedback loop: {}", max_signal);

        Ok(())
    }
}

impl Solver {
    fn run_amplifier_config(program: &IntcodeProgram,
                            phases: &Vec<isize>) -> Result<isize, Error>
    {
        let mut signal = 0;
        for &phase in phases {
            signal = Self::run_single_amplifier(program, phase, signal)?;
        }
        Ok(signal)
    }

    fn run_feedback_loop(program: &IntcodeProgram,
                         phases: &Vec<isize>) -> Result<isize, Error>
    {
        let count = phases.len();
        let mut programs = (0..count).map(|_| {
                                         let mut p = program.clone();
                                         p.set_blocking_input(false);
                                         p
                                     })
                                     .collect::<Vec<IntcodeProgram>>();

        // Wire up the amplifiers in a series
        for i in 0..count-1 {
            let (sender, receiver) = channel();
            programs[i+1].connect_input(receiver, sender.clone());
            programs[i].connect_output(sender);
        }

        // Close the loop
        let (loop_sender, loop_receiver) = channel();
        programs[0].connect_input(loop_receiver, loop_sender.clone());
        programs[count-1].connect_output(loop_sender);

        // Initialize phase values
        for i in 0..count {
            programs[i].send_input(phases[i])?;
        }

        // Provide the input signal 0
        programs[0].send_input(0)?;

        // Run the programs
        let mut running = true;
        while running {
            running = false;
            for program in programs.iter_mut() {
                program.run_until_input_needed()?;
                // If any program is running, keep going.
                if program.is_running() {
                    running = true;
                }
            }
        }

        // Return the last output value
        Ok(programs[count-1].last_output())
    }

    fn run_single_amplifier(p: &IntcodeProgram,
                            phase: isize, signal: isize) -> Result<isize, Error>
    {
        let mut program: IntcodeProgram = p.clone();
        let input_sender = program.create_input_channel();
        let output_receiver = program.create_output_channel();
        input_sender.send(phase)?;
        input_sender.send(signal)?;
        program.run()?;
        Ok(output_receiver.recv()?)
    }
}

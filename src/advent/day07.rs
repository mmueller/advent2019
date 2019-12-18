use crate::advent::AdventSolver;
use crate::shared::intcode::{Program, Simulator};
use anyhow::Error;
use permutohedron::LexicalPermutation;
use std::sync::mpsc::channel;

#[derive(Default)]
pub struct Solver;

const NUM_AMPLIFIERS: isize = 5;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let program = Program::from_path("input/day07.txt")?;
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
    fn run_amplifier_config(program: &Program,
                            phases: &Vec<isize>) -> Result<isize, Error>
    {
        let mut signal = 0;
        for &phase in phases {
            signal = Self::run_single_amplifier(program, phase, signal)?;
        }
        Ok(signal)
    }

    fn run_feedback_loop(program: &Program,
                         phases: &Vec<isize>) -> Result<isize, Error>
    {
        let count = phases.len();
        let mut sims =
            (0..count).map(|_| {
                          let mut sim = Simulator::with_program(program);
                          sim.set_blocking_input(false);
                          sim
                      })
                      .collect::<Vec<Simulator>>();

        // Wire up the amplifiers in a series
        for i in 0..count-1 {
            let (sender, receiver) = channel();
            sims[i+1].connect_input(receiver, sender.clone());
            sims[i].connect_output(sender);
        }

        // Close the loop
        let (loop_sender, loop_receiver) = channel();
        sims[0].connect_input(loop_receiver, loop_sender.clone());
        sims[count-1].connect_output(loop_sender);

        // Initialize phase values
        for i in 0..count {
            sims[i].send_input(phases[i])?;
        }

        // Provide the input signal 0
        sims[0].send_input(0)?;

        // Run the programs
        loop {
            for sim in sims.iter_mut() {
                sim.run()?;
            }
            if !sims.iter().any(|sim| sim.is_running()) {
                break;
            }
        }

        // Return the last output value
        Ok(sims[count-1].last_output())
    }

    fn run_single_amplifier(program: &Program,
                            phase: isize, signal: isize) -> Result<isize, Error>
    {
        let mut sim = Simulator::with_program(program);
        let input_sender = sim.create_input_channel();
        let output_receiver = sim.create_output_channel();
        input_sender.send(phase)?;
        input_sender.send(signal)?;
        sim.run()?;
        Ok(output_receiver.recv()?)
    }
}

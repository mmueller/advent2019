use crate::advent::AdventSolver;
use anyhow::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Default)]
pub struct Solver;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let module_masses: Vec<u64> =
            BufReader::new(File::open("input/day01.txt")?)
                      .lines()
                      .collect::<Result<Vec<String>, _>>()?
                      .iter()
                      .map(|s| s.parse::<u64>())
                      .collect::<Result<Vec<u64>, _>>()?;

        // Part 1
        let fuel_for_modules_naive =
            module_masses.iter()
                .fold(0, |sum, &m| {
                    sum+Self::fuel_for_mass_naive(m)
                });
        println!("Fuel for all modules (naive): {}", fuel_for_modules_naive);

        // Part 2
        let fuel_for_modules_accounting_for_fuel_mass =
            module_masses.iter()
                         .fold(0, |sum, &m| {
                             sum+Self::fuel_for_mass_accounting_for_fuel_mass(m)
                         });
        println!("Fuel for all modules (accounting for fuel mass): {}",
                 fuel_for_modules_accounting_for_fuel_mass);
        Ok(())
    }
}

impl Solver {
    fn fuel_for_mass_naive(mass: u64) -> u64 {
        let fuel = mass / 3;
        if fuel > 2 {
            fuel - 2
        } else {
            0
        }
    }

    fn fuel_for_mass_accounting_for_fuel_mass(mass: u64) -> u64 {
        let fuel = Self::fuel_for_mass_naive(mass);
        if fuel > 0 {
            fuel + Self::fuel_for_mass_accounting_for_fuel_mass(fuel)
        } else {
            fuel
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Solver;

    #[test]
    fn test_fuel_for_mass_naive_examples() {
        assert_eq!(2, Solver::fuel_for_mass_naive(12));
        assert_eq!(2, Solver::fuel_for_mass_naive(14));
        assert_eq!(654, Solver::fuel_for_mass_naive(1969));
        assert_eq!(33583, Solver::fuel_for_mass_naive(100756));
    }

    #[test]
    fn test_fuel_for_mass_naive_edge_cases() {
        assert_eq!(0, Solver::fuel_for_mass_naive(0));
        assert_eq!(0, Solver::fuel_for_mass_naive(3));
        assert_eq!(0, Solver::fuel_for_mass_naive(5));
        assert_eq!(0, Solver::fuel_for_mass_naive(8));
        assert_eq!(1, Solver::fuel_for_mass_naive(9));
        assert_eq!(1, Solver::fuel_for_mass_naive(10));
    }

    #[test]
    fn test_fuel_for_mass_accounting_for_fuel_mass_examples() {
        assert_eq!(2, Solver::fuel_for_mass_accounting_for_fuel_mass(14));
        assert_eq!(966, Solver::fuel_for_mass_accounting_for_fuel_mass(1969));
        assert_eq!(50346,
                   Solver::fuel_for_mass_accounting_for_fuel_mass(100756));
    }
}

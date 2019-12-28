use crate::advent::AdventSolver;
use anyhow::{Error, format_err};
use lazy_static::lazy_static;
use num_integer::Integer;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Default)]
pub struct Solver {
    reactions: Vec<Reaction>,
    chems_on_hand: HashMap<String, u64>,
}

#[derive(Clone,Debug)]
struct Reaction {
    inputs: Vec<(u64, String)>,
    output: (u64, String),
}

lazy_static! {
    static ref REACTION_REGEX: Regex =
        Regex::new(r"(?P<inputs>.*) => (?P<output>.*)").unwrap();
        
    // Matches amount & name, like 7 BCNQ
    static ref CHEM_REGEX: Regex =
        Regex::new(r"(?P<amt>\d+) (?P<chem>[A-Z]+)").unwrap();
}

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        self.reactions =
            Solver::read_reactions_from_path("input/day14.txt")?;
        println!("ORE required for 1 FUEL: {:?}",
                 self.ore_to_produce(1, "FUEL"));

        // Leeroy Jenkinsssssssssssssss!!!!
        let supply = 1_000_000_000_000;
        let mut min = 1;
        let mut max = supply; // Unlikely to get more than 1 FUEL per ORE?
        loop {
            if min >= max {
                break;
            }
            let amount = (max+min)/2+1;
            self.chems_on_hand.clear();
            let ore = self.ore_to_produce(amount, "FUEL");
            if ore > supply {
                max = amount-1;
            } else {
                min = amount;
            }
        }

        println!("Produced {} FUEL with 1 trillion ORE.", min);

        Ok(())
    }
}

impl Solver {
    // So much mutable state :(
    fn ore_to_produce(&mut self, amount: u64, chem: &str) -> u64 {
        if amount == 0 {
            return 0;
        }
        let reaction: Reaction =
            self.reactions.iter()
                          .find(|r| r.output.1 == chem)
                          .unwrap()
                          .clone();
        let times_to_run = amount.div_ceil(&reaction.output.0);
        let mut ore = 0;
        for input in reaction.inputs.iter() {
            let input_chem = input.1.as_ref();
            let mut input_needed = input.0 * times_to_run;
            input_needed -= self.use_chem(input_needed, input_chem);
            ore += match input_chem {
                "ORE" => input_needed,
                other => self.ore_to_produce(input_needed, other),
            }
        }
        let produced = reaction.output.0 * times_to_run;
        self.store_chem(produced - amount, chem);
        return ore;
    }

    fn store_chem(&mut self, amount: u64, chem: &str) {
        let current = self.chems_on_hand.entry(chem.to_string()).or_insert(0);
        *current += amount;
    }

    // Returns amount used, which might be less than the amount requested.
    // You may have to produce more chem to get the full amount.
    fn use_chem(&mut self, mut amount: u64, chem: &str) -> u64 {
        let current = self.chems_on_hand.entry(chem.to_string()).or_insert(0);
        if *current > amount {
            *current -= amount;
        } else {
            amount = *current;
            *current = 0;
        }
        amount
    }

    fn read_reactions_from_path(path: &str) -> Result<Vec<Reaction>, Error> {
        let mut text = String::new();
        File::open(path)?
             .read_to_string(&mut text)?;
        Self::read_reactions_from_string(&text)
    }

    fn read_reactions_from_string(text: &str) -> Result<Vec<Reaction>, Error> {
        let mut result = Vec::new();
        for line in text.split("\n") {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match REACTION_REGEX.captures(line) {
                Some(caps) => {
                    let mut inputs = Vec::new();
                    for input in caps["inputs"].split(", ") {
                        inputs.push(Self::parse_chem(input)?);
                    }
                    result.push(Reaction {
                        inputs: inputs,
                        output: Self::parse_chem(&caps["output"])?,
                    });
                },
                None => {
                    return Err(format_err!("Parse error: {}", line));
                }
            }
        }
        Ok(result)
    }

    fn parse_chem(s: &str) -> Result<(u64, String), Error> {
        match CHEM_REGEX.captures(s) {
            Some(caps) => Ok((caps["amt"].parse::<u64>()?,
                              caps["chem"].to_string())),
            None => Err(format_err!("Parse error: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_solver(input: &str) -> Solver {
        let mut solver = Solver::default();
        solver.reactions = Solver::read_reactions_from_string(input).unwrap();
        solver
    }

    #[test]
    fn test_example_1() {
        let text = r"
            10 ORE => 10 A
            1 ORE => 1 B
            7 A, 1 B => 1 C
            7 A, 1 C => 1 D
            7 A, 1 D => 1 E
            7 A, 1 E => 1 FUEL
        ";

        let mut solver = build_solver(text);
        assert_eq!(31, solver.ore_to_produce(1, "FUEL"));
    }

    #[test]
    fn test_example_2() {
        let text = r"
            9 ORE => 2 A
            8 ORE => 3 B
            7 ORE => 5 C
            3 A, 4 B => 1 AB
            5 B, 7 C => 1 BC
            4 C, 1 A => 1 CA
            2 AB, 3 BC, 4 CA => 1 FUEL
        ";

        let mut solver = build_solver(text);
        assert_eq!(165, solver.ore_to_produce(1, "FUEL"));
    }

    #[test]
    fn test_example_3() {
        let text = r"
            157 ORE => 5 NZVS
            165 ORE => 6 DCFZ
            44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
            12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
            179 ORE => 7 PSHF
            177 ORE => 5 HKGWZ
            7 DCFZ, 7 PSHF => 2 XJWVT
            165 ORE => 2 GPVTF
            3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
        ";

        let mut solver = build_solver(text);
        assert_eq!(13312, solver.ore_to_produce(1, "FUEL"));
    }

    #[test]
    fn test_example_4() {
        let text = r"
            2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
            17 NVRVD, 3 JNWZP => 8 VPVL
            53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
            22 VJHF, 37 MNCFX => 5 FWMGM
            139 ORE => 4 NVRVD
            144 ORE => 7 JNWZP
            5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
            5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
            145 ORE => 6 MNCFX
            1 NVRVD => 8 CXFTF
            1 VJHF, 6 MNCFX => 4 RFSQX
            176 ORE => 6 VJHF
        ";

        let mut solver = build_solver(text);
        assert_eq!(180697, solver.ore_to_produce(1, "FUEL"));
    }

    #[test]
    fn test_example_5() {
        let text = r"
          171 ORE => 8 CNZTR
          7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
          114 ORE => 4 BHXH
          14 VRPVC => 6 BMBT
          6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
          6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
          15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
          13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
          5 BMBT => 4 WPTQ
          189 ORE => 9 KTJDG
          1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
          12 VRPVC, 27 CNZTR => 2 XDBXC
          15 KTJDG, 12 BHXH => 5 XCVML
          3 BHXH, 2 VRPVC => 7 MZWV
          121 ORE => 7 VRPVC
          7 XCVML => 6 RJRHP
          5 BHXH, 4 VRPVC => 5 LTCX
        ";

        let mut solver = build_solver(text);
        assert_eq!(2210736, solver.ore_to_produce(1, "FUEL"));
    }
}

use crate::advent::AdventSolver;
use anyhow::Error;

#[derive(Default)]
pub struct Solver;

// My personal input
const MIN_PASSWORD: u32 = 123257;
const MAX_PASSWORD: u32 = 647015;

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let p1_filters = [
            Self::digits_ascending,
            Self::two_adjacent_digits_same,
        ];
        let p2_filters: Vec<Box<dyn Fn(u32)->bool>> = vec![
            Box::new(Self::digits_ascending),
            Box::new(|p| Self::exactly_two_adjacent_digits_same(p, None)),
        ];

        println!("Part 1: Possible passwords: {}",
                 (MIN_PASSWORD..MAX_PASSWORD).filter(|&p| {
                     p1_filters.iter().all(|f| f(p))
                 }).count());
        println!("Part 2: Possible passwords: {}",
                 (MIN_PASSWORD..MAX_PASSWORD).filter(|&p| {
                     p2_filters.iter().all(|f| f(p))
                 }).count());
        Ok(())
    }
}

impl Solver {
    fn digits_ascending(password: u32) -> bool {
        if password < 10 {
            true
        } else {
            let shifted = password / 10;
            password % 10 >= shifted % 10 && Self::digits_ascending(shifted)
        }
    }

    fn two_adjacent_digits_same(password: u32) -> bool {
        if password < 10 {
            false
        } else {
            let shifted = password / 10;
            password % 10 == shifted % 10
                || Self::two_adjacent_digits_same(shifted)
        }
    }

    fn exactly_two_adjacent_digits_same(password: u32,
                                        ignore: Option<u32>) -> bool {
        let digit1 = password % 10;
        let digit2 = (password / 10) % 10;
        let digit3 = (password / 100) % 10;
        if password < 10 {
            false
        } else if password < 100 {
            digit1 == digit2 && ignore != Some(digit1)
        } else {
            ignore != Some(digit1) && digit1 == digit2 && digit2 != digit3
                || Self::exactly_two_adjacent_digits_same(password/10,
                                                          Some(digit1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digits_ascending() {
        assert!(Solver::digits_ascending(1));
        assert!(Solver::digits_ascending(22));
        assert!(Solver::digits_ascending(333));
        assert!(Solver::digits_ascending(144589));
        assert!(Solver::digits_ascending(999999));

        assert!(!Solver::digits_ascending(21));
        assert!(!Solver::digits_ascending(112230));
        assert!(!Solver::digits_ascending(654321));
    }

    #[test]
    fn test_two_adjacent_digits_same() {
        assert!(Solver::two_adjacent_digits_same(66));
        assert!(Solver::two_adjacent_digits_same(100001));
        assert!(Solver::two_adjacent_digits_same(111111));
        assert!(Solver::two_adjacent_digits_same(112233));
        assert!(Solver::two_adjacent_digits_same(123455));
        assert!(Solver::two_adjacent_digits_same(223456));
        assert!(Solver::two_adjacent_digits_same(988754));

        assert!(!Solver::two_adjacent_digits_same(123456));
        assert!(!Solver::two_adjacent_digits_same(821952));
    }

    #[test]
    fn test_exactly_two_adjacent_digits_same() {
        assert!(Solver::exactly_two_adjacent_digits_same(66, None));
        assert!(Solver::exactly_two_adjacent_digits_same(112233, None));
        assert!(Solver::exactly_two_adjacent_digits_same(123455, None));
        assert!(Solver::exactly_two_adjacent_digits_same(223456, None));
        assert!(Solver::exactly_two_adjacent_digits_same(988754, None));

        assert!(!Solver::exactly_two_adjacent_digits_same(100001, None));
        assert!(!Solver::exactly_two_adjacent_digits_same(111111, None));
        assert!(!Solver::exactly_two_adjacent_digits_same(123456, None));
        assert!(!Solver::exactly_two_adjacent_digits_same(821952, None));
    }
}

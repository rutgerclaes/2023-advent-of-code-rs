use std::{collections::HashMap, fmt::Display, num::ParseIntError, str::FromStr};

use itertools::Itertools;
use utils::{
    io::{input::parse_input_lines, output::*},
    result::SolutionError,
};

fn main() {
    setup_logging();
    let input: Vec<Triangle> = parse_input_lines().expect("Could not parse input lines");

    let part_one = part_one(&input);
    show_result_part_one(part_one);

    let part_two = part_two(&input);
    show_result_part_two(part_two);
}

fn part_one(input: &[Triangle]) -> Result<i64, SolutionError> {
    input
        .iter()
        .map(|t| t.next())
        .fold_ok(0, |a, b| a + b as i64)
}

fn part_two(input: &[Triangle]) -> Result<i64, SolutionError> {
    input
        .iter()
        .map(|t| t.prev())
        .fold_ok(0, |a, b| a + b as i64)
}

#[derive(Debug)]
struct Triangle {
    max_x: i32,
    min_x: i32,
    max_y: usize,
    values: HashMap<(usize, i32), i32>,
}

impl Triangle {
    fn from<I>(input: I) -> Triangle
    where
        I: IntoIterator<Item = i32>,
    {
        fn extend(
            y: usize,
            row: Vec<i32>,
            mut values: HashMap<(usize, i32), i32>,
        ) -> (usize, HashMap<(usize, i32), i32>) {
            let next_row: Vec<_> = row.iter().tuple_windows().map(|(a, b)| b - a).collect();
            values.extend(row.iter().enumerate().map(|(x, i)| ((y, x as i32), *i)));

            if next_row.iter().all(|a| a == &0) {
                values.extend(
                    next_row
                        .iter()
                        .enumerate()
                        .map(|(x, i)| ((y + 1, x as i32), *i)),
                );
                (y + 1, values)
            } else {
                extend(y + 1, next_row, values)
            }
        }

        let initial_row: Vec<_> = input.into_iter().collect();
        let max_x = initial_row.len() as i32 - 1;
        let values = HashMap::new();
        let (max_y, values) = extend(0, initial_row, values);

        Triangle {
            max_x,
            min_x: 0,
            max_y,
            values,
        }
    }

    fn next(&self) -> Result<i32, SolutionError> {
        (0..self.max_y)
            .map(|dy| {
                let y = self.max_y - dy - 1;
                let x = self.max_x - y as i32;
                self.values
                    .get(&(y, x))
                    .ok_or(SolutionError::NoSolutionFound)
            })
            .fold_ok(0, |a, b| a + b)
    }

    fn prev(&self) -> Result<i32, SolutionError> {
        (0..self.max_y)
            .map(|dy| {
                let y = self.max_y - dy - 1;
                self.values
                    .get(&(y, 0))
                    .ok_or(SolutionError::NoSolutionFound)
            })
            .fold_ok(0, |a, b| b - a)
    }
}

impl Display for Triangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = (0..=self.max_y)
            .map(|y| {
                (self.min_x..=self.max_x)
                    .map(|x| {
                        self.values
                            .get(&(y, x))
                            .map_or(String::from("   "), |i| format!("{:>3}", i))
                    })
                    .join(" ")
            })
            .join("\n");
        write!(f, "{}", output)
    }
}

impl FromStr for Triangle {
    type Err = SolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers: Result<Vec<i32>, ParseIntError> =
            s.split_ascii_whitespace().map(|s| s.parse()).try_collect();
        Ok(Triangle::from(numbers?))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_triangle_filling() {
        let triangle: Triangle = "10 13 16 21 30 45"
            .parse()
            .expect("Parsing the input failed");

        assert_eq!(triangle.max_y, 4);
        assert_eq!(triangle.min_x, 0);
        assert_eq!(triangle.max_x, 5);

        assert_eq!(triangle.values.get(&(0, 0)), Some(&10));
        assert_eq!(triangle.values.get(&(1, 0)), Some(&3));
        assert_eq!(triangle.values.get(&(2, 0)), Some(&0));
        assert_eq!(triangle.values.get(&(3, 0)), Some(&2));
        assert_eq!(triangle.values.get(&(4, 0)), Some(&0));

        assert_eq!(triangle.values.get(&(0, 5)), Some(&45));
        assert_eq!(triangle.values.get(&(1, 4)), Some(&15));
        assert_eq!(triangle.values.get(&(2, 3)), Some(&6));
        assert_eq!(triangle.values.get(&(3, 2)), Some(&2));
        assert_eq!(triangle.values.get(&(4, 1)), Some(&0));
    }

    #[test]
    fn test_triangle_extrapolation() {
        let triangle: Triangle = "0 3 6 9 12 15".parse().expect("Parsing the input failed");
        assert_eq!(
            triangle.next().expect("Next value could not be calculated"),
            18
        );

        let triangle: Triangle = "1 3 6 10 15 21".parse().expect("Parsing the input failed");
        assert_eq!(
            triangle.next().expect("Next value could not be calculated"),
            28
        );

        let triangle: Triangle = "10 13 16 21 30 45"
            .parse()
            .expect("Parsing the input failed");
        assert_eq!(
            triangle.next().expect("Next value could not be calculated"),
            68
        );
    }

    #[test]
    fn test_triangle_backwards_extrapolation() {
        let triangle: Triangle = "0 3 6 9 12 15".parse().expect("Parsing the input failed");
        assert_eq!(
            triangle.prev().expect("Next value could not be calculated"),
            -3
        );

        let triangle: Triangle = "1 3 6 10 15 21".parse().expect("Parsing the input failed");
        assert_eq!(
            triangle.prev().expect("Next value could not be calculated"),
            0
        );

        let triangle: Triangle = "10 13 16 21 30 45"
            .parse()
            .expect("Parsing the input failed");
        assert_eq!(
            triangle.prev().expect("Next value could not be calculated"),
            5
        );
    }
}

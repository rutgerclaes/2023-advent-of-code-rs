use anyhow::Result;
use itertools::Itertools;
use std::io::BufRead;
use std::iter::Sum;
use utils::io::input::read_input;
use utils::io::output::{setup_logging, show_part_one, show_part_two};

fn main() -> Result<()> {
    setup_logging();
    let input: Vec<String> = read_input()?.lines().try_collect()?;

    let part_one = part_one(&input);
    show_part_one(part_one);

    let part_two = part_two(&input);
    show_part_two(part_two);
    Ok(())
}

fn part_one(input: &[String]) -> u32 {
    solve(input, parse_line)
}

fn part_two(input: &[String]) -> u32 {
    solve(input, parse_line_with_words)
}

fn solve<F, R>(input: &[String], m: F) -> R
where
    F: Fn(&str) -> Option<R>,
    R: Sum<R>,
{
    input.iter().filter_map(|l| m(l)).sum()
}

#[tracing::instrument(level = "trace", ret())]
fn parse_line(input: &str) -> Option<u32> {
    input
        .chars()
        .filter_map(|c| c.to_digit(10))
        .fold(None, |e, d| match e {
            Some((start, _)) => Some((start, d)),
            None => Some((d, d)),
        })
        .map(|(a, b)| a * 10 + b)
}

const NUMBERS: [&str; 9] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
];

#[tracing::instrument(level = "trace", ret())]
fn parse_line_with_words(input: &str) -> Option<u32> {
    fn inner(input: &str, start: Option<u32>, end: Option<u32>) -> Option<u32> {
        if input.is_empty() {
            start.zip(end).map(|(a, b)| a * 10 + b)
        } else {
            let digit: Option<u32> =
                input
                    .chars()
                    .next()
                    .and_then(|c| c.to_digit(10))
                    .or_else(|| {
                        NUMBERS
                            .iter()
                            .find_position(|&word| input.starts_with(word))
                            .map(|(i, _)| i as u32 + 1)
                    });

            inner(&input[1..], start.or(digit), digit.or(end))
        }
    }

    inner(input, None, None)
}

#[cfg(test)]
mod test {
    use crate::*;
    use itertools::Itertools;

    #[test]
    fn test_parse_lines() {
        assert_eq!(12, parse_line("1abc2").expect("line contains no number"));
        assert_eq!(
            38,
            parse_line("pqr3stu8vwx").expect("line contains no number")
        );
        assert_eq!(
            15,
            parse_line("a1b2c3d4e5f").expect("line contains no number")
        );
        assert_eq!(
            77,
            parse_line("treb7uchet").expect("line contains no number")
        );

        assert_eq!(None, parse_line("foobar"));
    }

    #[test]
    fn test_part_one() {
        let input = "1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet"
            .lines()
            .map(|s| s.to_owned())
            .collect_vec();

        assert_eq!(142, part_one(&input))
    }

    #[test]
    fn test_parse_lines_with_words() {
        assert_eq!(
            29,
            parse_line_with_words("two1nine").expect("line contains no number")
        );
        assert_eq!(
            83,
            parse_line_with_words("eightwothree").expect("line contains no number")
        );
        assert_eq!(
            13,
            parse_line_with_words("abcone2threexyz").expect("line contains no number")
        );
        assert_eq!(
            24,
            parse_line_with_words("xtwone3four").expect("line contains no number")
        );
        assert_eq!(
            42,
            parse_line_with_words("4nineeightseven2").expect("line contains no number")
        );
        assert_eq!(
            14,
            parse_line_with_words("zoneight234").expect("line contains no number")
        );
        assert_eq!(
            76,
            parse_line_with_words("7pqrstsixteen").expect("line contains no number")
        );
    }

    #[test]
    fn test_part_two() {
        let input = "two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen"
            .lines()
            .map(|s| s.to_owned())
            .collect_vec();

        assert_eq!(281, part_two(&input))
    }
}

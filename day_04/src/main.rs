use im::HashMap;
use std::{collections::HashSet, fmt::Display, str::FromStr};

use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use utils::prelude::*;

fn main() {
    let cards: Vec<Card> = parse_input_lines().expect("Input could not be read");

    let part_one: u32 = part_one(&cards);
    show_part_one(part_one);

    let part_two = part_two(&cards);
    show_part_two(part_two);
}

#[tracing::instrument(level = "info", ret(), skip_all)]
fn part_one(cards: &[Card]) -> u32 {
    cards.iter().map(|c| c.score()).sum()
}

#[tracing::instrument(level = "info", ret(), skip_all)]
fn part_two(cards: &[Card]) -> u32 {
    let max_index = cards.last().map(|c| c.index).unwrap_or(0);
    let (card_count, _) = cards
        .iter()
        .fold((0, HashMap::new()), |(total_count, copies), card| {
            let matching_number_count = card.matching_numbers_count();
            let current_card_count = *copies.get(&card.index).unwrap_or(&1);

            let copies = (1..=matching_number_count)
                .map(|i| i + card.index)
                .filter(|i| i <= &max_index)
                .fold(copies, |copies, update_index| {
                    copies.alter(
                        |value| Some(value.unwrap_or(1) + current_card_count),
                        update_index,
                    )
                });

            (total_count + current_card_count, copies)
        });

    card_count
}

#[derive(Debug)]
struct Card {
    index: usize,
    winning_numbers: HashSet<u32>,
    picked_numbers: Vec<u32>,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wn = self
            .winning_numbers
            .iter()
            .map(|n| format!("{:2}", n))
            .join(" ");
        let pn = self
            .picked_numbers
            .iter()
            .map(|n| format!("{:2}", n))
            .join(" ");
        write!(f, "Card {:3}: {} | {}", self.index, wn, pn)
    }
}

impl Card {
    #[tracing::instrument(level = "trace", ret())]
    fn score(&self) -> u32 {
        self.picked_numbers
            .iter()
            .fold(None, |sum, number| {
                if self.winning_numbers.contains(number) {
                    Some(sum.map(|s| s * 2).unwrap_or(1))
                } else {
                    sum
                }
            })
            .unwrap_or(0)
    }

    #[tracing::instrument(level = "trace", ret())]
    fn matching_numbers_count(&self) -> usize {
        self.picked_numbers
            .iter()
            .filter(|n| self.winning_numbers.contains(n))
            .count()
    }
}

impl FromStr for Card {
    type Err = SolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^Card\s+(?<index>\d+): (?<winning>[\d\s]+) \| (?<picked>[\d\s]+)$")
                .unwrap()
        });

        let m = capture_regex(&RE, s)?;
        let index: usize = named_match(&m, "index")?.parse()?;

        let winning = named_match(&m, "winning")?;
        let picked = named_match(&m, "picked")?;

        let winning_numbers = winning
            .split_ascii_whitespace()
            .map(|d| d.parse())
            .try_collect()?;
        let picked_numbers = picked
            .split_ascii_whitespace()
            .map(|d| d.parse())
            .try_collect()?;

        Ok(Card {
            index,
            winning_numbers,
            picked_numbers,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_card_parsing() {
        let card: Card = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(1, card.index);
        assert_eq!(HashSet::from([41, 48, 83, 86, 17]), card.winning_numbers);
        assert_eq!(vec![83, 86, 6, 31, 17, 9, 48, 53], card.picked_numbers);

        let card: Card = "Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(2, card.index);
        assert_eq!(HashSet::from([13, 32, 20, 16, 61]), card.winning_numbers);
        assert_eq!(vec![61, 30, 68, 82, 17, 32, 24, 19], card.picked_numbers);
    }

    #[test]
    fn test_score_calculation() {
        let card: Card = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(8, card.score());
        let card: Card = "Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(2, card.score());
        let card: Card = "Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(2, card.score());
        let card: Card = "Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(1, card.score());
        let card: Card = "Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(0, card.score());
        let card: Card = "Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(0, card.score());
    }

    #[test]
    fn test_matching_numbers_count() {
        let card: Card = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(4, card.matching_numbers_count());
        let card: Card = "Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(2, card.matching_numbers_count());
        let card: Card = "Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(2, card.matching_numbers_count());
        let card: Card = "Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(1, card.matching_numbers_count());
        let card: Card = "Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(0, card.matching_numbers_count());
        let card: Card = "Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11"
            .parse()
            .expect("Parsing didn't work");
        assert_eq!(0, card.score());
    }
}

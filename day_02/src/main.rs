use std::{cmp::max, fmt::Display, str::FromStr};

use itertools::Itertools;
use regex::Regex;
use utils::prelude::*;

fn main() -> SolutionResult<()> {
    setup_logging();
    let games: Vec<Game> = parse_input_lines()?;

    let constraint = Cubes::new(12, 13, 14);
    let part_one = part_one(&games, &constraint);
    show_part_one(part_one);

    let part_two = part_two(&games);
    show_part_two(part_two);
    Ok(())
}

#[tracing::instrument(level = "info", ret(), skip(games))]
fn part_one(games: &[Game], constraint: &Cubes) -> u32 {
    games
        .iter()
        .filter(|g| g.fits_in(constraint))
        .map(|g| g.index as u32)
        .sum()
}

#[tracing::instrument(level = "info", ret(), skip_all)]
fn part_two(games: &[Game]) -> u128 {
    games.iter().map(|g| g.minimal_set().power() as u128).sum()
}

#[derive(Debug)]
struct Game {
    index: usize,
    grabs: Vec<Cubes>,
}

impl Game {
    fn fits_in(&self, constraint: &Cubes) -> bool {
        self.grabs.iter().all(|g| g.fits_in(constraint))
    }

    fn minimal_set(&self) -> Cubes {
        self.grabs
            .iter()
            .fold(Cubes::empty(), |minimal, cubes| minimal.union(cubes))
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let grabs_str = self.grabs.iter().map(|g| format!("{}", g)).join("; ");
        write!(f, "Game {}: {}", self.index, grabs_str)
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Cubes {
    red: usize,
    green: usize,
    blue: usize,
}

impl Display for Cubes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inputs = [
            ("red", self.red),
            ("green", self.green),
            ("blue", self.blue),
        ];

        let output = inputs
            .iter()
            .filter(|&&(_, c)| c > 0)
            .map(|(color, count)| format!("{} {}", count, color))
            .join(", ");
        write!(f, "{}", output)
    }
}

impl Cubes {
    fn new(red: usize, green: usize, blue: usize) -> Self {
        Self { red, green, blue }
    }

    fn fits_in(&self, other: &Cubes) -> bool {
        self.red <= other.red && self.green <= other.green && self.blue <= other.blue
    }

    fn empty() -> Self {
        Self::new(0, 0, 0)
    }

    fn with_red(self, red: usize) -> Self {
        Self { red, ..self }
    }

    fn with_green(self, green: usize) -> Self {
        Self { green, ..self }
    }

    fn with_blue(self, blue: usize) -> Self {
        Self { blue, ..self }
    }

    fn power(&self) -> usize {
        self.red * self.green * self.blue
    }

    fn union(&self, other: &Cubes) -> Cubes {
        Cubes::new(
            max(self.red, other.red),
            max(self.green, other.green),
            max(self.blue, other.blue),
        )
    }
}

impl FromStr for Cubes {
    type Err = SolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let colors: Vec<(&str, usize)> = s
            .split(",")
            .map(|e| e.trim())
            .filter(|e| !e.is_empty())
            .map(|string| {
                string
                    .split_ascii_whitespace()
                    .collect_tuple()
                    .ok_or_else(|| {
                        SolutionError::InputParsingFailed(format!("Could not parse '{}'", string))
                    })
                    .and_then(|(count, color)| {
                        if color == "red" || color == "green" || color == "blue" {
                            Ok((color, count.parse()?))
                        } else {
                            Err(SolutionError::InputParsingFailed(format!(
                                "Unknown color encountered: '{}'",
                                color
                            )))
                        }
                    })
            })
            .try_collect()?;

        Ok(colors
            .iter()
            .fold(Cubes::empty(), |cubes, &(color, count)| {
                if color == "red" {
                    cubes.with_red(count)
                } else if color == "green" {
                    cubes.with_green(count)
                } else {
                    cubes.with_blue(count)
                }
            }))
    }
}

impl FromStr for Game {
    type Err = SolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"^Game (?<index>\d+): (?<cubes>.+)$").unwrap();
        let captures = regex.captures(s).ok_or_else(|| {
            SolutionError::InputParsingFailed("Regular expression didn't match input".to_owned())
        })?;

        let index: usize = captures
            .name("index")
            .ok_or_else(|| {
                SolutionError::InputParsingFailed("Could not find 'index' match".to_owned())
            })?
            .as_str()
            .parse()?;
        let cubes: Vec<Cubes> = captures
            .name("cubes")
            .ok_or_else(|| {
                SolutionError::InputParsingFailed("Could not find 'cubes' match".to_owned())
            })?
            .as_str()
            .split("; ")
            .map(|cube_string| cube_string.parse())
            .try_collect()?;

        Ok(Game {
            index,
            grabs: cubes,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_cubes_fit_in() {
        assert_eq!(true, Cubes::new(0, 0, 0).fits_in(&Cubes::new(0, 0, 0)));
        assert_eq!(true, Cubes::new(0, 0, 0).fits_in(&Cubes::new(1, 1, 1)));
        assert_eq!(true, Cubes::new(1, 2, 3).fits_in(&Cubes::new(2, 3, 4)));

        assert_eq!(false, Cubes::new(1, 2, 3).fits_in(&Cubes::new(2, 2, 2)));
        assert_eq!(false, Cubes::new(2, 3, 1).fits_in(&Cubes::new(2, 2, 2)));
        assert_eq!(false, Cubes::new(3, 1, 2).fits_in(&Cubes::new(2, 2, 2)));
        assert_eq!(false, Cubes::new(3, 3, 3).fits_in(&Cubes::new(2, 2, 2)));
    }

    #[test]
    fn test_cubes_union() {
        assert_eq!(
            Cubes::new(1, 0, 1),
            Cubes::new(1, 0, 0).union(&Cubes::new(0, 0, 1))
        );
        assert_eq!(
            Cubes::new(1, 1, 0),
            Cubes::new(1, 0, 0).union(&Cubes::new(0, 1, 0))
        );
        assert_eq!(
            Cubes::new(2, 2, 2),
            Cubes::new(2, 1, 1).union(&Cubes::new(0, 2, 2))
        );
    }

    #[test]
    fn test_cubes_power() {
        assert_eq!(6, Cubes::new(1, 2, 3).power());
        assert_eq!(24, Cubes::new(2, 3, 4).power());
        assert_eq!(30, Cubes::new(2, 3, 5).power());

        assert_eq!(0, Cubes::new(0, 3, 5).power());
    }

    #[test]
    fn test_cube_construction() {
        let empty = Cubes::empty();
        assert_eq!(0, empty.red);
        assert_eq!(0, empty.green);
        assert_eq!(0, empty.blue);

        let non_empty = Cubes::new(0, 1, 2);
        assert_eq!(0, non_empty.red);
        assert_eq!(1, non_empty.green);
        assert_eq!(2, non_empty.blue);

        let with_red = empty.with_red(1);
        assert_eq!(Cubes::new(1, 0, 0), with_red);
        let with_green = with_red.with_green(2);
        assert_eq!(Cubes::new(1, 2, 0), with_green);
        let with_blue = with_green.with_blue(3);
        assert_eq!(Cubes::new(1, 2, 3), with_blue);
    }

    #[test]
    fn test_cubes_parsing() {
        assert_eq!(
            Cubes::new(1, 2, 3),
            "1 red, 2 green, 3 blue"
                .parse()
                .expect("Could not be parsed")
        );
        assert_eq!(
            Cubes::new(2, 3, 1),
            "3 green, 1 blue, 2 red"
                .parse()
                .expect("Could not be parsed")
        );
        assert_eq!(
            Cubes::new(0, 3, 1),
            "3 green, 1 blue".parse().expect("Could not be parsed")
        );
        assert_eq!(
            Cubes::new(0, 0, 1),
            "1 blue".parse().expect("Could not be parsed")
        );
        assert_eq!(
            Cubes::new(0, 0, 0),
            "".parse().expect("Could not be parsed")
        );
    }

    #[test]
    fn test_game_parsing() {
        let input = "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red";
        let game: Game = input.parse().expect("Could not parse game");

        assert_eq!(3, game.index);
        assert_eq!(
            vec![
                Cubes::new(20, 8, 6),
                Cubes::new(4, 13, 5),
                Cubes::new(1, 5, 0)
            ],
            game.grabs
        );
    }

    #[test]
    fn test_game_fits_in() {
        let constraint = Cubes::new(12, 13, 14);

        let fit: Game = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green"
            .parse()
            .expect("Game could not be parsed");
        let no_fit: Game =
            "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red"
                .parse()
                .expect("Game could not be parsed");

        assert_eq!(true, fit.fits_in(&constraint));
        assert_eq!(false, no_fit.fits_in(&constraint));
    }

    #[test]
    fn test_game_minimal_set() {
        let game_1: Game = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green"
            .parse()
            .expect("Game could not be parsed");
        let game_2: Game = "Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue"
            .parse()
            .expect("Game could not be parsed");

        assert_eq!(Cubes::new(4, 2, 6), game_1.minimal_set());
        assert_eq!(Cubes::new(1, 3, 4), game_2.minimal_set());
    }
}

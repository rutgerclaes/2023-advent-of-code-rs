use derive_more::{Deref, From, Into};
use std::{fmt::Display, marker::PhantomData, ops::Deref, str::FromStr};

use itertools::Itertools;
use utils::prelude::*;

fn main() {
    setup_logging();

    let input: Vec<String> = read_input_lines().expect("Could not read input");
    let (seeds, translation) = parse_input(&input).expect("Could not parse input");

    let part_one = part_one(&seeds, &translation);
    show_result_part_one(part_one);
}

fn part_one(
    seeds: &[Seed],
    translation: &TypedTranslation<Seed, Location>,
) -> SolutionResult<Location> {
    seeds
        .iter()
        .map(|x| translation.transform(x))
        .min()
        .ok_or_else(|| SolutionError::NoSolutionFound)
}

fn parse_input(lines: &[String]) -> SolutionResult<(Vec<Seed>, TypedTranslation<Seed, Location>)> {
    let mut iter = lines.iter();
    let seeds: Vec<Seed> = iter
        .next()
        .ok_or_else(|| SolutionError::InputParsingFailed(owned!("Input is empty")))?
        .strip_prefix("seeds: ")
        .ok_or_else(|| SolutionError::InputParsingFailed(owned!("Malformed first line")))?
        .split_ascii_whitespace()
        .map(|d| d.parse::<u32>().map(Seed::from))
        .try_collect()?;

    tracing::debug!("Parsed seeds: {}", seeds.iter().join(" "));

    let tables: Vec<Translation> = iter
        .skip(1)
        .batching(|i| {
            if let Some(title) = i.next() {
                let result: SolutionResult<Translation> = (|| {
                    tracing::debug!("Parsing table {}", title);
                    let rules: Vec<TranslationRule> = i
                        .take_while(|l| !l.is_empty())
                        .map(|line| line.parse())
                        .try_collect()?;

                    tracing::debug!("Done parsing table {}, found {} rules", title, rules.len());
                    Ok(Translation::Table(rules))
                })();

                Some(result)
            } else {
                None
            }
        })
        .try_collect()?;

    let (
        seed2soil,
        soil2fertilizer,
        fertilizer2water,
        water2light,
        light2temperature,
        temperature2humidity,
        humidity2location,
    ) = tables.into_iter().collect_tuple().ok_or_else(|| {
        SolutionError::InputParsingFailed(owned!("Incorrect number of translation tables"))
    })?;

    let translation = seed2soil
        .typed::<Seed, Soil>()
        .and_then(soil2fertilizer.typed::<Soil, Fertilizer>())
        .and_then(fertilizer2water.typed::<Fertilizer, Water>())
        .and_then(water2light.typed::<Water, Light>())
        .and_then(light2temperature.typed::<Light, Temperature>())
        .and_then(temperature2humidity.typed::<Temperature, Humidity>())
        .and_then(humidity2location.typed::<Humidity, Location>());

    Ok((seeds, translation))
}

#[derive(Deref, From, Into)]
struct Seed(u32);

impl Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deref, From)]
struct Soil(u32);
#[derive(Deref, From)]
struct Fertilizer(u32);
#[derive(Deref, From)]
struct Water(u32);
#[derive(Deref, From)]
struct Light(u32);
#[derive(Deref, From)]
struct Temperature(u32);
#[derive(Deref, From)]
struct Humidity(u32);
#[derive(Deref, From, PartialEq, PartialOrd, Eq, Ord)]
struct Location(u32);

enum Translation {
    Table(Vec<TranslationRule>),
    Chain(Box<Translation>, Box<Translation>),
}

struct TypedTranslation<I, O>
where
    I: Deref<Target = u32>,
    O: From<u32>,
{
    translation: Translation,
    input: PhantomData<I>,
    output: PhantomData<O>,
}

impl<I: Deref<Target = u32>, O: From<u32>> TypedTranslation<I, O> {
    fn transform(&self, input: &I) -> O {
        self.translation.translate(&input).into()
    }
}

impl<I: Deref<Target = u32>, O: Deref<Target = u32> + From<u32>> TypedTranslation<I, O> {
    fn and_then<N>(self, other: TypedTranslation<O, N>) -> TypedTranslation<I, N>
    where
        N: From<u32>,
    {
        TypedTranslation {
            translation: self.translation.and_then(other.translation),
            input: PhantomData,
            output: PhantomData,
        }
    }
}

struct TranslationRule {
    source_index: u32,
    destination_index: u32,
    length: u32,
}

impl TranslationRule {
    fn translate(&self, input: &u32) -> Option<u32> {
        if input >= &self.source_index && input < &(self.source_index + self.length) {
            Some(self.destination_index + (input - self.source_index))
        } else {
            None
        }
    }
}

impl FromStr for TranslationRule {
    type Err = SolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (destination_index, source_index, length) = s
            .split_ascii_whitespace()
            .map(|s| s.parse())
            .collect_tuple()
            .ok_or_else(|| {
                SolutionError::InputParsingFailed(format!("Could not parse rule: {}", s))
            })?;

        Ok(TranslationRule {
            destination_index: destination_index?,
            source_index: source_index?,
            length: length?,
        })
    }
}

impl Translation {
    fn translate(&self, input: &u32) -> u32 {
        match self {
            Self::Table(rules) => rules
                .iter()
                .find_map(|rule| rule.translate(input))
                .unwrap_or(*input),
            Self::Chain(a, b) => b.translate(&a.translate(input)),
        }
    }

    fn and_then(self, other: Translation) -> Translation {
        Translation::Chain(Box::new(self), Box::new(other))
    }

    fn typed<I, O>(self) -> TypedTranslation<I, O>
    where
        I: Deref<Target = u32>,
        O: From<u32>,
    {
        TypedTranslation {
            translation: self,
            input: PhantomData,
            output: PhantomData,
        }
    }
}

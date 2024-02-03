use derive_more::{Deref, From, Into};
use im::{vector, Vector};
use std::{
    cmp::{max, min},
    fmt::Display,
    marker::PhantomData,
    ops::Deref,
    str::FromStr,
};

use itertools::{FoldWhile, Itertools};
use utils::prelude::*;

fn main() {
    setup_logging();

    let input: Vec<String> = read_input_lines().expect("Could not read input");
    let (seeds, translation) = parse_input(&input).expect("Could not parse input");

    let part_one = part_one(&seeds, &translation);
    show_result_part_one(part_one);

    let part_two = part_two(seeds, translation);
    show_result_part_two(part_two);
}

fn part_one(
    seeds: &[Seed],
    translation: &TypedTranslation<Seed, Location>,
) -> SolutionResult<Location> {
    seeds
        .iter()
        .map(|x| translation.transform(x))
        .min()
        .ok_or(SolutionError::NoSolutionFound)
}

fn part_two(
    seeds: Vec<Seed>,
    translation: TypedTranslation<Seed, Location>,
) -> SolutionResult<Location> {
    let seed_ranges = seeds
        .iter()
        .tuples()
        .map(|(start, length)| (start.0, start.0 + length.0))
        .sorted_by_key(|(a, _)| *a)
        .collect_vec();

    let lowest = translation
        .translation
        .collapse_table()
        .0
        .into_iter()
        .sorted_by_key(|rule| rule.destination_range().0)
        .fold_while(None, |lowest, rule| {
            let (dst_start, _) = rule.destination_range();
            if lowest.map_or_else(|| true, |lowest_location| dst_start < lowest_location) {
                let rule_range = rule.source_range();
                if let Some(seed_range) = seed_ranges
                    .iter()
                    .find(|&&seed_range| TranslationRule::overlaps_with(seed_range, rule_range))
                {
                    let min_seed = max(rule_range.0, seed_range.0);
                    let min_location = rule.translate(&min_seed).unwrap();
                    FoldWhile::Continue(Some(min_location))
                } else {
                    FoldWhile::Continue(lowest)
                }
            } else {
                FoldWhile::Done(lowest)
            }
        });

    match lowest {
        FoldWhile::Done(Some(location)) | FoldWhile::Continue(Some(location)) => {
            Ok(Location(location))
        }
        FoldWhile::Continue(_) => Err(SolutionError::NoSolutionFound),
        _ => unreachable!(),
    }
}

fn parse_input(lines: &[String]) -> SolutionResult<(Vec<Seed>, TypedTranslation<Seed, Location>)> {
    let mut iter = lines.iter();
    let seeds: Vec<Seed> = iter
        .next()
        .ok_or_else(|| SolutionError::InputParsingFailed(owned!("Input is empty")))?
        .strip_prefix("seeds: ")
        .ok_or_else(|| SolutionError::InputParsingFailed(owned!("Malformed first line")))?
        .split_ascii_whitespace()
        .map(|d| d.parse::<u64>().map(Seed::from))
        .try_collect()?;

    tracing::debug!("Parsed seeds: {}", seeds.iter().join(" "));

    let tables: Vec<Translation> = iter
        .skip(1)
        .batching(|i| {
            if let Some(title) = i.next() {
                let result: SolutionResult<Translation> = (|| {
                    tracing::debug!("Parsing table {}", title);
                    let rules: Vector<TranslationRule> = i
                        .take_while(|l| !l.is_empty())
                        .map(|line| line.parse())
                        .try_collect()?;

                    tracing::debug!("Done parsing table {}, found {} rules", title, rules.len());
                    Ok(Translation::new(rules))
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
struct Seed(u64);

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
struct Soil(u64);
#[derive(Deref, From)]
struct Fertilizer(u64);
#[derive(Deref, From)]
struct Water(u64);
#[derive(Deref, From)]
struct Light(u64);
#[derive(Deref, From)]
struct Temperature(u64);
#[derive(Deref, From)]
struct Humidity(u64);
#[derive(Deref, From, PartialEq, PartialOrd, Eq, Ord)]
struct Location(u64);

enum Translation {
    Table(TranslationTable),
    Chain(Box<Translation>, Box<Translation>),
}

impl Translation {
    fn new<I>(rules: I) -> Self
    where
        I: IntoIterator<Item = TranslationRule>,
    {
        Translation::Table(TranslationTable::new(rules))
    }
}

struct TypedTranslation<I, O>
where
    I: Deref<Target = u64>,
    O: From<u64>,
{
    translation: Translation,
    input: PhantomData<I>,
    output: PhantomData<O>,
}

impl<I: Deref<Target = u64>, O: From<u64>> TypedTranslation<I, O> {
    fn transform(&self, input: &I) -> O {
        self.translation.translate(input).into()
    }
}

impl<I: Deref<Target = u64>, O: Deref<Target = u64> + From<u64>> TypedTranslation<I, O> {
    fn and_then<N>(self, other: TypedTranslation<O, N>) -> TypedTranslation<I, N>
    where
        N: From<u64>,
    {
        TypedTranslation {
            translation: self.translation.and_then(other.translation),
            input: PhantomData,
            output: PhantomData,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct TranslationRule {
    start: u64,
    end: u64,
    delta: i64,
}

impl PartialOrd for TranslationRule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TranslationRule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

impl Display for TranslationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}..{} -> {}..{}",
            self.start,
            self.end,
            self.start as i64 + self.delta,
            self.end as i64 + self.delta
        )
    }
}

impl TranslationRule {
    fn new(start: u64, end: u64, delta: i64) -> Self {
        if start > end {
            panic!("Start {} can not be greater than end {}", start, end);
        }

        TranslationRule { start, end, delta }
    }

    fn translate(&self, input: &u64) -> Option<u64> {
        if input >= &self.start && input <= &(self.end) {
            Some((*input as i64 + self.delta) as u64)
        } else {
            None
        }
    }

    fn source_range(&self) -> (u64, u64) {
        (self.start, self.end)
    }

    fn destination_range(&self) -> (u64, u64) {
        (
            (self.start as i64 + self.delta) as u64,
            (self.end as i64 + self.delta) as u64,
        )
    }

    fn length(&self) -> u64 {
        self.end - self.start + 1
    }

    fn overlaps_with(a: (u64, u64), b: (u64, u64)) -> bool {
        (a.0 >= b.0 && a.0 <= b.1) || (b.0 >= a.0 && b.0 <= a.1)
    }

    fn split(&self, length: u64) -> (TranslationRule, TranslationRule) {
        if length == 0 || self.start + length >= self.end {
            panic!("length must > 0 and < {}, got, {}", self.length(), length)
        }

        (
            TranslationRule {
                start: self.start,
                end: self.start + length - 1,
                delta: self.delta,
            },
            TranslationRule {
                start: self.start + length,
                end: self.end,
                delta: self.delta,
            },
        )
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

        let dest_start = destination_index?;
        let start: u64 = source_index?;
        let end: u64 = start + length? - 1;

        let delta = (dest_start as i64) - (start as i64);

        Ok(TranslationRule { start, end, delta })
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
struct TranslationTable(Vector<TranslationRule>);

impl TranslationTable {
    fn new<I>(rules: I) -> Self
    where
        I: IntoIterator<Item = TranslationRule>,
    {
        TranslationTable(rules.into_iter().sorted().collect())
    }

    fn translate(&self, input: &u64) -> u64 {
        let (output, rule) = self
            .0
            .iter()
            .find_map(|rule| rule.translate(input).map(|i| (i, Some(rule))))
            .unwrap_or((*input, None));

        if let Some(rule) = rule {
            tracing::debug!("Translating {input} -> {output} based on '{rule}'");
        } else {
            tracing::debug!("Translating {input} -> {output} because no rule matches");
        }
        output
    }

    fn map(&self, rule: TranslationRule) -> Vector<TranslationRule> {
        let result = self.0.iter().fold_while(
            (Vector::<TranslationRule>::new(), Some(rule)),
            |(mut new_rules, prev_leftover), rule| {
                let leftover = prev_leftover.unwrap();
                let (leftover_dst_start, leftover_dst_end) = leftover.destination_range();
                if rule.start > leftover_dst_end {
                    new_rules.push_back(leftover);
                    FoldWhile::Done((new_rules, None))
                } else if rule.end < leftover_dst_start {
                    FoldWhile::Continue((new_rules, Some(leftover)))
                } else {
                    let dst_start = max(leftover_dst_start, rule.start);
                    let dst_end = min(leftover_dst_end, rule.end);

                    let in_start = (dst_start as i64 - leftover.delta) as u64;
                    let in_end = (dst_end as i64 - leftover.delta) as u64;
                    let mapped =
                        TranslationRule::new(in_start, in_end, leftover.delta + rule.delta);

                    if dst_start > leftover_dst_start {
                        let (lower, _) = leftover.split(dst_start - leftover_dst_start);
                        new_rules.push_back(lower);
                    }

                    new_rules.push_back(mapped);

                    if dst_end < leftover_dst_end {
                        let next = TranslationRule {
                            start: in_end + 1,
                            end: leftover.end,
                            delta: leftover.delta,
                        };
                        FoldWhile::Continue((new_rules, Some(next)))
                    } else {
                        FoldWhile::Done((new_rules, None))
                    }
                }
            },
        );

        let rules = match result {
            FoldWhile::Done((rules, _)) | FoldWhile::Continue((rules, None)) => rules,
            FoldWhile::Continue((mut rules, Some(leftover))) => {
                rules.push_back(leftover);
                rules
            }
        };

        rules.into_iter().sorted().collect()
    }

    fn clear(self, input_range: (u64, u64)) -> Self {
        let updated_rules: Vec<_> = self
            .0
            .into_iter()
            .flat_map(|rule| {
                if rule.start >= input_range.0 && rule.end <= input_range.1 {
                    vector![]
                } else if !TranslationRule::overlaps_with(input_range, rule.source_range()) {
                    vector![rule]
                } else {
                    let mut result = vector![];

                    if input_range.0 > rule.start && input_range.0 < rule.end {
                        result.push_back(TranslationRule {
                            end: input_range.0 - 1,
                            ..rule
                        })
                    }

                    if input_range.1 > rule.start && input_range.1 < rule.end {
                        result.push_back(TranslationRule {
                            start: input_range.1 + 1,
                            ..rule
                        })
                    }

                    result
                }
            })
            .collect();

        TranslationTable::new(updated_rules)
    }

    fn insert(self, rule: TranslationRule) -> Self {
        let mut result = self.clear(rule.source_range());
        result.0.insert_ord(rule);
        result
    }

    fn fold(self, other: Self) -> Self {
        self.0
            .into_iter()
            .flat_map(|rule| other.map(rule))
            .fold(other.clone(), |result, new_rule| result.insert(new_rule))
    }
}

impl Translation {
    fn translate(&self, input: &u64) -> u64 {
        match self {
            Self::Table(table) => table.translate(input),
            Self::Chain(a, b) => b.translate(&a.translate(input)),
        }
    }

    fn and_then(self, other: Translation) -> Translation {
        Translation::Chain(Box::new(self), Box::new(other))
    }

    fn typed<I, O>(self) -> TypedTranslation<I, O>
    where
        I: Deref<Target = u64>,
        O: From<u64>,
    {
        TypedTranslation {
            translation: self,
            input: PhantomData,
            output: PhantomData,
        }
    }

    fn collapse_table(self) -> TranslationTable {
        match self {
            Self::Table(table) => table,
            Self::Chain(a, b) => a.collapse_table().fold(b.collapse_table()),
        }
    }
}

#[cfg(test)]
mod test {
    use im::vector;
    use utils::io::output::setup_logging;

    use crate::{TranslationRule, TranslationTable};

    #[test]
    fn test_rule_overlap() {
        assert_eq!(true, TranslationRule::overlaps_with((1, 5), (5, 9)));
        assert_eq!(true, TranslationRule::overlaps_with((1, 6), (5, 9)));
        assert_eq!(true, TranslationRule::overlaps_with((1, 9), (5, 6)));
        assert_eq!(true, TranslationRule::overlaps_with((1, 9), (5, 9)));
        assert_eq!(true, TranslationRule::overlaps_with((5, 9), (1, 6)));
        assert_eq!(true, TranslationRule::overlaps_with((5, 9), (1, 5)));
        assert_eq!(true, TranslationRule::overlaps_with((5, 6), (1, 9)));
        assert_eq!(true, TranslationRule::overlaps_with((5, 9), (1, 9)));

        assert_eq!(false, TranslationRule::overlaps_with((1, 4), (5, 8)));
        assert_eq!(false, TranslationRule::overlaps_with((5, 8), (1, 4)));
    }

    #[test]
    fn test_rule_utils() {
        let a = TranslationRule::new(1, 5, 2);

        assert_eq!((1, 5), a.source_range());
        assert_eq!((3, 7), a.destination_range());
        assert_eq!(5, a.length());

        let b = TranslationRule::new(3, 7, -2);

        assert_eq!((3, 7), b.source_range());
        assert_eq!((1, 5), b.destination_range());
        assert_eq!(5, b.length());
    }

    #[test]
    fn test_translation_table_map() {
        setup_logging();

        let table = TranslationTable::new(vec![
            TranslationRule::new(10, 19, 10),
            TranslationRule::new(21, 29, 20),
            TranslationRule::new(30, 39, -10),
        ]);

        let result = table.map(TranslationRule::new(0, 9, 0));
        assert_eq!(vector![TranslationRule::new(0, 9, 0)], result);

        let result = table.map(TranslationRule::new(0, 9, 10));
        assert_eq!(vector![TranslationRule::new(0, 9, 20)], result);

        let result = table.map(TranslationRule::new(0, 9, 5));
        assert_eq!(
            vector![
                TranslationRule::new(0, 4, 5),
                TranslationRule::new(5, 9, 15)
            ],
            result
        );

        let result = table.map(TranslationRule::new(5, 10, 7));
        assert_eq!(vector![TranslationRule::new(5, 10, 17)], result);

        let result = table.map(TranslationRule::new(0, 20, 5));
        assert_eq!(
            vector![
                TranslationRule::new(0, 4, 5),
                TranslationRule::new(5, 14, 5 + 10),
                TranslationRule::new(15, 15, 5),
                TranslationRule::new(16, 20, 5 + 20)
            ],
            result
        );

        let result = table.map(TranslationRule::new(0, 9, 35));
        assert_eq!(
            vector![
                TranslationRule::new(0, 4, 35 - 10),
                TranslationRule::new(5, 9, 35)
            ],
            result
        );

        let result = table.map(TranslationRule::new(0, 10, 40));
        assert_eq!(vector![TranslationRule::new(0, 10, 40)], result);
    }

    #[test]
    fn test_translation_table_clear() {
        let table = TranslationTable::new(vector![
            TranslationRule::new(10, 19, 0),
            TranslationRule::new(20, 29, 1),
            TranslationRule::new(31, 39, 2),
        ]);

        let result = table.clone().clear((0, 9));
        assert_eq!(result, table);

        let result = table.clone().clear((40, 49));
        assert_eq!(result, table);

        let result = table.clone().clear((30, 30));
        assert_eq!(result, table);

        let result = table.clone().clear((5, 15));
        assert_eq!(
            result.0,
            vector![
                TranslationRule::new(16, 19, 0),
                TranslationRule::new(20, 29, 1),
                TranslationRule::new(31, 39, 2)
            ]
        );

        let result = table.clone().clear((10, 19));
        assert_eq!(
            result.0,
            vector![
                TranslationRule::new(20, 29, 1),
                TranslationRule::new(31, 39, 2)
            ]
        );

        let result = table.clone().clear((5, 25));
        assert_eq!(
            result.0,
            vector![
                TranslationRule::new(26, 29, 1),
                TranslationRule::new(31, 39, 2)
            ]
        );

        let result = table.clone().clear((22, 25));
        assert_eq!(
            result.0,
            vector![
                TranslationRule::new(10, 19, 0),
                TranslationRule::new(20, 21, 1),
                TranslationRule::new(26, 29, 1),
                TranslationRule::new(31, 39, 2)
            ]
        );

        let result = table.clone().clear((35, 50));
        assert_eq!(
            result.0,
            vector![
                TranslationRule::new(10, 19, 0),
                TranslationRule::new(20, 29, 1),
                TranslationRule::new(31, 34, 2)
            ]
        );

        let result = table.clone().clear((10, 39));
        assert_eq!(result.0, vector![]);
    }

    #[test]
    fn test_simple_merge() {
        // let a = vector![TranslationRule {
        //     source_index: 5,
        //     destination_index: 2,
        //     length: 4
        // }];
        // let b = vector![
        //     TranslationRule {
        //         source_index: 2,
        //         destination_index: 10,
        //         length: 1
        //     },
        //     TranslationRule {
        //         source_index: 3,
        //         destination_index: 12,
        //         length: 2
        //     },
        //     TranslationRule {
        //         source_index: 5,
        //         destination_index: 1,
        //         length: 1
        //     }
        // ];

        // let mut expected = vector![
        //     TranslationRule {
        //         source_index: 5,
        //         destination_index: 10,
        //         length: 1
        //     },
        //     TranslationRule {
        //         source_index: 6,
        //         destination_index: 12,
        //         length: 2
        //     },
        //     TranslationRule {
        //         source_index: 8,
        //         destination_index: 1,
        //         length: 1
        //     },
        // ];
        // expected.sort_by( |a,b| a.source_index.cmp(&b.source_index) );

        // let result = Translation::merge(a, b);
        // assert_eq!(expected, result);
    }

    #[test]
    fn test_first_sample() {
        // let seed2soil = vector![
        //     TranslationRule { destination_index: 50, source_index: 98, length: 2 },
        //     TranslationRule { destination_index: 52, source_index: 50, length: 48 },
        // ];

        // let soil2fertilizer = vector![
        //     TranslationRule { destination_index: 0, source_index: 15, length: 37 },
        //     TranslationRule { destination_index: 37, source_index: 52, length: 2 },
        //     TranslationRule { destination_index: 39, source_index: 0, length: 15 },
        // ];

        // let expected = vector![
        //     TranslationRule { destination_index: 39, source_index: 0, length: 15 },
        //     TranslationRule { destination_index: 0, source_index: 34, length: 35 },
        //     TranslationRule { destination_index: 37, source_index: 50, length: 2 },
        //     TranslationRule { destination_index: 54, source_index: 51, length: 99 - 54 },
        //     TranslationRule { destination_index: 35, source_index: 98, length: 2 },
        // ];

        // let result = Translation::merge(seed2soil, soil2fertilizer);
        // assert_eq!(expected, result);
    }
}

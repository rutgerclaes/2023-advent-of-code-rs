use std::collections::HashMap;

use itertools::Itertools;
use utils::prelude::*;

type Parts = Vec<(HorizontalRange, u32)>;
type Symbols = HashMap<Point, char>;

fn main() {
    setup_logging();
    let lines: Vec<String> = read_input_lines().expect("Could not read input");
    let (parts, symbols) = parse_input(&lines);

    let part_one = part_one(&parts, &symbols);
    show_part_one(part_one);

    let part_two = part_two(&parts, &symbols);
    show_part_two(part_two);
}

fn part_one(parts: &Parts, symbols: &Symbols) -> u32 {
    parts
        .iter()
        .filter_map(|(hor_pos, num)| {
            let perimeter: Vec<_> = hor_pos.perimeter();
            if perimeter.iter().any(|pos| symbols.contains_key(pos)) {
                Some(num)
            } else {
                None
            }
        })
        .sum()
}

fn part_two(parts: &Parts, symbols: &Symbols) -> u32 {
    symbols
        .iter()
        .filter_map(|(pos, c)| {
            if *c != '*' {
                None
            } else {
                let touching_parts = parts
                    .iter()
                    .filter_map(|(hpos, num)| if hpos.touches(pos) { Some(*num) } else { None })
                    .collect_tuple();
                touching_parts.map(|(a, b)| a * b)
            }
        })
        .sum()
}

fn parse_input(input: &[String]) -> (Parts, Symbols) {
    input
        .iter()
        .enumerate()
        .fold((vec![], HashMap::new()), |(parts, symbols), (y, line)| {
            let (mut parts, symbols, acc) = line.chars().enumerate().fold(
                (parts, symbols, None),
                |(mut parts, mut symbols, acc), (x, c)| {
                    if c.is_ascii_digit() {
                        let next_acc = match acc {
                            None => (x, x, c.to_digit(10).unwrap()),
                            Some((begin, _, val)) => (begin, x, val * 10 + c.to_digit(10).unwrap()),
                        };

                        (parts, symbols, Some(next_acc))
                    } else {
                        if let Some((start, end, num)) = acc {
                            parts.push((HorizontalRange::new(start, end, y), num));
                        }

                        if c != '.' {
                            symbols.insert(Point::new(x, y), c);
                        }
                        (parts, symbols, None)
                    }
                },
            );

            if let Some((start, end, num)) = acc {
                parts.push((HorizontalRange::new(start, end, y), num))
            }

            (parts, symbols)
        })
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }
}

impl<I> From<(I, I)> for Point
where
    I: Into<usize>,
{
    fn from(value: (I, I)) -> Self {
        Point::new(value.0.into(), value.1.into())
    }
}

struct HorizontalRange {
    min_x: usize,
    max_x: usize,
    y: usize,
}

impl HorizontalRange {
    fn new(min_x: usize, max_x: usize, y: usize) -> Self {
        HorizontalRange { min_x, max_x, y }
    }

    fn perimeter<I>(&self) -> I
    where
        I: FromIterator<Point>,
    {
        let inter_x_start = if self.min_x > 0 {
            self.min_x - 1
        } else {
            self.min_x
        };
        let inter_x_end = self.max_x + 1;
        let inter_y_start = if self.y > 0 { self.y - 1 } else { self.y };
        let inter_y_end = self.y + 1;

        (inter_y_start..=inter_y_end)
            .flat_map(|y| (inter_x_start..=inter_x_end).map(move |x| (x, y)))
            .filter(|&(x, y)| y != self.y || (x < self.min_x || x > self.max_x))
            .map_into()
            .collect()
    }

    fn touches(&self, point: &Point) -> bool {
        if self.y == point.y {
            point.x + 1 == self.min_x || point.x == self.max_x + 1
        } else if self.y == point.y + 1 || self.y + 1 == point.y {
            point.x + 1 >= self.min_x && point.x <= self.max_x + 1
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_horiz_range_touches() {
        let range = HorizontalRange::new(1, 3, 1);
        let perim: Vec<Point> = range.perimeter();
        assert!(perim.iter().all(|p| range.touches(p)));
        let intern = (range.min_x..=range.max_x)
            .map(|x| Point::new(x, range.y))
            .collect_vec();
        assert!(intern.iter().all(|p| !range.touches(p)));
        assert_eq!(false, range.touches(&Point::new(0, 3)));
        assert_eq!(false, range.touches(&Point::new(5, 2)));
        assert_eq!(false, range.touches(&Point::new(5, 1)));

        let range = HorizontalRange::new(0, 2, 2);
        let perim: Vec<Point> = range.perimeter();
        assert!(perim.iter().all(|p| range.touches(p)));
        let intern = (range.min_x..=range.max_x)
            .map(|x| Point::new(x, range.y))
            .collect_vec();
        assert!(intern.iter().all(|p| !range.touches(p)));
        assert_eq!(false, range.touches(&Point::new(4, 0)));
        assert_eq!(false, range.touches(&Point::new(4, 2)));
        assert_eq!(false, range.touches(&Point::new(4, 1)));

        let range = HorizontalRange::new(0, 2, 0);
        let perim: Vec<Point> = range.perimeter();
        assert!(perim.iter().all(|p| range.touches(p)));
        let intern = (range.min_x..=range.max_x)
            .map(|x| Point::new(x, range.y))
            .collect_vec();
        assert!(intern.iter().all(|p| !range.touches(p)));
        assert_eq!(false, range.touches(&Point::new(4, 0)));
        assert_eq!(false, range.touches(&Point::new(4, 1)));
        assert_eq!(false, range.touches(&Point::new(1, 2)));
    }

    #[test]
    fn test_horiz_range_perimeter() {
        let test: Vec<Point> = HorizontalRange::new(1, 3, 1).perimeter();
        assert_eq!(
            vec![
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (0, 1),
                (4, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
                (4, 2)
            ]
            .iter()
            .map(|t: &(usize, usize)| Point::from(*t))
            .collect_vec(),
            test
        );

        let test: Vec<_> = HorizontalRange::new(1, 1, 1).perimeter();
        assert_eq!(
            vec![
                (0, 0),
                (1, 0),
                (2, 0),
                (0, 1),
                (2, 1),
                (0, 2),
                (1, 2),
                (2, 2)
            ]
            .iter()
            .map(|t: &(usize, usize)| Point::from(*t))
            .collect_vec(),
            test
        );

        let test: Vec<_> = HorizontalRange::new(0, 1, 1).perimeter();
        assert_eq!(
            vec![(0, 0), (1, 0), (2, 0), (2, 1), (0, 2), (1, 2), (2, 2)]
                .iter()
                .map(|t: &(usize, usize)| Point::from(*t))
                .collect_vec(),
            test
        );

        let test: Vec<_> = HorizontalRange::new(1, 3, 0).perimeter();
        assert_eq!(
            vec![(0, 0), (4, 0), (0, 1), (1, 1), (2, 1), (3, 1), (4, 1)]
                .iter()
                .map(|t: &(usize, usize)| Point::from(*t))
                .collect_vec(),
            test
        );

        let test: Vec<_> = HorizontalRange::new(0, 1, 0).perimeter();
        assert_eq!(
            vec![(2, 0), (0, 1), (1, 1), (2, 1)]
                .iter()
                .map(|t: &(usize, usize)| Point::from(*t))
                .collect_vec(),
            test
        );
    }
}

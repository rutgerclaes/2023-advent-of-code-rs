use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use derive_more::From;
use itertools::{FoldWhile, Itertools};
use once_cell::sync::Lazy;
use regex::Regex;
use utils::prelude::*;

fn main() {
    setup_logging();

    let input: Vec<_> = read_input_lines().expect("Input could not be read");
    let (instructions, network) = parse_input(input).expect("Input could not be parsed");

    let part_one = part_one(&instructions, &network);
    show_result_part_one(part_one);

    let part_two = part_two(&instructions, &network);
    show_result_part_two(part_two);
}

fn parse_input<I>(input: I) -> Result<(Vec<Instruction>, Network), SolutionError>
where
    I: IntoIterator<Item = String>,
{
    let mut iter = input.into_iter();

    let instruction_line: String = iter
        .next()
        .ok_or_else(|| SolutionError::InputParsingFailed(owned!("No instruction line in input")))?;
    let instructions: Vec<Instruction> = instruction_line
        .chars()
        .map(|c| c.try_into())
        .try_collect()?;

    let nodes: Vec<NodeDefinition> = iter
        .filter(|l| !l.is_empty())
        .map(|l| l.parse::<NodeDefinition>())
        .try_collect()?;

    Ok((instructions, Network::new(nodes)))
}

fn part_one(instructions: &[Instruction], network: &Network) -> Result<usize, SolutionError> {
    let result = instructions.iter().cycle().fold_while(
        (0, network.start()),
        |(length, position), instruction| match position {
            Some(node) if node.is_end() => {
                tracing::debug!(length = length, node = node.0, "Found the end");
                itertools::FoldWhile::Done((length, Some(node)))
            }
            Some(node) => {
                tracing::trace!(
                    length = length,
                    node = node.0,
                    "Following {:?}",
                    instruction
                );
                itertools::FoldWhile::Continue((length + 1, network.lookup(node, instruction)))
            }
            None => {
                tracing::error!(length = length, "Lost my way");
                itertools::FoldWhile::Done((0, None))
            }
        },
    );

    match result {
        FoldWhile::Continue(_) | FoldWhile::Done((_, None)) => Err(SolutionError::NoSolutionFound),
        FoldWhile::Done((result, _)) => Ok(result),
    }
}

fn part_two(instructions: &[Instruction], network: &Network) -> Result<u128, SolutionError> {
    let starts: Vec<_> = network.ghost_start();

    tracing::info!("Need to resolve {} paths", starts.len());

    let state: HashMap<&str, (&Node, usize, Option<usize>)> = starts
        .into_iter()
        .map(|node| (node.prefix(), (node, 0, None)))
        .collect();

    let result = instructions.iter().cycle().fold_while(
        Ok(state),
        |maybe_state: Result<HashMap<&str, _>, SolutionError>, instruction| match maybe_state {
            Err(e) => FoldWhile::Done(Err(e)),
            Ok(state) if state.values().all(|(_, _, l)| l.is_some()) => FoldWhile::Done(Ok(state)),
            Ok(state) => {
                let next_state: Result<HashMap<_, _>, _> = state
                    .into_iter()
                    .map(
                        |(prefix, (current_position, current_length, maybe_cycle))| {
                            let next_cycle = if current_position.is_ghost_end() {
                                match maybe_cycle {
                                    Some(existing_cycle) if existing_cycle != current_length => {
                                        tracing::error!(
                                            length = current_length,
                                            node = current_position.0,
                                            prefix = prefix,
                                            cycle = existing_cycle,
                                            "Existing cycle does not correspond with new cycle"
                                        );
                                        Err(SolutionError::NoSolutionFound)
                                    }
                                    Some(existing_cycle) => {
                                        tracing::trace!(
                                            length = current_length,
                                            node = current_position.0,
                                            prefix = prefix,
                                            cycle = existing_cycle,
                                            "Existing cycle corresponds with new cycle"
                                        );
                                        Ok(Some(existing_cycle))
                                    }
                                    None => {
                                        tracing::info!(
                                            length = current_length,
                                            node = current_position.0,
                                            prefix = prefix,
                                            cycle = current_length,
                                            "New cycle detected"
                                        );
                                        Ok(Some(current_length))
                                    }
                                }
                            } else {
                                tracing::trace!(
                                    length = current_length,
                                    node = current_position.0,
                                    prefix = prefix,
                                    "Ignoring cycle information, not and ghost endpoint"
                                );
                                Ok(maybe_cycle)
                            }?;

                            let next_position = match network.lookup(current_position, instruction)
                            {
                                Some(next) => {
                                    tracing::trace!(
                                        length = current_length,
                                        node = current_position.0,
                                        prefix = prefix,
                                        "Following {:?} from {:?} to {:?}",
                                        instruction,
                                        current_position.0,
                                        next.0
                                    );
                                    Ok(next)
                                }
                                None => {
                                    tracing::error!(
                                        node = current_position.0,
                                        prefix = prefix,
                                        "Lost my way for"
                                    );
                                    Err(SolutionError::NoSolutionFound)
                                }
                            }?;

                            Ok((prefix, (next_position, current_length + 1, next_cycle)))
                        },
                    )
                    .try_collect();

                FoldWhile::Continue(next_state)
            }
        },
    );

    match result {
        FoldWhile::Done(Ok(outcome)) => {
            let cycle_lengths: HashSet<usize> = outcome
                .values()
                .map(|(_, _, c)| c.ok_or(SolutionError::NoSolutionFound))
                .try_collect()?;
            Ok(cycle_lengths
                .into_iter()
                .fold(1u128, |a, b| num::integer::lcm(a, b as u128)))
        }
        FoldWhile::Continue(Err(e)) => Err(e),
        _ => unreachable!("Iteration never stops"),
    }
}

#[derive(Debug)]
enum Instruction {
    Left,
    Right,
}

impl Instruction {
    fn choose<'a, I>(&self, possibilities: &'a (I, I)) -> &'a I {
        match self {
            Self::Left => &possibilities.0,
            Self::Right => &possibilities.1,
        }
    }
}

impl TryFrom<char> for Instruction {
    type Error = SolutionError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_uppercase() {
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            _ => Err(SolutionError::InputParsingFailed(format!(
                "Could not translate {value} into an instruction"
            ))),
        }
    }
}

#[derive(From, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct Node(String);

impl Node {
    fn is_start(&self) -> bool {
        self.0 == "AAA"
    }
    fn is_end(&self) -> bool {
        self.0 == "ZZZ"
    }

    fn is_ghost_start(&self) -> bool {
        self.0.ends_with('A')
    }
    fn is_ghost_end(&self) -> bool {
        self.0.ends_with('Z')
    }

    fn prefix(&self) -> &str {
        &self.0[0..&self.0.len() - 1]
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

struct NodeDefinition {
    node: Node,
    left: Node,
    right: Node,
}

impl FromStr for NodeDefinition {
    type Err = SolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(?<node>[0-9A-Z]{3}) = \((?<left>[0-9A-Z]{3}), (?<right>[0-9A-Z]{3})\)$")
                .unwrap()
        });

        let m = capture_regex(&RE, s)?;

        let node = named_match(&m, "node")?.into();
        let left = named_match(&m, "left")?.into();
        let right = named_match(&m, "right")?.into();

        Ok(NodeDefinition { node, left, right })
    }
}

impl From<NodeDefinition> for (Node, (Node, Node)) {
    fn from(value: NodeDefinition) -> Self {
        (value.node, (value.left, value.right))
    }
}

struct Network(HashMap<Node, (Node, Node)>);

impl Network {
    fn new<I, N>(input: I) -> Self
    where
        I: IntoIterator<Item = N>,
        N: Into<(Node, (Node, Node))>,
    {
        Network(input.into_iter().map_into().collect())
    }

    fn start(&self) -> Option<&Node> {
        self.0.keys().find(|n| n.is_start())
    }

    fn ghost_start<'a, I>(&'a self) -> I
    where
        I: FromIterator<&'a Node>,
    {
        self.0.keys().filter(|n| n.is_ghost_start()).collect()
    }

    fn lookup(&self, node: &Node, instruction: &Instruction) -> Option<&Node> {
        self.0.get(node).map(|dirs| instruction.choose(dirs))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_parse_node_definition() {
        let input = "123 = (456, 789)";
        let node_def_res: Result<NodeDefinition, _> = input.parse();

        if let Ok(node_def) = node_def_res {
            assert_eq!(node_def.node.0, "123");
            assert_eq!(node_def.left.0, "456");
            assert_eq!(node_def.right.0, "789");
        } else {
            assert!(node_def_res.is_ok())
        }
    }

    #[test]
    fn test_instruction() {
        let left: Instruction = 'L'.try_into().unwrap();
        let right: Instruction = 'R'.try_into().unwrap();

        let options: (Node, Node) = ("A".into(), "B".into());
        assert_eq!(left.choose(&options), &options.0);
        assert_eq!(right.choose(&options), &options.1);
    }

    #[test]
    fn test_node_classifications() {
        let start: Node = "AAA".into();
        let end: Node = "ZZZ".into();

        let ghost_start: Node = "BBA".into();
        let ghost_end: Node = "BBZ".into();

        assert_eq!(start.is_start(), true);
        assert_eq!(start.is_end(), false);
        assert_eq!(start.is_ghost_start(), true);
        assert_eq!(start.is_ghost_end(), false);

        assert_eq!(end.is_start(), false);
        assert_eq!(end.is_end(), true);
        assert_eq!(end.is_ghost_start(), false);
        assert_eq!(end.is_ghost_end(), true);

        assert_eq!(ghost_start.is_start(), false);
        assert_eq!(ghost_start.is_end(), false);
        assert_eq!(ghost_start.is_ghost_start(), true);
        assert_eq!(ghost_start.is_ghost_end(), false);

        assert_eq!(ghost_end.is_start(), false);
        assert_eq!(ghost_end.is_end(), false);
        assert_eq!(ghost_end.is_ghost_start(), false);
        assert_eq!(ghost_end.is_ghost_end(), true);
    }

    #[test]
    fn test_node_prefix() {
        let test: Node = "123".into();
        assert_eq!(test.prefix(), "12");
    }

    #[test]
    fn test_network_start() {
        let input: Vec<NodeDefinition> = vec![
            "AAA = (BBB, CCC)".parse().unwrap(),
            "CCC = (AAA, BBB)".parse().unwrap(),
            "BBB = (CCC, AAA)".parse().unwrap(),
        ];

        let network = Network::new(input);
        let start: Node = "AAA".into();

        assert_eq!(network.start().unwrap(), &start);
    }

    #[test]
    fn test_network_ghost_start() {
        let input: Vec<NodeDefinition> = vec![
            "AAA = (AAX, BBA)".parse().unwrap(),
            "BBA = (BBX, CCA)".parse().unwrap(),
            "CCA = (CCX, AAA)".parse().unwrap(),
            "AAZ = (AAX, BBA)".parse().unwrap(),
            "BBZ = (BBX, CCA)".parse().unwrap(),
            "CCZ = (CCX, AAA)".parse().unwrap(),
            "AAX = (AAZ, AAA)".parse().unwrap(),
            "BBX = (BBZ, BBB)".parse().unwrap(),
            "CCX = (CCZ, CCA)".parse().unwrap(),
        ];

        let network = Network::new(input);

        let a: Node = "AAA".into();
        let b: Node = "BBA".into();
        let c: Node = "CCA".into();

        let mut ghost_starts: Vec<&Node> = network.ghost_start();
        ghost_starts.sort();

        assert_eq!(ghost_starts, vec![&a, &b, &c]);
    }

    #[test]
    fn test_network_lookup() {
        let input: Vec<NodeDefinition> = vec![
            "AAA = (BBB, CCC)".parse().unwrap(),
            "BBB = (CCC, AAA)".parse().unwrap(),
            "CCC = (AAA, ZZZ)".parse().unwrap(),
        ];

        let network = Network::new(input);

        let a: Node = "AAA".into();
        let b: Node = "BBB".into();
        let c: Node = "CCC".into();
        let z: Node = "ZZZ".into();

        assert_eq!(network.lookup(&a, &Instruction::Left), Some(&b));
        assert_eq!(network.lookup(&a, &Instruction::Right), Some(&c));

        assert_eq!(network.lookup(&b, &Instruction::Left), Some(&c));
        assert_eq!(network.lookup(&b, &Instruction::Right), Some(&a));

        assert_eq!(network.lookup(&c, &Instruction::Left), Some(&a));
        assert_eq!(network.lookup(&c, &Instruction::Right), Some(&z));

        assert_eq!(network.lookup(&z, &Instruction::Left), None);
        assert_eq!(network.lookup(&z, &Instruction::Right), None);
    }
}

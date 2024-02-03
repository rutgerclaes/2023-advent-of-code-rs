use itertools::Itertools;
use std::convert::TryInto;
use std::str::FromStr;
use utils::prelude::*;

fn main() {
    setup_logging();

    let bids: Vec<HandWithBid> = parse_input_lines().expect("Input could not be parsed");
    let part_one = part_one(&bids);
    show_part_one(part_one);

    let part_two = part_two(bids);
    show_part_one(part_two);
}

fn part_one(bids: &[HandWithBid]) -> u64 {
    bids.iter()
        .sorted_by_key(|HandWithBid(hand, _)| hand)
        .enumerate()
        .map(|(pos, HandWithBid(_, bid))| (pos + 1) as u64 * *bid as u64)
        .sum()
}

fn part_two(bids: Vec<HandWithBid>) -> u64 {
    part_one(
        &bids
            .into_iter()
            .map(|b| b.replace_jack_with_joker())
            .collect_vec(),
    )
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
enum Card {
    Joker,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<char> for Card {
    type Error = SolutionError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' => Ok(Self::Ace),
            '2' => Ok(Self::Two),
            '3' => Ok(Self::Three),
            '4' => Ok(Self::Four),
            '5' => Ok(Self::Five),
            '6' => Ok(Self::Six),
            '7' => Ok(Self::Seven),
            '8' => Ok(Self::Eight),
            '9' => Ok(Self::Nine),
            'T' => Ok(Self::Ten),
            'J' => Ok(Self::Jack),
            'Q' => Ok(Self::Queen),
            'K' => Ok(Self::King),
            value => Err(SolutionError::InputParsingFailed(format!(
                "Could not parse '{}'",
                value
            ))),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

impl HandType {
    fn from<I>(cards: I) -> Self
    where
        I: IntoIterator<Item = Card>,
    {
        let cards = cards.into_iter().collect_vec();
        if cards.len() != 5 {
            panic!("Number of cards passed to HandType is not 5: {:?}", cards);
        }

        let mut groups = cards.iter().counts();
        let jokers = groups.remove(&Card::Joker).unwrap_or(0);

        let max_count = groups.values().max().unwrap_or(&0);

        if max_count + jokers == 5 {
            Self::FiveOfAKind
        } else if max_count + jokers == 4 {
            Self::FourOfAKind
        } else if groups.len() == 2 {
            Self::FullHouse
        } else if max_count + jokers == 3 {
            Self::ThreeOfAKind
        } else if groups.len() == 3 {
            Self::TwoPair
        } else if max_count + jokers == 2 {
            Self::OnePair
        } else {
            Self::HighCard
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Hand {
    cards: [Card; 5],
    hand_type: HandType,
}

impl Hand {
    fn new(cards: [Card; 5]) -> Self {
        Hand {
            cards,
            hand_type: HandType::from(cards),
        }
    }

    fn from<I>(cards: I) -> Result<Self, SolutionError>
    where
        I: IntoIterator<Item = Card>,
    {
        let cards: [Card; 5] = cards.into_iter().collect_vec().try_into().map_err(|_| {
            SolutionError::InputParsingFailed(owned!("Hand with more than 5 cards found"))
        })?;
        Ok(Self::new(cards))
    }

    fn replace_jack_with_joker(self) -> Self {
        let updated_cards = self
            .cards
            .into_iter()
            .map(|l| if l == Card::Jack { Card::Joker } else { l })
            .collect_vec();

        Self::from(updated_cards).unwrap()
    }
}

impl FromStr for Hand {
    type Err = SolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let symbols: Vec<_> = s.chars().map(Card::try_from).try_collect()?;
        let symbols: [Card; 5] = symbols.try_into().expect("There should be 5 Cards");
        Ok(Hand::new(symbols))
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.hand_type == other.hand_type {
            self.cards.cmp(&other.cards)
        } else {
            self.hand_type.cmp(&other.hand_type)
        }
    }
}

#[derive(Eq, PartialEq)]
struct HandWithBid(Hand, u32);

impl HandWithBid {
    fn replace_jack_with_joker(self) -> Self {
        Self(self.0.replace_jack_with_joker(), self.1)
    }
}

impl FromStr for HandWithBid {
    type Err = SolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hand, bid) = s
            .split_ascii_whitespace()
            .collect_tuple()
            .ok_or_else(|| SolutionError::InputParsingFailed(format!("Could not parse '{}'", s)))?;
        Ok(HandWithBid(hand.parse()?, bid.parse()?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn test_hand_parsing() {
        let hand: Hand = "32T3K".parse().expect("Parsing should work");
        assert_eq!(HandType::OnePair, hand.hand_type);
        assert_eq!(
            [Card::Three, Card::Two, Card::Ten, Card::Three, Card::King],
            hand.cards
        );

        let hand: Hand = "T55J5".parse().expect("Parsing should work");
        assert_eq!(HandType::ThreeOfAKind, hand.hand_type);
        assert_eq!(
            [Card::Ten, Card::Five, Card::Five, Card::Jack, Card::Five],
            hand.cards
        );

        let hand: Hand = "KK677".parse().expect("Parsing should work");
        assert_eq!(HandType::TwoPair, hand.hand_type);
        assert_eq!(
            [Card::King, Card::King, Card::Six, Card::Seven, Card::Seven],
            hand.cards
        );

        let hand: Hand = "KTJJT".parse().expect("Parsing should work");
        assert_eq!(HandType::TwoPair, hand.hand_type);
        assert_eq!(
            [Card::King, Card::Ten, Card::Jack, Card::Jack, Card::Ten],
            hand.cards
        );

        let hand: Hand = "QQQJQ".parse().expect("Parsing should work");
        assert_eq!(HandType::FourOfAKind, hand.hand_type);
        assert_eq!(
            [
                Card::Queen,
                Card::Queen,
                Card::Queen,
                Card::Jack,
                Card::Queen
            ],
            hand.cards
        );
    }

    #[test]
    fn test_hand_sorting() {
        let hands: [Hand; 5] = [
            "32T3K".parse().expect("Parsing should work"), // 1
            "T55J5".parse().expect("Parsing should work"), // 4
            "KK677".parse().expect("Parsing should work"), // 3
            "KTJJT".parse().expect("Parsing should work"), // 2
            "QQQJA".parse().expect("Parsing should work"), // 5
        ];

        let order = hands
            .iter()
            .enumerate()
            .sorted_by_key(|&(_, hand)| hand)
            .map(|(pos, _)| pos)
            .collect_vec();

        assert_eq!(vec![0, 3, 2, 1, 4], order);
    }

    #[test]
    fn test_hand_sorting_with_joker() {
        let hands: [Hand; 5] = [
            "32T3K"
                .parse::<Hand>()
                .expect("Parsing should work")
                .replace_jack_with_joker(), // 1
            "T55J5"
                .parse::<Hand>()
                .expect("Parsing should work")
                .replace_jack_with_joker(), // 3
            "KK677"
                .parse::<Hand>()
                .expect("Parsing should work")
                .replace_jack_with_joker(), // 2
            "KTJJT"
                .parse::<Hand>()
                .expect("Parsing should work")
                .replace_jack_with_joker(), // 5
            "QQQJA"
                .parse::<Hand>()
                .expect("Parsing should work")
                .replace_jack_with_joker(), // 4
        ];

        assert_eq!(HandType::OnePair, hands[0].hand_type);
        assert_eq!(HandType::FourOfAKind, hands[1].hand_type);
        assert_eq!(HandType::TwoPair, hands[2].hand_type);
        assert_eq!(HandType::FourOfAKind, hands[3].hand_type);
        assert_eq!(HandType::FourOfAKind, hands[4].hand_type);

        let order = hands
            .iter()
            .enumerate()
            .sorted_by_key(|&(_, hand)| hand)
            .map(|(pos, _)| pos)
            .collect_vec();

        assert_eq!(vec![0, 2, 1, 4, 3], order);
    }

    #[test]
    fn test_hand_ordering() {
        let a: Hand = "A2222".parse().expect("Parsing should work");
        let b: Hand = "K2222".parse().expect("Parsing should work");
        let c: Hand = "2222K".parse().expect("Parsing should work");
        let d: Hand = "3333K".parse().expect("Parsing should work");

        assert!(a > b);
        assert!(a > c);
        assert!(a > d);

        assert!(b > c);
        assert!(b > d);

        assert!(d > c);
    }

    #[test]
    fn test_joker_parsing() {
        let hand: Hand = "A2345"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::HighCard, hand.hand_type);
        assert_eq!(
            [Card::Ace, Card::Two, Card::Three, Card::Four, Card::Five],
            hand.cards
        );

        let hand: Hand = "AJ345"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::OnePair, hand.hand_type);
        assert_eq!(
            [Card::Ace, Card::Joker, Card::Three, Card::Four, Card::Five],
            hand.cards
        );

        let hand: Hand = "AJ335"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::ThreeOfAKind, hand.hand_type);
        assert_eq!(
            [Card::Ace, Card::Joker, Card::Three, Card::Three, Card::Five],
            hand.cards
        );

        let hand: Hand = "AJJ45"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::ThreeOfAKind, hand.hand_type);
        assert_eq!(
            [Card::Ace, Card::Joker, Card::Joker, Card::Four, Card::Five],
            hand.cards
        );

        let hand: Hand = "AAJ44"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::FullHouse, hand.hand_type);
        assert_eq!(
            [Card::Ace, Card::Ace, Card::Joker, Card::Four, Card::Four],
            hand.cards
        );

        let hand: Hand = "AAJA4"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::FourOfAKind, hand.hand_type);
        assert_eq!(
            [Card::Ace, Card::Ace, Card::Joker, Card::Ace, Card::Four],
            hand.cards
        );

        let hand: Hand = "AAJJ4"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::FourOfAKind, hand.hand_type);
        assert_eq!(
            [Card::Ace, Card::Ace, Card::Joker, Card::Joker, Card::Four],
            hand.cards
        );

        let hand: Hand = "AJJJ4"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::FourOfAKind, hand.hand_type);
        assert_eq!(
            [Card::Ace, Card::Joker, Card::Joker, Card::Joker, Card::Four],
            hand.cards
        );

        let hand: Hand = "JJJJ4"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::FiveOfAKind, hand.hand_type);
        assert_eq!(
            [
                Card::Joker,
                Card::Joker,
                Card::Joker,
                Card::Joker,
                Card::Four
            ],
            hand.cards
        );

        let hand: Hand = "J4444"
            .parse::<Hand>()
            .expect("Parsing should work")
            .replace_jack_with_joker();
        assert_eq!(HandType::FiveOfAKind, hand.hand_type);
        assert_eq!(
            [Card::Joker, Card::Four, Card::Four, Card::Four, Card::Four],
            hand.cards
        );
    }
}

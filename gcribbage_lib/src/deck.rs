use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::Ordering;

/// Represents the ranks of cards.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Rank {
    Ace,
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
}

impl Rank {
    /// Makes an iterator for all the ranks.
    pub fn iter() -> impl Iterator<Item = Self> {
        use Rank::*;
        [
            Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King,
        ]
        .iter()
        .copied()
    }

    pub fn prev(&self) -> Rank {
        use Rank::*;
        match self {
            Ace => King,
            Two => Ace,
            Three => Two,
            Four => Three,
            Five => Four,
            Six => Five,
            Seven => Six,
            Eight => Seven,
            Nine => Eight,
            Ten => Nine,
            Jack => Ten,
            Queen => Jack,
            King => Queen,
        }
    }

    pub fn next(&self) -> Rank {
        use Rank::*;
        match self {
            Ace => Two,
            Two => Three,
            Three => Four,
            Four => Five,
            Five => Six,
            Six => Seven,
            Seven => Eight,
            Eight => Nine,
            Nine => Ten,
            Ten => Jack,
            Jack => Queen,
            Queen => King,
            King => Ace,
        }
    }

    /// Makes a new rank from the given character, or panics if it doesn't
    /// know how to make this.
    pub fn from(origin: char) -> Rank {
        use Rank::*;
        match origin {
            'a' | 'A' => Ace,
            '2' => Two,
            '3' => Three,
            '4' => Four,
            '5' => Five,
            '6' => Six,
            '7' => Seven,
            '8' => Eight,
            '9' => Nine,
            't' | 'T' => Ten,
            'j' | 'J' => Jack,
            'q' | 'Q' => Queen,
            'k' | 'K' => King,
            _ => panic!("Can't convert {:?} to a rank", origin),
        }
    }

    pub fn to_char(&self) -> char {
        use Rank::*;
        match self {
            Ace => 'A',
            Two => '2',
            Three => '3',
            Four => '4',
            Five => '5',
            Six => '6',
            Seven => '7',
            Eight => '8',
            Nine => '9',
            Ten => 'T',
            Jack => 'J',
            Queen => 'Q',
            King => 'K',
        }
    }

    /// Converts a rank to an ordinal number, aces low, kings high.
    pub fn ordinal(&self) -> usize {
        use Rank::*;
        match self {
            Ace => 1,
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8,
            Nine => 9,
            Ten => 10,
            Jack => 11,
            Queen => 12,
            King => 13,
        }
    }

    pub fn value(&self) -> u8 {
        use Rank::*;
        match self {
            Ace => 1,
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8,
            Nine => 9,
            _ => 10,
        }
    }
}

impl Ord for Rank {
    fn cmp(&self, other: &Rank) -> Ordering {
        self.ordinal().cmp(&other.ordinal())
    }
}

impl PartialOrd for Rank {
    fn partial_cmp(&self, other: &Rank) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a suit
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
}

impl Suit {
    /// Makes an iterator for all the suits.
    pub fn iter() -> impl Iterator<Item = Self> {
        use Suit::*;
        [Spades, Hearts, Clubs, Diamonds].iter().copied()
    }

    /// Makes a new suit from the given character, or panics if it doesn't
    /// know how to make this.
    pub fn from(origin: char) -> Suit {
        use Suit::*;
        match origin {
            's' | 'S' => Spades,
            'h' | 'H' => Hearts,
            'c' | 'C' => Clubs,
            'd' | 'D' => Diamonds,
            _ => panic!("Can't convert {:?} to a suit", origin),
        }
    }

    pub fn to_symbol(&self) -> char {
        use Suit::*;
        match self {
            Spades => '\u{2660}',
            Hearts => '\u{2665}',
            Clubs => '\u{2663}',
            Diamonds => '\u{2666}',
        }
    }

    pub fn ordinal(&self) -> usize {
        use Suit::*;
        match self {
            Spades => 1,
            Hearts => 2,
            Clubs => 3,
            Diamonds => 4,
        }
    }

    pub fn next(&self) -> Self {
        use Suit::*;
        match self {
            Spades => Hearts,
            Hearts => Clubs,
            Clubs => Diamonds,
            Diamonds => Spades,
        }
    }

    pub fn prev(&self) -> Self {
        use Suit::*;
        match self {
            Spades => Diamonds,
            Hearts => Spades,
            Clubs => Hearts,
            Diamonds => Clubs,
        }
    }
}

impl Ord for Suit {
    fn cmp(&self, other: &Suit) -> Ordering {
        self.ordinal().cmp(&other.ordinal())
    }
}

impl PartialOrd for Suit {
    fn partial_cmp(&self, other: &Suit) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { rank, suit }
    }

    pub fn from(origin: &str) -> Self {
        Card::new(
            Suit::from(origin.chars().nth(1).unwrap()),
            Rank::from(origin.chars().next().unwrap()),
        )
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Card) -> Ordering {
        match self.rank.cmp(&other.rank) {
            Ordering::Equal => self.suit.cmp(&other.suit),
            ord => ord,
        }
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Card) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Deck {
    cards: Vec<Card>,
    stacking: Option<Vec<Card>>,
}

impl Deck {
    pub fn new() -> Self {
        let cards = Suit::iter()
            .flat_map(|s| Rank::iter().map(move |r| Card { suit: s, rank: r }))
            .collect();
        Self {
            cards,
            stacking: None,
        }
    }
    pub fn stacked(cards: Vec<Card>) -> Self {
        let stacking = Some(cards.to_vec());
        Self { cards, stacking }
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Deck {
    pub fn shuffle(&mut self) {
        match &self.stacking {
            Some(stacking) => {
                self.cards.clear();
                self.cards.extend(stacking);
            }
            None => {
                self.cards.clear();
                self.cards.extend(
                    Suit::iter().flat_map(|s| Rank::iter().map(move |r| Card { suit: s, rank: r })),
                );
                self.cards.shuffle(&mut thread_rng());
            }
        }
    }

    pub fn deal(&mut self, count: usize) -> Vec<Card> {
        self.cards.drain(0..count).collect()
    }
}

use std::cmp::Ordering;

/// Represents the ranks of cards.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
        ].iter().copied()
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
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
        [ Spades, Hearts, Clubs, Diamonds ].iter().copied()
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
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn from(origin: &str) -> Self {
        Card{
            rank: Rank::from(origin.chars().next().unwrap()),
            suit: Suit::from(origin.chars().nth(1).unwrap()),
        }
    }
}

pub struct Deck {
    cards: Vec<Card>
}

pub trait Dealable {
    fn shuffle(&mut self);
    fn deal(&mut self, count: usize) -> Vec<Card>;
}

impl Deck {
    pub fn new() -> Self {
        let cards = Suit::iter()
            .flat_map(|s| {
                Rank::iter()
                    .map(move |r| Card {suit: s, rank: r})
            })
            .collect();
        Self {
            cards,
        }
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Dealable for Deck {
    fn shuffle(&mut self) {}
    fn deal(&mut self, count: usize) -> Vec<Card> {
        self.cards.drain(0..count).collect()
    }
}

pub struct StackedDeck<'a> {
    cards: &'a mut Vec<Card>
}

impl<'a> StackedDeck<'a> {
    pub fn new(cards: &'a mut Vec<Card>) -> Self {
        Self{
            cards 
        }
    }
}

impl<'a> Dealable for StackedDeck<'a> {
    fn shuffle(&mut self) {}
    fn deal(&mut self, count: usize) -> Vec<Card> {
        self.cards.drain(0..count).collect()
    }
}

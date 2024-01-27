use crate::deck::Card;
use std::collections::HashSet;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum PlayerPosition {
    First,
    Second,
}

impl PlayerPosition {
    pub fn next(self) -> Self {
        use PlayerPosition::*;
        match self {
            First => Second,
            Second => First,
        }
    }
}

pub trait KnowsCribbage {
    fn choose_crib(&mut self, hand: &HashSet<Card>) -> Vec<Card>;
    fn play(&mut self, hand: &HashSet<Card>) -> Card;
}

pub struct SimplePlayer {}

impl KnowsCribbage for SimplePlayer {
    fn choose_crib(&mut self, hand: &HashSet<Card>) -> Vec<Card> {
        let mut sorted = hand.iter().copied().collect::<Vec<Card>>();
        sorted.sort();
        sorted[0..2].to_vec()
    }

    fn play(&mut self, hand: &HashSet<Card>) -> Card {
        let mut sorted = hand.iter().copied().collect::<Vec<Card>>();
        sorted.sort();
        sorted[0]
    }
}

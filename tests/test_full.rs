use gcribbage::simulation::deck::{Card, Deck};
use gcribbage::simulation::player::KnowsCribbage;
use gcribbage::simulation::runner::{GameRunner, PlayResult};
use std::collections::HashSet;

struct TestPlayer {}

impl KnowsCribbage for TestPlayer {
    fn choose_crib(&mut self, hand: &HashSet<Card>) -> Vec<Card> {
        let mut sorted = hand.into_iter().map(|c| *c).collect::<Vec<Card>>();
        sorted.sort();
        sorted[0..2].to_vec()
    }
}

#[test]
fn test_full_game() {
    let cards = [
        Card::from("as"),
        Card::from("js"),
        Card::from("as"),
        Card::from("2s"),
        Card::from("3s"),
        Card::from("4s"),
        Card::from("5s"),
        Card::from("6s"),
        Card::from("ac"),
        Card::from("2c"),
        Card::from("3c"),
        Card::from("4c"),
        Card::from("5c"),
        Card::from("6c"),
        Card::from("ad"),
    ];
    let cards_vec = cards.to_vec();
    let deck = Deck::stacked(cards_vec);
    let mut first = TestPlayer {};
    let mut second = TestPlayer {};
    let mut runner = GameRunner::new_with_deck(&mut first, &mut second, deck);
    loop {
        let result = runner.do_next_play();
        match result {
            Some(PlayResult::GameOver(_)) | None => break,
            _ => (),
        }
    }
}

use gcribbage::simulation::runner::{GameRunner, PlayResult};
use gcribbage::simulation::player::PlayerPosition;
use gcribbage::simulation::deck::{StackedDeck, Card};

#[test]
fn test_full_game() {
    let mut cards = vec![
        Card::from("as"),
        Card::from("js"),
    ];
    let mut deck = StackedDeck::new(&mut cards);
    let runner = GameRunner::new(&mut deck);
    let result = runner.last().unwrap();

    match result {
        PlayResult::DealerChosen(result) => {
            assert_eq!(result.up_cards, (Card::from("as"), Card::from("js")));
            assert_eq!(result.dealer, PlayerPosition::First);
        },
    }
}

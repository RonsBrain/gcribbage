use crate::simulation::runner::{GameRunner, GameState, PlayResult, CutInfo};
use crate::simulation::deck::{Dealable};
use crate::simulation::player::PlayerPosition;

impl<'a, D: Dealable> GameRunner<'a, D> {
    /// Implements the rules to choose a dealer.
    ///
    /// This shuffles the deck, takes two cards from it, and uses those to
    /// determine the dealer. Lowest card goes first.
    pub fn choose_dealer(&mut self) -> (GameState, Option<PlayResult>) {
        loop {
            let cards = self.deck.deal(2);
            /* If the cards are a tie, choose two new ones. That way we don't
             * have to write tie-breaking logic for the user interface.
             */
            if cards[0].rank == cards[1].rank {
                continue;
            }
            let dealer = match cards[0].rank < cards[1].rank {
                true => PlayerPosition::First,
                false => PlayerPosition::Second,
            };
            let result = CutInfo {
                up_cards: (cards[0], cards[1]),
                dealer,
            };
            return (GameState::Done, Some(PlayResult::DealerChosen(result)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::deck::{Card, StackedDeck};

    #[test]
    fn test_selects_right_dealer() {
        for (cards, expected_dealer) in [
            ([Card::from("as"), Card::from("ks")], PlayerPosition::First),
            ([Card::from("ks"), Card::from("as")], PlayerPosition::Second),
        ] {
            let mut deck_cards = Vec::from(cards);
            let mut deck = StackedDeck::new(&mut deck_cards);
            let mut runner = GameRunner::new(&mut deck);
            let result = runner.next().unwrap();
            match result {
                PlayResult::DealerChosen(info) => {
                    assert_eq!(info.dealer, expected_dealer);
                    assert_eq!(info.up_cards.0, cards[0]);
                    assert_eq!(info.up_cards.1, cards[1]);
                }
            }
        }
    }

    #[test]
    fn test_does_not_entertain_a_tie() {
        let cards = [
            Card::from("as"),
            Card::from("ah"),
            Card::from("ac"),
            Card::from("kc")
        ];
        let mut deck_cards = Vec::from(cards);
        let mut deck = StackedDeck::new(&mut deck_cards);
        let mut runner = GameRunner::new(&mut deck);
        let result = runner.next().unwrap();
        match result {
            PlayResult::DealerChosen(info) => {
                assert_eq!(info.dealer, PlayerPosition::First);
                assert_eq!(info.up_cards.0, cards[2]);
                assert_eq!(info.up_cards.1, cards[3]);
            }
        }
    }
}

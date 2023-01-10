use crate::simulation::deck::{Card, Deck, Rank};
use crate::simulation::player::{KnowsCribbage, PlayerPosition};
use std::collections::{HashMap, HashSet};

enum GameState {
    ChooseDealer,
    DealCards,
    ChooseCrib,
    Done,
}

pub struct GameRunner<'a, P: KnowsCribbage> {
    /// The main game runner. Simulates all the rules of cribbage
    /// in the right sequence.
    state: GameState,
    dealer: PlayerPosition,
    deck: Deck,
    hands: HashMap<PlayerPosition, HashSet<Card>>,
    scores: HashMap<PlayerPosition, u8>,
    players: HashMap<PlayerPosition, &'a mut P>,
    up_card: Option<Card>,
}

#[derive(Debug)]
pub struct CutInfo {
    pub up_cards: HashMap<PlayerPosition, Card>,
    pub dealer: PlayerPosition,
}

#[derive(Debug)]
pub struct Deal {
    pub dealer: PlayerPosition,
    pub hands: HashMap<PlayerPosition, HashSet<Card>>,
    pub scores: HashMap<PlayerPosition, u8>,
}

#[derive(Debug)]
pub struct UpCardInfo {
    pub dealer: PlayerPosition,
    pub scores: HashMap<PlayerPosition, u8>,
    pub up_card: Card,
    pub is_nibs: bool,
}

#[derive(Debug)]
pub struct FinalScores {
    pub winner: PlayerPosition,
    pub scores: HashMap<PlayerPosition, u8>,
}

#[derive(Debug)]
pub enum PlayResult {
    DealerChosen(CutInfo),
    WaitingForCrib(Deal),
    AnnounceUpCard(UpCardInfo),
    GameOver(FinalScores),
}

/// Implements the rules to choose a dealer.
///
/// This shuffles the deck, takes two cards from it, and uses those to
/// determine the dealer. Lowest card goes first.
fn choose_dealer(deck: &mut Deck) -> (HashMap<PlayerPosition, Card>, PlayerPosition) {
    loop {
        let cards = deck.deal(2);
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
        let mut up_cards = HashMap::new();
        up_cards.insert(PlayerPosition::First, cards[0]);
        up_cards.insert(PlayerPosition::Second, cards[1]);
        return (up_cards, dealer);
    }
}

fn deal_cards(deck: &mut Deck) -> HashMap<PlayerPosition, HashSet<Card>> {
    let mut hands: HashMap<PlayerPosition, HashSet<Card>> = HashMap::new();
    deck.shuffle();
    hands.insert(
        PlayerPosition::First,
        HashSet::from_iter(deck.deal(6).iter().copied()),
    );
    hands.insert(
        PlayerPosition::Second,
        HashSet::from_iter(deck.deal(6).iter().copied()),
    );
    hands
}

fn choose_crib<'a, P: KnowsCribbage>(
    players: &mut HashMap<PlayerPosition, &'a mut P>,
    hands: &HashMap<PlayerPosition, HashSet<Card>>,
) -> HashMap<PlayerPosition, Vec<Card>> {
    let mut choices = HashMap::new();
    for position in [PlayerPosition::First, PlayerPosition::Second] {
        let player = players.get_mut(&position).unwrap();
        let hand = hands.get(&position).unwrap();
        choices.insert(position, player.choose_crib(hand));
    }
    choices
}

impl<'a, P: KnowsCribbage> GameRunner<'a, P> {
    pub fn new(first: &'a mut P, second: &'a mut P) -> Self {
        let deck = Deck::new();
        Self::new_with_deck(first, second, deck)
    }

    pub fn new_with_deck(first: &'a mut P, second: &'a mut P, deck: Deck) -> Self {
        let mut hands = HashMap::new();
        hands.insert(PlayerPosition::First, HashSet::new());
        hands.insert(PlayerPosition::Second, HashSet::new());

        let mut scores = HashMap::new();
        scores.insert(PlayerPosition::First, 0);
        scores.insert(PlayerPosition::Second, 0);

        let mut players = HashMap::new();
        players.insert(PlayerPosition::First, first);
        players.insert(PlayerPosition::Second, second);

        Self {
            state: GameState::ChooseDealer,
            deck,
            hands,
            scores,
            players,
            dealer: PlayerPosition::First,
            up_card: None,
        }
    }

    /// Iterates over the game states until the end of the game. Each
    /// iteration returns a PlayResult which describes the current game
    /// state with the relevant information.
    pub fn do_next_play(&mut self) -> Option<PlayResult> {
        /* Check to see if anyone has won yet. If so, end the game.
         *
         * Note: We might need to do this differently later on.
         */
        for (player, score) in self.scores.iter() {
            if *score > 120 {
                self.state = GameState::Done;

                let normalized_scores = self
                    .scores
                    .iter()
                    .map(|(p, s)| if *s > 121 { (*p, 121) } else { (*p, *s) })
                    .collect::<HashMap<PlayerPosition, u8>>();

                return Some(PlayResult::GameOver(FinalScores {
                    winner: *player,
                    scores: normalized_scores,
                }));
            }
        }

        let (new_state, result) = match self.state {
            GameState::ChooseDealer => {
                let (up_cards, dealer) = choose_dealer(&mut self.deck);
                let next_result = CutInfo { up_cards, dealer };
                self.dealer = dealer;
                (
                    GameState::DealCards,
                    Some(PlayResult::DealerChosen(next_result)),
                )
            }
            GameState::DealCards => {
                let hands = deal_cards(&mut self.deck);
                self.hands = hands;
                let next_result = Deal {
                    dealer: self.dealer,
                    scores: self.scores.clone(),
                    hands: self.hands.clone(),
                };
                (
                    GameState::ChooseCrib,
                    Some(PlayResult::WaitingForCrib(next_result)),
                )
            }
            GameState::ChooseCrib => {
                let choices = choose_crib(&mut self.players, &self.hands);
                for (position, choice) in choices {
                    let hand = self.hands.get_mut(&position).unwrap();
                    for card in choice {
                        hand.remove(&card);
                    }
                }
                let up_card = self.deck.deal(1);
                self.up_card = Some(up_card[0]);
                if self.up_card.unwrap().rank == Rank::Jack {
                    let score = self.scores.get_mut(&self.dealer).unwrap();
                    *score += 2;
                }
                let next_result = UpCardInfo {
                    dealer: self.dealer,
                    scores: self.scores.clone(),
                    up_card: up_card[0],
                    is_nibs: self.up_card.unwrap().rank == Rank::Jack,
                };
                (
                    GameState::Done,
                    Some(PlayResult::AnnounceUpCard(next_result)),
                )
            }
            _ => (GameState::Done, None),
        };
        self.state = new_state;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::deck::{Card, Deck};
    use crate::simulation::player::SimplePlayer;

    impl<'a, P: KnowsCribbage> GameRunner<'a, P> {
        fn new_with_deck_and_state(
            first: &'a mut P,
            second: &'a mut P,
            deck: Deck,
            state: GameState,
        ) -> Self {
            let mut temp = Self::new_with_deck(first, second, deck);
            temp.state = state;
            temp
        }

        fn set_hand(&mut self, position: PlayerPosition, hand: &[Card]) {
            self.hands
                .insert(position, HashSet::from_iter(hand.iter().copied()));
        }
    }

    #[test]
    fn test_selects_right_dealer() {
        for (cards, expected_dealer) in [
            ([Card::from("as"), Card::from("ks")], PlayerPosition::First),
            ([Card::from("ks"), Card::from("as")], PlayerPosition::Second),
        ] {
            let deck_cards = Vec::from(cards);
            let deck = Deck::stacked(deck_cards);
            let mut first = SimplePlayer {};
            let mut second = SimplePlayer {};
            let mut runner = GameRunner::new_with_deck(&mut first, &mut second, deck);
            let result = runner.do_next_play().unwrap();
            match result {
                PlayResult::DealerChosen(info) => {
                    assert_eq!(info.dealer, expected_dealer);
                    assert_eq!(
                        *info.up_cards.get(&PlayerPosition::First).unwrap(),
                        cards[0]
                    );
                    assert_eq!(
                        *info.up_cards.get(&PlayerPosition::Second).unwrap(),
                        cards[1]
                    );
                }
                _ => panic!("Wrong play result {:?}", result),
            }
        }
    }

    #[test]
    fn test_does_not_entertain_a_tie() {
        let cards = [
            Card::from("as"),
            Card::from("ah"),
            Card::from("ac"),
            Card::from("kc"),
        ];
        let deck_cards = Vec::from(cards);
        let deck = Deck::stacked(deck_cards);
        let mut first = SimplePlayer {};
        let mut second = SimplePlayer {};
        let mut runner = GameRunner::new_with_deck(&mut first, &mut second, deck);
        let result = runner.do_next_play().unwrap();
        match result {
            PlayResult::DealerChosen(info) => {
                assert_eq!(info.dealer, PlayerPosition::First);
                assert_eq!(
                    *info.up_cards.get(&PlayerPosition::First).unwrap(),
                    cards[2]
                );
                assert_eq!(
                    *info.up_cards.get(&PlayerPosition::Second).unwrap(),
                    cards[3]
                );
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_deals_cards() {
        let cards = [
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
        ];
        let deck_cards = Vec::from(cards);
        let deck = Deck::stacked(deck_cards);
        let state = GameState::DealCards;
        let mut first = SimplePlayer {};
        let mut second = SimplePlayer {};
        let mut runner = GameRunner::new_with_deck_and_state(&mut first, &mut second, deck, state);

        let result = runner.do_next_play().unwrap();
        match result {
            PlayResult::WaitingForCrib(info) => {
                assert_eq!(info.dealer, PlayerPosition::First);

                let hand = info.hands.get(&PlayerPosition::First).unwrap();
                let expected_hand = HashSet::from_iter(cards[0..6].iter().copied());
                assert_eq!(0, hand.symmetric_difference(&expected_hand).count());

                let hand = info.hands.get(&PlayerPosition::Second).unwrap();
                let expected_hand = HashSet::from_iter(cards[6..].iter().copied());
                assert_eq!(0, hand.symmetric_difference(&expected_hand).count());

                let score = info.scores.get(&PlayerPosition::First).unwrap();
                assert_eq!(0, *score);

                let score = info.scores.get(&PlayerPosition::Second).unwrap();
                assert_eq!(0, *score);
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_gets_crib_choices() {
        for (up_card, is_nibs) in [
            (Card::from("ah"), false),
            (Card::from("js"), true),
            (Card::from("jh"), true),
            (Card::from("jc"), true),
            (Card::from("jd"), true),
        ] {
            let cards = [up_card];
            let deck_cards = Vec::from(cards);
            let deck = Deck::stacked(deck_cards);
            let state = GameState::ChooseCrib;
            let mut first = SimplePlayer {};
            let mut second = SimplePlayer {};
            let mut runner =
                GameRunner::new_with_deck_and_state(&mut first, &mut second, deck, state);

            runner.set_hand(
                PlayerPosition::First,
                &[
                    Card::from("as"),
                    Card::from("2s"),
                    Card::from("3s"),
                    Card::from("4s"),
                    Card::from("5s"),
                    Card::from("6s"),
                ],
            );
            runner.set_hand(
                PlayerPosition::Second,
                &[
                    Card::from("ac"),
                    Card::from("2c"),
                    Card::from("3c"),
                    Card::from("4c"),
                    Card::from("5c"),
                    Card::from("6c"),
                ],
            );

            let result = runner.do_next_play().unwrap();
            match result {
                PlayResult::AnnounceUpCard(info) => {
                    assert_eq!(info.dealer, PlayerPosition::First);

                    let score = info.scores.get(&PlayerPosition::First).unwrap();
                    if is_nibs {
                        assert_eq!(2, *score);
                    } else {
                        assert_eq!(0, *score);
                    }

                    let score = info.scores.get(&PlayerPosition::Second).unwrap();
                    assert_eq!(0, *score);

                    assert_eq!(cards[0], info.up_card);
                    assert_eq!(is_nibs, info.is_nibs);
                }
                _ => panic!("Wrong play result {:?}", result),
            }
        }
    }

    #[test]
    fn test_ends_game_if_nibs_scores_enough() {
        let cards = [Card::from("js")];
        let deck_cards = Vec::from(cards);
        let deck = Deck::stacked(deck_cards);
        let state = GameState::ChooseCrib;
        let mut first = SimplePlayer {};
        let mut second = SimplePlayer {};
        let mut runner = GameRunner::new_with_deck_and_state(&mut first, &mut second, deck, state);

        runner.set_hand(
            PlayerPosition::First,
            &[
                Card::from("as"),
                Card::from("2s"),
                Card::from("3s"),
                Card::from("4s"),
                Card::from("5s"),
                Card::from("6s"),
            ],
        );
        runner.set_hand(
            PlayerPosition::Second,
            &[
                Card::from("ac"),
                Card::from("2c"),
                Card::from("3c"),
                Card::from("4c"),
                Card::from("5c"),
                Card::from("6c"),
            ],
        );

        let score = runner.scores.get_mut(&PlayerPosition::First).unwrap();
        *score = 120;

        runner.do_next_play(); // Announce the up card
        let result = runner.do_next_play().unwrap();
        match result {
            PlayResult::GameOver(info) => {
                assert_eq!(info.winner, PlayerPosition::First);
                assert_eq!(121, *info.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *info.scores.get(&PlayerPosition::Second).unwrap());
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }
}

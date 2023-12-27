use crate::simulation::deck::{Card, Deck, Rank};
use crate::simulation::player::{KnowsCribbage, PlayerPosition};
use crate::simulation::scoring::{
    score_crib, score_hand, score_pegging, HandScorings, PeggingScorings,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
enum AcknowledgementType {
    Scoring,
    FirstGo,
    SecondGo,
}

#[derive(Copy, Clone)]
enum GameState {
    ChooseDealer,
    DealCards,
    ChooseCrib,
    PrepareForPlay,
    WaitingForPlay,
    WaitingForAcknowledgement(AcknowledgementType),
    ScoreHand,
    ScoreDealer,
    ScoreCrib,
    Done,
}

pub struct GameRunner<'a, P: KnowsCribbage> {
    /// The main game runner. Simulates all the rules of cribbage
    /// in the right sequence.
    state: GameState,
    dealer: PlayerPosition,
    deck: Deck,
    dealt_hands: HashMap<PlayerPosition, HashSet<Card>>,
    hands: HashMap<PlayerPosition, HashSet<Card>>,
    scores: HashMap<PlayerPosition, u8>,
    players: HashMap<PlayerPosition, &'a mut P>,
    up_card: Option<Card>,
    crib: Vec<Card>,
    current_player: PlayerPosition,
    played_cards: Vec<Card>,
    last_card_player: Option<PlayerPosition>,
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
pub struct PlayInfo {
    pub played_cards: Vec<Card>,
    pub hands: HashMap<PlayerPosition, HashSet<Card>>,
    pub scores: HashMap<PlayerPosition, u8>,
    pub current_player: PlayerPosition,
}

#[derive(Debug)]
pub struct FinalScores {
    pub winner: PlayerPosition,
    pub scores: HashMap<PlayerPosition, u8>,
}

#[derive(Debug)]
pub struct ScoreInfo {
    pub played_cards: Vec<Card>,
    pub hands: HashMap<PlayerPosition, HashSet<Card>>,
    pub scores: HashMap<PlayerPosition, u8>,
    pub current_player: PlayerPosition,
    pub scorings: Vec<PeggingScorings>,
}

#[derive(Debug)]
pub struct HandInfo {
    pub scores: HashMap<PlayerPosition, u8>,
    pub current_player: PlayerPosition,
    pub scorings: Vec<HandScorings>,
}

#[derive(Debug)]
pub struct CurrentScoreInfo {
    pub scores: HashMap<PlayerPosition, u8>,
    pub next_dealer: PlayerPosition,
}

#[derive(Debug)]
pub enum PlayResult {
    DealerChosen(CutInfo),
    WaitingForCrib(Deal),
    AnnounceUpCard(UpCardInfo),
    GameOver(FinalScores),
    ReadyForPlay(PlayInfo),
    AnnounceScoring(ScoreInfo),
    AnnounceGo(PlayInfo),
    AnnounceHandScore(HandInfo),
    AnnounceDealerScore(HandInfo),
    AnnounceCribScore(HandInfo),
    NextHand(CurrentScoreInfo),
}

fn has_play(hand: &HashSet<Card>, played: &[Card]) -> bool {
    let current_total = played.iter().map(|c| c.rank.value()).sum::<u8>();
    for card in hand.iter() {
        if card.rank.value() + current_total <= 31 {
            return true;
        }
    }
    false
}

impl<'a, P: KnowsCribbage> GameRunner<'a, P> {
    pub fn new(first: &'a mut P, second: &'a mut P) -> Self {
        let deck = Deck::new();

        let mut hands = HashMap::new();
        hands.insert(PlayerPosition::First, HashSet::new());
        hands.insert(PlayerPosition::Second, HashSet::new());

        let mut dealt_hands = HashMap::new();
        dealt_hands.insert(PlayerPosition::First, HashSet::new());
        dealt_hands.insert(PlayerPosition::Second, HashSet::new());

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
            dealt_hands,
            scores,
            players,
            dealer: PlayerPosition::First,
            up_card: None,
            current_player: PlayerPosition::Second,
            played_cards: Vec::new(),
            last_card_player: None,
            crib: Vec::new(),
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
                /* Shuffle the deck and choose two cards */
                self.deck.shuffle();
                let mut cards = self.deck.deal(2);

                /* Don't bother entertaining a tie. Just redraw. */
                while cards[0].rank == cards[1].rank {
                    cards = self.deck.deal(2);
                };

                /* Dealer is the one who chose the lowest card. Assume
                 * first card is for the first player.
                 */
                self.dealer = match cards[0].rank < cards[1].rank {
                    true => PlayerPosition::First,
                    false => PlayerPosition::Second,
                };
                let mut up_cards = HashMap::new();
                up_cards.insert(PlayerPosition::First, cards[0]);
                up_cards.insert(PlayerPosition::Second, cards[1]);
                let next_result = CutInfo { up_cards, dealer: self.dealer };
                self.current_player = self.dealer.next();
                (
                    GameState::DealCards,
                    Some(PlayResult::DealerChosen(next_result)),
                )
            },
            GameState::DealCards => {
                self.hands.clear();
                self.dealt_hands.clear();
                self.deck.shuffle();
                self.hands.insert(
                    PlayerPosition::First,
                    HashSet::from_iter(self.deck.deal(6).iter().copied()),
                );
                self.dealt_hands.insert(
                    PlayerPosition::First,
                    HashSet::from_iter(self.hands.get(&PlayerPosition::First).unwrap().iter().copied()),
                );
                self.hands.insert(
                    PlayerPosition::Second,
                    HashSet::from_iter(self.deck.deal(6).iter().copied()),
                );
                self.dealt_hands.insert(
                    PlayerPosition::Second,
                    HashSet::from_iter(self.hands.get(&PlayerPosition::Second).unwrap().iter().copied()),
                );
                self.up_card = self.deck.deal(1).first().copied();
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
                self.crib.clear();
                let mut choices = HashMap::new();
                for position in [PlayerPosition::First, PlayerPosition::Second] {
                    let player = self.players.get_mut(&position).unwrap();
                    let hand = self.hands.get(&position).unwrap();
                    choices.insert(position, player.choose_crib(hand));
                }
                for (position, choice) in choices {
                    let hand = self.hands.get_mut(&position).unwrap();
                    for card in choice {
                        hand.remove(&card);
                        self.crib.push(card);
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
                    GameState::PrepareForPlay,
                    Some(PlayResult::AnnounceUpCard(next_result)),
                )
            }
            GameState::PrepareForPlay => {
                self.played_cards = Vec::new();
                let next_result = PlayInfo {
                    played_cards: self.played_cards.clone(),
                    hands: self.hands.clone(),
                    scores: self.scores.clone(),
                    current_player: self.current_player,
                };
                (
                    GameState::WaitingForPlay,
                    Some(PlayResult::ReadyForPlay(next_result)),
                )
            }
            GameState::WaitingForPlay => {
                let current_player = self.players.get_mut(&self.current_player).unwrap();
                let hand = self.hands.get_mut(&self.current_player).unwrap();
                let choice = current_player.play(hand);
                self.played_cards.push(choice);
                hand.remove(&choice);
                self.last_card_player = Some(self.current_player);

                let scorings = score_pegging(self.played_cards.to_vec());
                match !scorings.is_empty() {
                    true => {
                        let player_score = self.scores.get_mut(&self.current_player).unwrap();
                        for scoring in scorings.iter() {
                            *player_score += scoring.value();
                        }

                        let next_result = ScoreInfo {
                            played_cards: self.played_cards.clone(),
                            hands: self.hands.clone(),
                            scores: self.scores.clone(),
                            current_player: self.current_player,
                            scorings,
                        };
                        (
                            GameState::WaitingForAcknowledgement(AcknowledgementType::Scoring),
                            Some(PlayResult::AnnounceScoring(next_result)),
                        )
                    }
                    false => {
                        self.current_player = self.current_player.next();

                        let hand = self.hands.get(&self.current_player).unwrap();
                        let next_result = PlayInfo {
                            played_cards: self.played_cards.clone(),
                            hands: self.hands.clone(),
                            scores: self.scores.clone(),
                            current_player: self.current_player,
                        };

                        match has_play(hand, &self.played_cards) {
                            true => (
                                GameState::WaitingForPlay,
                                Some(PlayResult::ReadyForPlay(next_result)),
                            ),
                            false => (
                                GameState::WaitingForAcknowledgement(AcknowledgementType::FirstGo),
                                Some(PlayResult::AnnounceGo(next_result)),
                            ),
                        }
                    }
                }
            }
            GameState::WaitingForAcknowledgement(ack_type) => match ack_type {
                AcknowledgementType::SecondGo => {
                    self.played_cards = Vec::new();
                    let scorings = [PeggingScorings::LastCard].to_vec();
                    let player_score = self.scores.get_mut(&self.current_player).unwrap();
                    for scoring in scorings.iter() {
                        *player_score += scoring.value();
                    }

                    let next_result = ScoreInfo {
                        played_cards: self.played_cards.clone(),
                        hands: self.hands.clone(),
                        scores: self.scores.clone(),
                        current_player: self.current_player,
                        scorings,
                    };
                    (
                        GameState::WaitingForAcknowledgement(AcknowledgementType::Scoring),
                        Some(PlayResult::AnnounceScoring(next_result)),
                    )
                }
                _ => {
                    let cards_left = self.hands.values().map(|hand| hand.len()).sum();
                    match cards_left {
                        0 => {
                            self.current_player = self.dealer.next();
                            let hand = self
                                .dealt_hands
                                .get(&self.current_player)
                                .unwrap()
                                .iter()
                                .copied()
                                .collect::<Vec<Card>>();
                            let scorings = score_hand(hand, self.up_card.unwrap());
                            let player_score = self.scores.get_mut(&self.current_player).unwrap();
                            for scoring in scorings.iter() {
                                *player_score += scoring.value();
                            }
                            let hand_info = HandInfo {
                                scores: self.scores.clone(),
                                current_player: self.current_player,
                                scorings: scorings.clone(),
                            };

                            (
                                GameState::ScoreHand,
                                Some(PlayResult::AnnounceHandScore(hand_info)),
                            )
                        }
                        _ => {
                            if self.played_cards.iter().map(|c| c.rank.value()).sum::<u8>() == 31 {
                                self.played_cards = Vec::new();
                            }
                            self.current_player = self.current_player.next();
                            let next_result = PlayInfo {
                                played_cards: self.played_cards.clone(),
                                hands: self.hands.clone(),
                                scores: self.scores.clone(),
                                current_player: self.current_player,
                            };

                            let hand = self.hands.get(&self.current_player).unwrap();
                            match has_play(hand, &self.played_cards) || self.played_cards.is_empty()
                            {
                                true => (
                                    GameState::WaitingForPlay,
                                    Some(PlayResult::ReadyForPlay(next_result)),
                                ),
                                false => {
                                    let next_type = match ack_type {
                                        AcknowledgementType::FirstGo => {
                                            AcknowledgementType::SecondGo
                                        }
                                        _ => AcknowledgementType::FirstGo,
                                    };
                                    (
                                        GameState::WaitingForAcknowledgement(next_type),
                                        Some(PlayResult::AnnounceGo(next_result)),
                                    )
                                }
                            }
                        }
                    }
                }
            },
            GameState::ScoreHand => {
                let hand = self
                    .dealt_hands
                    .get(&self.dealer)
                    .unwrap()
                    .iter()
                    .copied()
                    .collect::<Vec<Card>>();
                let scorings = score_hand(hand, self.up_card.unwrap());
                let player_score = self.scores.get_mut(&self.dealer).unwrap();
                for scoring in scorings.iter() {
                    *player_score += scoring.value();
                }
                let hand_info = HandInfo {
                    scores: self.scores.clone(),
                    current_player: self.dealer,
                    scorings: scorings.clone(),
                };
                (
                    GameState::ScoreDealer,
                    Some(PlayResult::AnnounceDealerScore(hand_info)),
                )
            }
            GameState::ScoreDealer => {
                let crib = self.crib.to_vec();
                let scorings = score_crib(crib, self.up_card.unwrap());
                let player_score = self.scores.get_mut(&self.dealer).unwrap();
                for scoring in scorings.iter() {
                    *player_score += scoring.value();
                }
                let hand_info = HandInfo {
                    scores: self.scores.clone(),
                    current_player: self.dealer,
                    scorings: scorings.clone(),
                };
                (
                    GameState::ScoreCrib,
                    Some(PlayResult::AnnounceCribScore(hand_info)),
                )
            }
            GameState::ScoreCrib => {
                self.dealer = self.dealer.next();
                self.current_player = self.dealer.next();
                let current_info = CurrentScoreInfo {
                    scores: self.scores.clone(),
                    next_dealer: self.dealer,
                };
                (
                    GameState::DealCards,
                    Some(PlayResult::NextHand(current_info)),
                )
            }
            _ => (GameState::WaitingForPlay, None),
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

    struct TestFixture {
        hands: [HashSet<Card>; 2],
        dealt_hands: [HashSet<Card>; 2],
        deck: Vec<Card>,
        played_cards: Vec<Card>,
        state: GameState,
        scores: [u8; 2],
        crib: Vec<Card>,
    }

    impl TestFixture {
        fn new() -> Self {
            Self {
                hands: [HashSet::new(), HashSet::new()],
                dealt_hands: [HashSet::new(), HashSet::new()],
                deck: Vec::new(),
                played_cards: Vec::new(),
                state: GameState::ChooseDealer,
                scores: [0, 0],
                crib: Vec::new(),
            }
        }

        fn stack_deck(&mut self, cards: &[Card]) -> () {
            self.deck = Vec::from(cards);
        }

        fn set_state(&mut self, state: GameState) -> () {
            self.state = state;
        }

        fn set_hand(&mut self, position: PlayerPosition, hand: &[Card]) -> () {
            match position {
                PlayerPosition::First => self.hands[0].extend(hand),
                PlayerPosition::Second => self.hands[1].extend(hand),
            }
        }

        fn set_dealt_hand(&mut self, position: PlayerPosition, hand: &[Card]) -> () {
            match position {
                PlayerPosition::First => self.dealt_hands[0].extend(hand),
                PlayerPosition::Second => self.dealt_hands[1].extend(hand),
            }
        }

        fn set_score(&mut self, position: PlayerPosition, score: u8) -> () {
            match position {
                PlayerPosition::First => self.scores[0] = score,
                PlayerPosition::Second => self.scores[1] = score,
            }
        }

        fn set_played_cards(&mut self, cards: &[Card]) -> () {
            self.played_cards = Vec::from(cards);
        }

        fn set_crib(&mut self, cards: &[Card]) -> () {
            self.crib = Vec::from(cards);
        }

        fn do_iterations(&mut self, num: usize) -> PlayResult {
            let mut result: Option<PlayResult> = None;
            let mut first = SimplePlayer {};
            let mut second = SimplePlayer {};
            let mut runner = GameRunner::new(&mut first, &mut second);
            runner.deck = Deck::stacked(self.deck.iter().copied().collect());
            runner.state = self.state;
            runner.played_cards = self.played_cards.iter().copied().collect();
            runner.hands = HashMap::from([
                (
                    PlayerPosition::First,
                    self.hands[0].iter().copied().collect(),
                ),
                (
                    PlayerPosition::Second,
                    self.hands[1].iter().copied().collect(),
                ),
            ]);
            runner.dealt_hands = HashMap::from([
                (
                    PlayerPosition::First,
                    self.dealt_hands[0].iter().copied().collect(),
                ),
                (
                    PlayerPosition::Second,
                    self.dealt_hands[1].iter().copied().collect(),
                ),
            ]);
            runner.scores = HashMap::from([
                (PlayerPosition::First, self.scores[0]),
                (PlayerPosition::Second, self.scores[1]),
            ]);
            runner.up_card = Some(Card::from("ad"));
            runner.crib = self.crib.iter().copied().collect();
            for _ in 0..num {
                result = runner.do_next_play();
            }
            result.unwrap()
        }
    }

    #[test]
    fn test_selects_right_dealer() {
        for (cards, expected_dealer) in [
            ([Card::from("as"), Card::from("ks")], PlayerPosition::First),
            ([Card::from("ks"), Card::from("as")], PlayerPosition::Second),
        ] {
            let mut fixture = TestFixture::new();
            fixture.stack_deck(&cards);
            let result = fixture.do_iterations(1);
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
        let mut fixture = TestFixture::new();
        fixture.stack_deck(&cards);
        let result = fixture.do_iterations(1);
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
            Card::from("ad"),
        ];
        let mut fixture = TestFixture::new();
        fixture.stack_deck(&cards);
        fixture.set_state(GameState::DealCards);
        let result = fixture.do_iterations(1);
        match result {
            PlayResult::WaitingForCrib(info) => {
                assert_eq!(info.dealer, PlayerPosition::First);

                let hand = info.hands.get(&PlayerPosition::First).unwrap();
                let expected_hand = HashSet::from_iter(cards[0..6].iter().copied());
                assert_eq!(0, hand.symmetric_difference(&expected_hand).count());

                let hand = info.hands.get(&PlayerPosition::Second).unwrap();
                let expected_hand = HashSet::from_iter(cards[6..12].iter().copied());
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
            let mut fixture = TestFixture::new();
            fixture.stack_deck(&[up_card]);
            fixture.set_state(GameState::ChooseCrib);
            fixture.set_hand(
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
            fixture.set_hand(
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

            let result = fixture.do_iterations(1);
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

                    assert_eq!(up_card, info.up_card);
                    assert_eq!(is_nibs, info.is_nibs);
                }
                _ => panic!("Wrong play result {:?}", result),
            }
        }
    }

    #[test]
    fn test_ends_game_if_nibs_scores_enough() {
        let mut fixture = TestFixture::new();
        fixture.stack_deck(&[Card::from("js")]);
        fixture.set_state(GameState::ChooseCrib);
        fixture.set_hand(
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
        fixture.set_hand(
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
        fixture.set_score(PlayerPosition::First, 120);

        let result = fixture.do_iterations(2);
        match result {
            PlayResult::GameOver(info) => {
                assert_eq!(info.winner, PlayerPosition::First);
                assert_eq!(121, *info.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *info.scores.get(&PlayerPosition::Second).unwrap());
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_begins_pegging() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::PrepareForPlay);
        let first_hand = [
            Card::from("as"),
            Card::from("2s"),
            Card::from("3s"),
            Card::from("4s"),
        ];
        let second_hand = [
            Card::from("ac"),
            Card::from("2c"),
            Card::from("3c"),
            Card::from("4c"),
        ];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);

        let result = fixture.do_iterations(1);
        match result {
            PlayResult::ReadyForPlay(info) => {
                assert_eq!(0, info.played_cards.len());
                assert_eq!(
                    HashSet::from_iter(first_hand.into_iter()),
                    *info.hands.get(&PlayerPosition::First).unwrap()
                );
                assert_eq!(
                    HashSet::from_iter(second_hand.into_iter()),
                    *info.hands.get(&PlayerPosition::Second).unwrap()
                );
                assert_eq!(0, *info.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *info.scores.get(&PlayerPosition::Second).unwrap());
                assert_eq!(PlayerPosition::Second, info.current_player);
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_keeps_track_of_played_cards() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::WaitingForPlay);
        let first_hand = [
            Card::from("as"),
            Card::from("3s"),
            Card::from("6s"),
            Card::from("9s"),
        ];
        let second_hand = [
            Card::from("5c"),
            Card::from("jc"),
            Card::from("qd"),
            Card::from("kc"),
        ];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);

        /* Play four cards */
        let result = fixture.do_iterations(4);

        match result {
            PlayResult::ReadyForPlay(info) => {
                assert_eq!(4, info.played_cards.len(), "{:?}", info.played_cards);
                assert_eq!(
                    HashSet::from_iter(first_hand[2..].into_iter().copied()),
                    *info.hands.get(&PlayerPosition::First).unwrap()
                );
                assert_eq!(
                    HashSet::from_iter(second_hand[2..].into_iter().copied()),
                    *info.hands.get(&PlayerPosition::Second).unwrap()
                );
                assert_eq!(PlayerPosition::Second, info.current_player);
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_counts_score() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::WaitingForPlay);
        let first_hand = [Card::from("5s")];
        let second_hand = [Card::from("kc")];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);

        /* Second player will play a king, first player (dealer) will
         * play a 5, scoring 2 points.
         */
        let result = fixture.do_iterations(2);

        match result {
            PlayResult::AnnounceScoring(info) => {
                assert_eq!(2, *info.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *info.scores.get(&PlayerPosition::Second).unwrap());
                assert_eq!(1, info.scorings.len());
                assert_eq!(PeggingScorings::Fifteen, info.scorings[0]);
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_counts_thirty_one() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::WaitingForPlay);
        let first_hand = [Card::from("ts"), Card::from("qs")];
        let second_hand = [Card::from("as"), Card::from("kc")];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);

        /* Second player will play an ace.
         * First player (dealer) will play a ten.
         * Second player will play a king.
         * First player will play a queen, hitting 31 and getting 2 points.
         */
        let result = fixture.do_iterations(4);

        match result {
            PlayResult::AnnounceScoring(info) => {
                assert_eq!(2, *info.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *info.scores.get(&PlayerPosition::Second).unwrap());
                assert_eq!(1, info.scorings.len());
                assert_eq!(PeggingScorings::ThirtyOne, info.scorings[0]);
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn resets_after_thirty_one() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::WaitingForPlay);
        let first_hand = [Card::from("ts"), Card::from("qs")];
        let second_hand = [Card::from("as"), Card::from("kc"), Card::from("qc")];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);

        /* Second player will play an ace.
         * First player (dealer) will play a ten.
         * Second player will play a king.
         * First player will play a queen, hitting 31 and getting 2 points.
         * This should clear the played cards.
         */
        let result = fixture.do_iterations(5);

        match result {
            PlayResult::ReadyForPlay(info) => {
                assert_eq!(0, info.played_cards.len());
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_announces_go() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::WaitingForPlay);
        let first_hand = [Card::from("ts"), Card::from("qs")];
        let second_hand = [Card::from("2s"), Card::from("kc")];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);

        /* Second player will play a two.
         * First player (dealer) will play a ten.
         * Second player will play a king.
         * First player must say go.
         */
        let result = fixture.do_iterations(3);

        match result {
            PlayResult::AnnounceGo(info) => {
                assert_eq!(3, info.played_cards.len());
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_announces_go_for_both_players() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::WaitingForPlay);
        let first_hand = [Card::from("ts"), Card::from("qs")];
        let second_hand = [Card::from("2s"), Card::from("kc")];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);

        /* Second player will play a two.
         * First player (dealer) will play a ten.
         * Second player will play a king.
         * First player must say go.
         * Second player must say go, as they have no cards left.
         */
        let result = fixture.do_iterations(4);

        match result {
            PlayResult::AnnounceGo(info) => {
                assert_eq!(3, info.played_cards.len());
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_gives_point_for_last_card() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::WaitingForPlay);
        let first_hand = [Card::from("ts"), Card::from("qs")];
        let second_hand = [Card::from("2s"), Card::from("kc")];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);
        fixture.set_played_cards(&[Card::from("4s")]);

        /* Second player will play a two.
         * First player (dealer) will play a ten.
         * Second player will play a king.
         * First player must say go.
         * Second player must say go, as they have no cards left.
         * Second player gets a point for last card.
         */
        let result = fixture.do_iterations(5);

        match result {
            PlayResult::AnnounceScoring(info) => {
                assert_eq!(1, info.scorings.len());
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_resets_after_last_card() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::WaitingForAcknowledgement(
            AcknowledgementType::SecondGo,
        ));
        let first_hand = [Card::from("ts"), Card::from("qs")];
        let second_hand = [Card::from("2s"), Card::from("kc")];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);

        /* Second player has said go.
         * Scoring acknowledged.
         * There should be no cards played.
         */
        let result = fixture.do_iterations(2);

        match result {
            PlayResult::ReadyForPlay(info) => {
                assert_eq!(0, info.played_cards.len());
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_player_can_win_on_last_card() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::WaitingForAcknowledgement(
            AcknowledgementType::SecondGo,
        ));
        let first_hand = [Card::from("ts"), Card::from("qs")];
        let second_hand = [Card::from("2s"), Card::from("kc")];
        fixture.set_hand(PlayerPosition::First, &first_hand);
        fixture.set_hand(PlayerPosition::Second, &second_hand);
        fixture.set_score(PlayerPosition::Second, 120);

        /* Second player has said go.
         * Scoring acknowledged.
         * Player has 121 points, so wins.
         */
        let result = fixture.do_iterations(3);

        match result {
            PlayResult::GameOver(info) => {
                assert_eq!(PlayerPosition::Second, info.winner);
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_scores_hand_when_no_pegging_play_left() {
        let mut fixture = TestFixture::new();
        let first_hand = [
            Card::from("as"),
            Card::from("2s"),
            Card::from("3s"),
            Card::from("4s"),
        ];
        let second_hand = [
            Card::from("ac"),
            Card::from("2c"),
            Card::from("3c"),
            Card::from("4c"),
        ];
        fixture.set_dealt_hand(PlayerPosition::First, &first_hand);
        fixture.set_dealt_hand(PlayerPosition::Second, &second_hand);
        fixture.set_state(GameState::WaitingForAcknowledgement(
            AcknowledgementType::SecondGo,
        ));

        /* Second player has said go.
         * Scoring acknowledged.
         * No cards left in either hand, so start scoring hands.
         */
        let result = fixture.do_iterations(2);

        match result {
            PlayResult::AnnounceHandScore(_) => {}
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_scores_dealer_after_scoring_hand() {
        let mut fixture = TestFixture::new();
        let first_hand = [
            Card::from("as"),
            Card::from("2s"),
            Card::from("3s"),
            Card::from("4s"),
        ];
        let second_hand = [
            Card::from("ac"),
            Card::from("2c"),
            Card::from("3c"),
            Card::from("4c"),
        ];
        fixture.set_dealt_hand(PlayerPosition::First, &first_hand);
        fixture.set_dealt_hand(PlayerPosition::Second, &second_hand);
        fixture.set_state(GameState::ScoreHand);

        let result = fixture.do_iterations(1);

        match result {
            PlayResult::AnnounceDealerScore(_) => {}
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_player_can_win_after_scoring_hand() {
        let mut fixture = TestFixture::new();
        let first_hand = [
            Card::from("as"),
            Card::from("2s"),
            Card::from("3s"),
            Card::from("4s"),
        ];
        let second_hand = [
            Card::from("ac"),
            Card::from("2c"),
            Card::from("3c"),
            Card::from("4c"),
        ];
        fixture.set_dealt_hand(PlayerPosition::First, &first_hand);
        fixture.set_dealt_hand(PlayerPosition::Second, &second_hand);
        fixture.set_score(PlayerPosition::First, 121);
        fixture.set_state(GameState::ScoreHand);

        let result = fixture.do_iterations(1);

        match result {
            PlayResult::GameOver(info) => {
                assert_eq!(PlayerPosition::First, info.winner);
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_scores_crib_after_scoring_dealer() {
        let mut fixture = TestFixture::new();
        let crib = [
            Card::from("as"),
            Card::from("2s"),
            Card::from("3s"),
            Card::from("4s"),
        ];
        fixture.set_crib(&crib);
        fixture.set_state(GameState::ScoreDealer);

        let result = fixture.do_iterations(1);

        match result {
            PlayResult::AnnounceCribScore(_) => {}
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_dealer_can_win_after_scoring_dealer() {
        let mut fixture = TestFixture::new();
        let first_hand = [
            Card::from("as"),
            Card::from("2s"),
            Card::from("3s"),
            Card::from("4s"),
        ];
        let second_hand = [
            Card::from("ac"),
            Card::from("2c"),
            Card::from("3c"),
            Card::from("4c"),
        ];
        fixture.set_dealt_hand(PlayerPosition::First, &first_hand);
        fixture.set_dealt_hand(PlayerPosition::Second, &second_hand);
        fixture.set_score(PlayerPosition::Second, 121);
        fixture.set_state(GameState::ScoreDealer);

        let result = fixture.do_iterations(1);

        match result {
            PlayResult::GameOver(info) => {
                assert_eq!(PlayerPosition::Second, info.winner);
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_redeals_after_scoring_crib() {
        let mut fixture = TestFixture::new();
        fixture.set_state(GameState::ScoreCrib);

        let result = fixture.do_iterations(1);

        match result {
            PlayResult::NextHand(_) => {}
            _ => panic!("Wrong play result {:?}", result),
        }
    }

    #[test]
    fn test_dealer_can_win_after_scoring_crib() {
        let mut fixture = TestFixture::new();
        let first_hand = [
            Card::from("as"),
            Card::from("2s"),
            Card::from("3s"),
            Card::from("4s"),
        ];
        let second_hand = [
            Card::from("ac"),
            Card::from("2c"),
            Card::from("3c"),
            Card::from("4c"),
        ];
        fixture.set_dealt_hand(PlayerPosition::First, &first_hand);
        fixture.set_dealt_hand(PlayerPosition::Second, &second_hand);
        fixture.set_score(PlayerPosition::Second, 121);
        fixture.set_state(GameState::ScoreCrib);

        let result = fixture.do_iterations(1);

        match result {
            PlayResult::GameOver(info) => {
                assert_eq!(PlayerPosition::Second, info.winner);
            }
            _ => panic!("Wrong play result {:?}", result),
        }
    }
}

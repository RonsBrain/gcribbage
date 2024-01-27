use crate::deck::{Card, Deck, Rank};
use crate::player::{KnowsCribbage, PlayerPosition};
use crate::scoring::{score_hand, score_pegging, HandScorings, PeggingScorings};
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
struct ChooseDealer;
#[derive(Debug, PartialEq)]
struct Deal;
#[derive(Debug, PartialEq)]
struct ChooseCrib;
#[derive(Debug, PartialEq)]
struct TurnUpcard;
#[derive(Debug, PartialEq)]
struct Pegging;
#[derive(Debug, PartialEq)]
struct GameOver;
#[derive(Debug, PartialEq)]
struct FirstGo;
#[derive(Debug, PartialEq)]
struct SecondGo;
#[derive(Debug, PartialEq)]
struct ResumePeggingOrScoreHands;
#[derive(Debug, PartialEq)]
struct ScoreDealer;

enum GameState {
    New(Rule<ChooseDealer>),
    ReadyToDeal(Rule<Deal>),
    WaitingForCrib(Rule<ChooseCrib>),
    TurningUpCard(Rule<TurnUpcard>),
    Pegging(Rule<Pegging>),
    GameOver(Rule<GameOver>),
    AnnouncingFirstGo(Rule<FirstGo>),
    AnnouncingSecondGo(Rule<SecondGo>),
    ScoringLastCard(Rule<ResumePeggingOrScoreHands>),
    ScoringDealer(Rule<ScoreDealer>),
}

struct GameComponents<'p> {
    deck: Deck,
    hands: HashMap<PlayerPosition, HashSet<Card>>,
    dealt: HashMap<PlayerPosition, HashSet<Card>>,
    players: HashMap<PlayerPosition, &'p mut dyn KnowsCribbage>,
    scores: HashMap<PlayerPosition, u8>,
    dealer: PlayerPosition,
    crib: HashSet<Card>,
    played: Vec<Card>,
    up_card: Card,
    current_player: PlayerPosition,
}

impl<'p> GameComponents<'p> {
    fn new(first: &'p mut dyn KnowsCribbage, second: &'p mut dyn KnowsCribbage) -> Self {
        let deck = Deck::new();
        let mut hands: HashMap<PlayerPosition, HashSet<Card>> = HashMap::new();
        let mut dealt: HashMap<PlayerPosition, HashSet<Card>> = HashMap::new();
        let mut players: HashMap<PlayerPosition, &'p mut dyn KnowsCribbage> = HashMap::new();
        let mut scores: HashMap<PlayerPosition, u8> = HashMap::new();
        let dealer = PlayerPosition::First;
        let crib: HashSet<Card> = HashSet::new();
        let played: Vec<Card> = Vec::new();
        let up_card = Card::from("As");
        let current_player = PlayerPosition::First;

        players.insert(PlayerPosition::First, first);
        players.insert(PlayerPosition::Second, second);
        for position in [PlayerPosition::First, PlayerPosition::Second] {
            hands.insert(position, HashSet::new());
            dealt.insert(position, HashSet::new());
            scores.insert(position, 0);
        }

        Self {
            deck,
            hands,
            dealt,
            players,
            scores,
            dealer,
            crib,
            played,
            up_card,
            current_player,
        }
    }
}

trait ApplyRule<'p> {
    fn apply(self, game_components: &'p mut GameComponents) -> (GameState, PlayResult);
}

#[derive(Debug)]
struct Rule<S> {
    _state: S,
}

impl From<Rule<ChooseDealer>> for Rule<Deal> {
    fn from(_: Rule<ChooseDealer>) -> Rule<Deal> {
        Rule { _state: Deal }
    }
}

impl<'p> ApplyRule<'p> for Rule<ChooseDealer> {
    fn apply(self, components: &'p mut GameComponents) -> (GameState, PlayResult) {
        components.scores.clear();
        components.scores.insert(PlayerPosition::First, 0);
        components.scores.insert(PlayerPosition::Second, 0);
        components.deck.shuffle();
        let mut cards = components.deck.deal(2);

        /* Do not entertain a tie. Just choose new cards. */
        while cards[0].rank == cards[1].rank {
            cards = components.deck.deal(2);
        }

        let dealer_info = DealerInfo::from(&[cards[0], cards[1]]);
        (
            GameState::ReadyToDeal(Rule::<Deal>::from(self)),
            PlayResult::DealerChosen(dealer_info),
        )
    }
}

impl From<Rule<Deal>> for Rule<ChooseCrib> {
    fn from(_: Rule<Deal>) -> Rule<ChooseCrib> {
        Rule { _state: ChooseCrib }
    }
}

impl<'p> ApplyRule<'p> for Rule<Deal> {
    fn apply(self, components: &mut GameComponents) -> (GameState, PlayResult) {
        components.deck.shuffle();
        components.hands.clear();
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(components.deck.deal(6).iter().copied()),
        );
        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(components.deck.deal(6).iter().copied()),
        );
        let dealt_info = DealtInfo {
            hands: components.hands.clone(),
            dealer: components.dealer,
            scores: components.scores.clone(),
        };
        (
            GameState::WaitingForCrib(Rule::<ChooseCrib>::from(self)),
            PlayResult::CardsDealt(dealt_info),
        )
    }
}

impl From<Rule<ChooseCrib>> for Rule<TurnUpcard> {
    fn from(_: Rule<ChooseCrib>) -> Rule<TurnUpcard> {
        Rule { _state: TurnUpcard }
    }
}

impl From<Rule<TurnUpcard>> for Rule<GameOver> {
    fn from(_: Rule<TurnUpcard>) -> Rule<GameOver> {
        Rule { _state: GameOver }
    }
}

impl<'p> ApplyRule<'p> for Rule<ChooseCrib> {
    fn apply(self, components: &mut GameComponents) -> (GameState, PlayResult) {
        components.crib.clear();
        for position in [PlayerPosition::First, PlayerPosition::Second] {
            let hand = components.hands.get_mut(&position).unwrap();
            let player = components.players.get_mut(&position).unwrap();
            let choices = player.choose_crib(hand);
            for choice in choices {
                hand.remove(&choice);
                components.crib.insert(choice);
            }
            let dealt = components.dealt.get_mut(&position).unwrap();
            for card in hand.iter() {
                dealt.insert(*card);
            }
        }
        let crib_info = CribInfo {
            hands: components.hands.clone(),
            dealer: components.dealer,
            scores: components.scores.clone(),
        };
        (
            GameState::TurningUpCard(Rule::<TurnUpcard>::from(self)),
            PlayResult::CribChosen(crib_info),
        )
    }
}

impl From<Rule<TurnUpcard>> for Rule<Pegging> {
    fn from(_: Rule<TurnUpcard>) -> Rule<Pegging> {
        Rule { _state: Pegging }
    }
}

impl From<Rule<Pegging>> for Rule<GameOver> {
    fn from(_: Rule<Pegging>) -> Rule<GameOver> {
        Rule { _state: GameOver }
    }
}

impl From<Rule<Pegging>> for Rule<FirstGo> {
    fn from(_: Rule<Pegging>) -> Rule<FirstGo> {
        Rule { _state: FirstGo }
    }
}

impl From<Rule<FirstGo>> for Rule<Pegging> {
    fn from(_: Rule<FirstGo>) -> Rule<Pegging> {
        Rule { _state: Pegging }
    }
}

impl From<Rule<SecondGo>> for Rule<ResumePeggingOrScoreHands> {
    fn from(_: Rule<SecondGo>) -> Rule<ResumePeggingOrScoreHands> {
        Rule { _state: ResumePeggingOrScoreHands }
    }
}

impl From<Rule<FirstGo>> for Rule<SecondGo> {
    fn from(_: Rule<FirstGo>) -> Rule<SecondGo> {
        Rule { _state: SecondGo }
    }
}

impl From<Rule<SecondGo>> for Rule<GameOver> {
    fn from(_: Rule<SecondGo>) -> Rule<GameOver> {
        Rule { _state: GameOver }
    }
}

impl From<Rule<ResumePeggingOrScoreHands>> for Rule<ScoreDealer> {
    fn from(_: Rule<ResumePeggingOrScoreHands>) -> Rule<ScoreDealer> {
        Rule { _state: ScoreDealer }
    }
}

impl From<Rule<ResumePeggingOrScoreHands>> for Rule<Pegging> {
    fn from(_: Rule<ResumePeggingOrScoreHands>) -> Rule<Pegging> {
        Rule { _state: Pegging }
    }
}

impl<'p> ApplyRule<'p> for Rule<TurnUpcard> {
    fn apply(self, components: &mut GameComponents) -> (GameState, PlayResult) {
        components.current_player = components.dealer.next();
        components.played.clear();
        components.up_card = components.deck.deal(1)[0];
        if components.up_card.rank == Rank::Jack {
            let score = components.scores.get_mut(&components.dealer).unwrap();
            *score += 2;
            if *score >= 121 {
                *score = 121;
                let result = GameResult {
                    up_card: components.up_card,
                    scores: components.scores.clone(),
                    hands: components.hands.clone(),
                    winner: components.dealer,
                };
                return (
                    GameState::GameOver(Rule::<GameOver>::from(self)),
                    PlayResult::GameOver(result),
                );
            }
        }
        let pegging_info = PeggingInfo {
            hands: components.hands.clone(),
            dealer: components.dealer,
            scores: components.scores.clone(),
            played: components.played.clone(),
            up_card: components.up_card,
            current_player: components.current_player,
        };
        (
            GameState::Pegging(Rule::<Pegging>::from(self)),
            PlayResult::WaitingForPlay(pegging_info),
        )
    }
}

fn must_say_go(hand: &HashSet<Card>, played: &[Card]) -> bool {
    let current_count: u8 = played
        .iter()
        .map(|c| c.rank.value())
        .sum();

    hand
        .iter()
        .filter(|c| c.rank.value() + current_count <= 31)
        .map(|c| c.rank.value())
        .collect::<Vec<u8>>()
        .is_empty()
}

impl<'p> ApplyRule<'p> for Rule<Pegging> {
    fn apply(self, components: &mut GameComponents) -> (GameState, PlayResult) {
        let player = components.players.get_mut(&components.current_player).unwrap();
        let hand = components.hands.get_mut(&components.current_player).unwrap();
        if must_say_go(hand, &components.played) {
            let go_info = GoInfo {
                dealer: components.dealer,
                played: components.played.clone(),
                hands: components.hands.clone(),
                scores: components.scores.clone(),
                caller: components.current_player,
                up_card: components.up_card,
            };

            return (
                GameState::AnnouncingFirstGo(Rule::<FirstGo>::from(self)),
                PlayResult::Go(go_info),
            );
        }

        let choice = player.play(hand);
        hand.remove(&choice);
        components.played.push(choice);
        let found_scorings = score_pegging(components.played.clone());
        let score = components.scores.get_mut(&components.current_player).unwrap();
        if !found_scorings.is_empty() {
            *score += found_scorings
                .iter()
                .map(|s| s.value())
                .sum::<u8>();
        }
        if *score >= 121 {
            *score = 121;
            let game_result = GameResult {
                hands: components.hands.clone(),
                scores: components.scores.clone(),
                winner: components.current_player,
                up_card: components.up_card,
            };
            return (
                GameState::GameOver(Rule::<GameOver>::from(self)),
                PlayResult::GameOver(game_result),
            );
        }
        components.current_player = components.current_player.next();
        let pegging_info = PeggingInfo {
            hands: components.hands.clone(),
            dealer: components.dealer,
            scores: components.scores.clone(),
            played: components.played.clone(),
            up_card: components.up_card,
            current_player: components.current_player,
        };
        (
            GameState::Pegging(self),
            PlayResult::WaitingForPlay(pegging_info),
        )
        
    }
}

impl<'p> ApplyRule<'p> for Rule<FirstGo> {
    fn apply(self, components: &mut GameComponents) -> (GameState, PlayResult) {
        components.current_player = components.current_player.next();
        let hand = components.hands.get(&components.current_player).unwrap();
        if must_say_go(hand, &components.played) {
            let go_info = GoInfo {
                dealer: components.dealer,
                played: components.played.clone(),
                hands: components.hands.clone(),
                scores: components.scores.clone(),
                caller: components.current_player,
                up_card: components.up_card,
            };

            return (
                GameState::AnnouncingSecondGo(Rule::<SecondGo>::from(self)),
                PlayResult::Go(go_info),
            );
        }
        let pegging_info = PeggingInfo {
            hands: components.hands.clone(),
            dealer: components.dealer,
            scores: components.scores.clone(),
            played: components.played.clone(),
            up_card: components.up_card,
            current_player: components.current_player,
        };
        (
            GameState::Pegging(Rule::<Pegging>::from(self)),
            PlayResult::WaitingForPlay(pegging_info),
        )
    }
}

impl<'p> ApplyRule<'p> for Rule<SecondGo> {
    fn apply(self, components: &mut GameComponents) -> (GameState, PlayResult) {
        let score = components.scores.get_mut(&components.current_player).unwrap();
        *score += 2;
        if *score >= 121 {
            *score = 121;
            let result = GameResult {
                up_card: components.up_card,
                scores: components.scores.clone(),
                hands: components.hands.clone(),
                winner: components.current_player,
            };
            return (
                GameState::GameOver(Rule::<GameOver>::from(self)),
                PlayResult::GameOver(result),
            );
        }

        components.current_player = components.current_player.next();
        let pegging_info = PeggingInfo {
            hands: components.hands.clone(),
            dealer: components.dealer,
            scores: components.scores.clone(),
            played: components.played.clone(),
            up_card: components.up_card,
            current_player: components.current_player,
        };
        (
            GameState::ScoringLastCard(Rule::<ResumePeggingOrScoreHands>::from(self)),
            PlayResult::WaitingForLastCardAcknowledgement(pegging_info),
        )
    }
}

impl<'p> ApplyRule<'p> for Rule<ResumePeggingOrScoreHands> {
    fn apply(self, components: &mut GameComponents) -> (GameState, PlayResult) {
        components.played = Vec::new();
        if components.hands.get(&components.current_player).unwrap().is_empty() {
            /* Current player can't play. Skip them. */
            components.current_player = components.current_player.next();
            if components.hands.get(&components.current_player).unwrap().is_empty() {
                /* Neither player can play. Move to start counting hands. */
                let dealt = components.dealt.get(&components.dealer.next()).unwrap();
                let scorings = score_hand(dealt, components.up_card);
                let score = components.scores.get_mut(&components.dealer.next()).unwrap();
                *score += scorings
                    .iter()
                    .map(|x| x.value())
                    .sum::<u8>();
                if *score >= 121 {
                    let game_results = GameResult {
                        hands: components.hands.clone(),
                        scores: components.scores.clone(),
                        up_card: components.up_card,
                        winner: components.current_player,
                    };
                } else {
                    let scoring_info = ScoreInfo {
                        hand: dealt.clone(),
                        player: components.dealer.next(),
                        scores: components.scores.clone(),
                        scorings: Some(scorings),
                        up_card: components.up_card,
                    };
                    return (
                        GameState::ScoringDealer(Rule::<ScoreDealer>::from(self)),
                        PlayResult::HandScore(scoring_info),
                    );
                }
            }
        }
        let pegging_info = PeggingInfo {
            current_player: components.current_player,
            dealer: components.dealer,
            hands: components.hands.clone(),
            played: components.played.clone(),
            scores: components.scores.clone(),
            up_card: components.up_card,
        };
        (
            GameState::Pegging(Rule::<Pegging>::from(self)),
            PlayResult::WaitingForPlay(pegging_info),
        )
    }
}

pub struct DealerInfo {
    pub dealer: PlayerPosition,
    pub chosen_cards: HashMap<PlayerPosition, Card>,
}

pub struct DealtInfo {
    pub hands: HashMap<PlayerPosition, HashSet<Card>>,
    pub dealer: PlayerPosition,
    pub scores: HashMap<PlayerPosition, u8>,
}

pub struct CribInfo {
    pub hands: HashMap<PlayerPosition, HashSet<Card>>,
    pub dealer: PlayerPosition,
    pub scores: HashMap<PlayerPosition, u8>,
}

pub struct PeggingInfo {
    pub hands: HashMap<PlayerPosition, HashSet<Card>>,
    pub dealer: PlayerPosition,
    pub scores: HashMap<PlayerPosition, u8>,
    pub up_card: Card,
    pub played: Vec<Card>,
    pub current_player: PlayerPosition,
}

pub struct GoInfo {
    pub hands: HashMap<PlayerPosition, HashSet<Card>>,
    pub dealer: PlayerPosition,
    pub scores: HashMap<PlayerPosition, u8>,
    pub up_card: Card,
    pub played: Vec<Card>,
    pub caller: PlayerPosition,
}

pub struct GameResult {
    pub winner: PlayerPosition,
    pub scores: HashMap<PlayerPosition, u8>,
    pub up_card: Card,
    pub hands: HashMap<PlayerPosition, HashSet<Card>>,
}

pub struct ScoreInfo {
    pub player: PlayerPosition,
    pub hand: HashSet<Card>,
    pub up_card: Card,
    pub scorings: Option<Vec<HandScorings>>,
    pub scores: HashMap<PlayerPosition, u8>,
}

impl From<&[Card; 2]> for DealerInfo {
    fn from(cards: &[Card; 2]) -> DealerInfo {
        let dealer = match cards[0].rank < cards[1].rank {
            true => PlayerPosition::First,
            false => PlayerPosition::Second,
        };
        let mut chosen_cards: HashMap<PlayerPosition, Card> = HashMap::new();
        chosen_cards.insert(PlayerPosition::First, cards[0]);
        chosen_cards.insert(PlayerPosition::Second, cards[1]);
        DealerInfo {
            dealer,
            chosen_cards,
        }
    }
}

pub enum PlayResult {
    DealerChosen(DealerInfo),
    CardsDealt(DealtInfo),
    CribChosen(CribInfo),
    WaitingForPlay(PeggingInfo),
    GameOver(GameResult),
    Go(GoInfo),
    WaitingForLastCardAcknowledgement(PeggingInfo),
    HandScore(ScoreInfo),
    Incomplete,
}

pub struct GameRunner<'p> {
    _game_state: GameState,
    _game_components: GameComponents<'p>,
}

impl<'a> GameRunner<'a> {
    pub fn new(first: &'a mut dyn KnowsCribbage, second: &'a mut dyn KnowsCribbage) -> Self {
        let _game_state = GameState::New(Rule { _state: ChooseDealer });
        let _game_components = GameComponents::new(first, second);
        Self {
            _game_state,
            _game_components,
        }
    }
}

#[cfg(test)]
mod test_rules {
    use super::*;
    use crate::player::SimplePlayer;

    #[test]
    fn chooses_dealer() {
        let tests = vec![
            (
                vec![Card::from("As"), Card::from("Qh")],
                PlayerPosition::First,
                None,
            ),
            (
                vec![Card::from("Kd"), Card::from("2c")],
                PlayerPosition::Second,
                None,
            ),
            (
                vec![
                    Card::from("Kd"),
                    Card::from("Kc"),
                    Card::from("As"),
                    Card::from("Qs"),
                ],
                PlayerPosition::First,
                Some(vec![Card::from("As"), Card::from("Qs")]),
            ),
        ];

        for (cards, dealer, chosen_cards) in tests {
            let mut first: SimplePlayer = SimplePlayer {};
            let mut second: SimplePlayer = SimplePlayer {};
            let mut components = GameComponents::new(&mut first, &mut second);
            let deck = Deck::stacked(cards.clone());
            components.deck = deck;
            let rule = Rule { _state: ChooseDealer };
            let (next_state, play_result) = rule.apply(&mut components);
            match next_state {
                GameState::ReadyToDeal(next_rule) => assert_eq!(Deal, next_rule._state),
                _ => panic!("Wrong game state"),
            }
            match play_result {
                PlayResult::DealerChosen(result) => {
                    assert_eq!(dealer, result.dealer);
                    let chosen_cards = match chosen_cards {
                        Some(chosen) => chosen,
                        None => cards,
                    };
                    assert_eq!(
                        chosen_cards[0],
                        *result.chosen_cards.get(&PlayerPosition::First).unwrap()
                    );
                    assert_eq!(
                        chosen_cards[1],
                        *result.chosen_cards.get(&PlayerPosition::Second).unwrap()
                    );
                }
                _ => panic!("Wrong rule result"),
            }
        }
    }

    #[test]
    fn deals_cards() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let cards = vec![
            Card::from("As"),
            Card::from("2s"),
            Card::from("3s"),
            Card::from("4s"),
            Card::from("5s"),
            Card::from("6s"),
            Card::from("Ah"),
            Card::from("Ah"),
            Card::from("Ah"),
            Card::from("Ah"),
            Card::from("Ah"),
            Card::from("Ah"),
        ];

        components.deck = Deck::stacked(cards.clone());
        let rule = Rule { _state: Deal };
        let (next_state, result) = rule.apply(&mut components);

        match next_state {
            GameState::WaitingForCrib(next_rule) => assert_eq!(ChooseCrib, next_rule._state),
            _ => panic!("Wrong game state"),
        }

        match result {
            PlayResult::CardsDealt(result) => {
                assert_eq!(
                    *result.hands.get(&PlayerPosition::First).unwrap(),
                    HashSet::from_iter(cards[0..6].iter().cloned())
                );
                assert_eq!(
                    *result.hands.get(&PlayerPosition::Second).unwrap(),
                    HashSet::from_iter(cards[6..].iter().cloned())
                );
                assert_eq!(0, *result.scores.get(&PlayerPosition::First).unwrap(),);
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn choose_crib() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: ChooseCrib };

        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![
                Card::from("As"),
                Card::from("2s"),
                Card::from("3s"),
                Card::from("4s"),
                Card::from("5s"),
                Card::from("6s"),
            ]),
        );

        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![
                Card::from("Ah"),
                Card::from("2h"),
                Card::from("3h"),
                Card::from("4h"),
                Card::from("5h"),
                Card::from("6h"),
            ]),
        );
        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::TurningUpCard(next_rule) => assert_eq!(TurnUpcard, next_rule._state),
            _ => panic!("Wrong game state")
        }
        match result {
            PlayResult::CribChosen(result) => {
                assert_eq!(
                    HashSet::from_iter(vec![
                        Card::from("3s"),
                        Card::from("4s"),
                        Card::from("5s"),
                        Card::from("6s"),
                    ]),
                    *result.hands.get(&PlayerPosition::First).unwrap()
                );
                assert_eq!(
                    HashSet::from_iter(vec![
                        Card::from("3h"),
                        Card::from("4h"),
                        Card::from("5h"),
                        Card::from("6h"),
                    ]),
                    *result.hands.get(&PlayerPosition::Second).unwrap()
                );
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn up_card_no_nibs() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: TurnUpcard };
        components.deck = Deck::stacked(vec![Card::from("As")]);
        let (next_rule, result) = rule.apply(&mut components);
        match next_rule {
            GameState::Pegging(next_rule) => assert_eq!(Pegging, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::WaitingForPlay(result) => {
                assert_eq!(Card::from("As"), result.up_card);
                assert_eq!(0, *result.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *result.scores.get(&PlayerPosition::Second).unwrap());
                assert_eq!(PlayerPosition::Second, result.current_player);
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn up_card_nibs_no_win() {
        for up_card in [
            Card::from("Js"),
            Card::from("Jh"),
            Card::from("Jc"),
            Card::from("Jd"),
        ] {
            let mut first: SimplePlayer = SimplePlayer {};
            let mut second: SimplePlayer = SimplePlayer {};
            let mut components = GameComponents::new(&mut first, &mut second);
            components.deck = Deck::stacked(vec![up_card]);
            let rule = Rule { _state: TurnUpcard };
            components.dealer = PlayerPosition::Second;
            let (next_state, result) = rule.apply(&mut components);
            match next_state {
                GameState::Pegging(next_rule) => assert_eq!(Pegging, next_rule._state),
                _ => panic!("Wrong game state"),
            }
            match result {
                PlayResult::WaitingForPlay(result) => {
                    assert_eq!(up_card, result.up_card);
                    assert_eq!(0, *result.scores.get(&PlayerPosition::First).unwrap());
                    assert_eq!(2, *result.scores.get(&PlayerPosition::Second).unwrap());
                    assert_eq!(PlayerPosition::First, result.current_player);
                }
                _ => panic!("Wrong rule result"),
            }
        }
    }

    #[test]
    fn up_card_nibs_with_win() {
        for up_card in [
            Card::from("Js"),
            Card::from("Jh"),
            Card::from("Jc"),
            Card::from("Jd"),
        ] {
            let mut first: SimplePlayer = SimplePlayer {};
            let mut second: SimplePlayer = SimplePlayer {};
            let mut components = GameComponents::new(&mut first, &mut second);
            components.deck = Deck::stacked(vec![up_card]);
            let rule = Rule { _state: TurnUpcard };
            components.dealer = PlayerPosition::Second;
            let score = components.scores.get_mut(&PlayerPosition::Second).unwrap();
            *score = 119;
            let (next_state, result) = rule.apply(&mut components);
            match next_state {
                GameState::GameOver(next_rule) => assert_eq!(GameOver, next_rule._state),
                _ => panic!("Wrong game state"),
            }
            match result {
                PlayResult::GameOver(result) => {
                    assert_eq!(up_card, result.up_card);
                    assert_eq!(0, *result.scores.get(&PlayerPosition::First).unwrap());
                    assert_eq!(121, *result.scores.get(&PlayerPosition::Second).unwrap());
                    assert_eq!(PlayerPosition::Second, result.winner);
                }
                _ => panic!("Wrong rule result"),
            }
        }
    }

    #[test]
    fn play_single_card() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: Pegging };
        components.current_player = PlayerPosition::First;
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![
                Card::from("As"),
            ]),
        );
        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![Card::from("Ac")]),
        );
        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::Pegging(next_rule) => assert_eq!(Pegging, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::WaitingForPlay(result) => {
                assert_eq!(0, *result.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *result.scores.get(&PlayerPosition::Second).unwrap());
                assert_eq!(PlayerPosition::Second, result.current_player);
                assert!(!result.hands.get(&PlayerPosition::First).unwrap().contains(&Card::from("As")));
                assert!(result.played.contains(&Card::from("As")));
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn pegging_will_apply_scores() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: Pegging };
        components.current_player = PlayerPosition::First;
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![
                Card::from("6s"),
            ]),
        );
        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![Card::from("Ac")]),
        );
        components.played = vec![Card::from("4c"), Card::from("5c")];
        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::Pegging(next_rule) => assert_eq!(Pegging, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::WaitingForPlay(result) => {
                assert_eq!(5, *result.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *result.scores.get(&PlayerPosition::Second).unwrap());
                assert_eq!(PlayerPosition::Second, result.current_player);
                assert!(!result.hands.get(&PlayerPosition::First).unwrap().contains(&Card::from("As")));
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn can_win_from_pegging() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: Pegging };
        components.current_player = PlayerPosition::First;
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![
                Card::from("6s"),
            ]),
        );
        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![Card::from("Ac")]),
        );
        components.played = vec![Card::from("4c"), Card::from("5c")];
        let score = components.scores.get_mut(&PlayerPosition::First).unwrap();
        *score = 119;

        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::GameOver(next_rule) => assert_eq!(GameOver, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::GameOver(result) => {
                assert_eq!(121, *result.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *result.scores.get(&PlayerPosition::Second).unwrap());
                assert_eq!(PlayerPosition::First, result.winner);
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn must_say_go_if_no_cards() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: Pegging };
        components.current_player = PlayerPosition::First;
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![]),
        );
        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![Card::from("Ac")]),
        );

        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::AnnouncingFirstGo(next_rule) => assert_eq!(FirstGo, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::Go(result) => {
                assert_eq!(0, *result.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *result.scores.get(&PlayerPosition::Second).unwrap());
                assert_eq!(PlayerPosition::First, result.caller);
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn must_say_go_if_cannot_play() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: Pegging };
        components.current_player = PlayerPosition::First;
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![Card::from("Ts")]),
        );
        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![Card::from("Ac")]),
        );
        components.played = vec![Card::from("Kc"), Card::from("Qd"), Card::from("Jh")];
        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::AnnouncingFirstGo(next_rule) => assert_eq!(FirstGo, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::Go(result) => {
                assert_eq!(0, *result.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(0, *result.scores.get(&PlayerPosition::Second).unwrap());
                assert_eq!(PlayerPosition::First, result.caller);
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn can_still_play_if_first_go_announced_but_still_able() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: FirstGo };
        components.current_player = PlayerPosition::First;
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![Card::from("Ts")]),
        );
        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![Card::from("Ac")]),
        );
        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::Pegging(next_rule) => assert_eq!(Pegging, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::WaitingForPlay(result) => {
                assert_eq!(PlayerPosition::Second, result.current_player);
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn must_say_go_if_first_go_announced_but_no_cards() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: FirstGo };
        components.current_player = PlayerPosition::First;
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![Card::from("Ts")]),
        );
        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![]),
        );
        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::AnnouncingSecondGo(next_rule) => assert_eq!(SecondGo, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::Go(result) => {
                assert_eq!(PlayerPosition::Second, result.caller);
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn can_win_if_scores_on_last_card() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: SecondGo };
        components.current_player = PlayerPosition::First;
        components.scores.insert(
            PlayerPosition::First,
            119,
        );
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![Card::from("Ts")]),
        );
        components.hands.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![Card::from("Td")]),
        );
        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::GameOver(next_rule) => assert_eq!(GameOver, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::GameOver(result) => {
                assert_eq!(121, *components.scores.get(&PlayerPosition::First).unwrap());
                assert_eq!(PlayerPosition::First, result.winner);
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn continues_pegging_if_more_cards_to_play() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: ResumePeggingOrScoreHands };
        components.current_player = PlayerPosition::First;
        components.hands.insert(
            PlayerPosition::First,
            HashSet::from_iter(vec![Card::from("As")]),
        );
        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::Pegging(next_rule) => assert_eq!(Pegging, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::WaitingForPlay(result) => {
                assert_eq!(PlayerPosition::First, result.current_player);
            }
            _ => panic!("Wrong rule result"),
        }
    }

    #[test]
    fn counts_pone_if_no_more_cards_to_play() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: ResumePeggingOrScoreHands };
        components.current_player = PlayerPosition::First;
        components.dealt.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![
                Card::from("4s"),
                Card::from("5h"),
                Card::from("6d"),
                Card::from("8c"),
            ])
        );
        components.up_card = Card::from("2s");

        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::ScoringDealer(next_rule) => assert_eq!(ScoreDealer, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::HandScore(result) => {
                assert_eq!(PlayerPosition::Second, result.player);
                if let Some(scorings) = result.scorings {
                    assert_eq!(3, scorings.len());
                    assert_eq!(7, *result.scores.get(&result.player).unwrap());
                } else {
                    panic!("Expected some scorings");
                }
            }
            _ => panic!("Wrong rule result"),
        }
    }

    /*
    #[test]
    fn can_win_on_a_pone_count() {
        let mut first: SimplePlayer = SimplePlayer {};
        let mut second: SimplePlayer = SimplePlayer {};
        let mut components = GameComponents::new(&mut first, &mut second);
        let rule = Rule { _state: ResumePeggingOrScoreHands };
        components.current_player = PlayerPosition::First;
        components.dealt.insert(
            PlayerPosition::Second,
            HashSet::from_iter(vec![
                Card::from("4s"),
                Card::from("5h"),
                Card::from("6d"),
                Card::from("8c"),
            ])
        );
        components.scores.insert(
            PlayerPosition::Second,
            119,
        );
        components.up_card = Card::from("2s");

        let (next_state, result) = rule.apply(&mut components);
        match next_state {
            GameState::GameOver(next_rule) => assert_eq!(GameOver, next_rule._state),
            _ => panic!("Wrong game state"),
        }
        match result {
            PlayResult::GameOver(result) => {
                assert_eq!(PlayerPosition::Second, result.winner);
                if let Some(scorings) = result.scorings {
                    assert_eq!(3, scorings.len());
                    assert_eq!(121, *result.scores.get(&result.winner).unwrap());
                } else {
                    panic!("Expected some scorings");
                }
            }
            _ => panic!("Wrong rule result"),
        }
    }
*/
}

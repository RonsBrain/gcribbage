use crate::simulation::deck::{Card, Dealable};
use crate::simulation::player::PlayerPosition;

pub enum GameState {
    ChooseDealer,
    Done,
}

pub struct GameRunner<'a, D: Dealable> {
    /// The main game runner. Simulates all the rules of cribbage
    /// in the right sequence.
    state: GameState,
    pub deck: &'a mut D,
}

#[derive(Debug)]
pub struct CutInfo {
    pub up_cards: (Card, Card),
    pub dealer: PlayerPosition,
}

#[derive(Debug)]
pub enum PlayResult {
    DealerChosen(CutInfo),
}

impl<'a, D: Dealable> GameRunner<'a, D> {
    /// Creates a new GameRunner with the provided deck.
    pub fn new(deck: &'a mut D) -> Self {
        Self{
            state: GameState::ChooseDealer,
            deck,
        }
    }
}

impl<'a, D: Dealable> Iterator for GameRunner<'a, D> {
    type Item = PlayResult;

    /// Iterates over the game states until the end of the game. Each
    /// iteration returns a PlayResult which describes the current game
    /// state with the relevant information.
    fn next(&mut self) -> Option<Self::Item> {
        let (new_state, result) = match self.state {
            GameState::ChooseDealer => self.choose_dealer(),
            _ => (GameState::Done, None), 
        };
        self.state = new_state;
        result
    }
}

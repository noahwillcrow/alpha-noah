use crate::enums::DecideNextStateError;
use crate::traits::{
    AvailableNextGameStatesFinder, BasicGameState, GameStateWeightsCalculator, TurnTaker,
};
use rand::distributions::WeightedIndex;
use rand::prelude::*;

pub struct WeightedGameStatesMonteCarloTurnTaker<'a, GameState: BasicGameState> {
    available_next_game_states_finder: &'a dyn AvailableNextGameStatesFinder<GameState>,
    game_state_weights_calculator: &'a dyn GameStateWeightsCalculator<GameState>,
    player_index: i32,
}

impl<'a, GameState: BasicGameState> WeightedGameStatesMonteCarloTurnTaker<'a, GameState> {
    pub fn new(
        available_next_game_states_finder: &'a dyn AvailableNextGameStatesFinder<GameState>,
        game_state_weights_calculator: &'a dyn GameStateWeightsCalculator<GameState>,
        player_index: i32,
    ) -> WeightedGameStatesMonteCarloTurnTaker<'a, GameState> {
        return WeightedGameStatesMonteCarloTurnTaker {
            available_next_game_states_finder: available_next_game_states_finder,
            game_state_weights_calculator: game_state_weights_calculator,
            player_index: player_index,
        };
    }
}

impl<'a, GameState: BasicGameState> TurnTaker<GameState>
    for WeightedGameStatesMonteCarloTurnTaker<'a, GameState>
{
    fn decide_next_game_state(
        &mut self,
        current_game_state: &GameState,
    ) -> Result<GameState, DecideNextStateError> {
        let available_next_game_states = self
            .available_next_game_states_finder
            .find_available_next_game_states(self.player_index, &current_game_state);

        if available_next_game_states.is_empty() {
            return Err(DecideNextStateError::NoAvailableStatesError);
        }

        let weights_for_available_next_game_states = self
            .game_state_weights_calculator
            .weigh_game_states(self.player_index, &available_next_game_states);

        match WeightedIndex::new(&weights_for_available_next_game_states) {
            Ok(dist) => {
                let mut rng = rand::thread_rng();
                let next_game_state_index = dist.sample(&mut rng);
                let next_game_state = available_next_game_states[next_game_state_index].clone();

                return Ok(next_game_state);
            }
            Err(_) => return Err(DecideNextStateError::Unknown),
        }
    }
}

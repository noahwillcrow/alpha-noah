use crate::enums::DecideNextStateError;
use crate::traits::{BasicGameState, GameRulesAuthority, GameStateWeightsCalculator, TurnTaker};
use rand::distributions::WeightedIndex;
use rand::prelude::*;

pub struct WeightedRandomSelectionTurnTaker<'a, GameState: BasicGameState> {
    game_rules_authority: &'a dyn GameRulesAuthority<GameState>,
    game_state_weights_calculator: &'a dyn GameStateWeightsCalculator<GameState>,
    player_index: i32,
}

impl<'a, GameState: BasicGameState> WeightedRandomSelectionTurnTaker<'a, GameState> {
    pub fn new(
        game_rules_authority: &'a dyn GameRulesAuthority<GameState>,
        game_state_weights_calculator: &'a dyn GameStateWeightsCalculator<GameState>,
        player_index: i32,
    ) -> WeightedRandomSelectionTurnTaker<'a, GameState> {
        return WeightedRandomSelectionTurnTaker {
            game_rules_authority: game_rules_authority,
            game_state_weights_calculator: game_state_weights_calculator,
            player_index: player_index,
        };
    }
}

impl<'a, GameState: BasicGameState> TurnTaker<GameState>
    for WeightedRandomSelectionTurnTaker<'a, GameState>
{
    fn decide_next_game_state(
        &self,
        current_game_state: &GameState,
    ) -> Result<GameState, DecideNextStateError> {
        let available_next_game_states = self
            .game_rules_authority
            .find_available_next_game_states(self.player_index, &current_game_state);

        if available_next_game_states.is_empty() {
            return Err(DecideNextStateError::NoAvailableStatesError);
        }

        let mut weights_for_available_next_game_states = self
            .game_state_weights_calculator
            .weigh_game_states(self.player_index, &available_next_game_states);

        let mut min_weight = f32::MAX;
        for weight in weights_for_available_next_game_states.iter() {
            if *weight < min_weight {
                min_weight = *weight;
            }
        }

        if min_weight < 1.0 {
            // It is possible that all of our weights are negative or zero
            // so now we'll go through and raise them all to at least 1.
            // To do this, we'll find the lowest value and then raise everything
            // else by the same value.
            let addend = 1.0 - min_weight;
            for i in 0..weights_for_available_next_game_states.len() {
                weights_for_available_next_game_states[i] += addend;
            }
        }

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

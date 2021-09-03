use crate::enums::DecideNextStateError;
use crate::traits::{BasicGameState, GameRulesAuthority, GameStateWeightsCalculator, TurnTaker};

pub struct BestWeightSelectionTurnTaker<'a, GameState: BasicGameState> {
    game_rules_authority: &'a dyn GameRulesAuthority<GameState>,
    game_state_weights_calculator: &'a dyn GameStateWeightsCalculator<GameState>,
    player_index: i32,
}

impl<'a, GameState: BasicGameState> BestWeightSelectionTurnTaker<'a, GameState> {
    pub fn new(
        game_rules_authority: &'a dyn GameRulesAuthority<GameState>,
        game_state_weights_calculator: &'a dyn GameStateWeightsCalculator<GameState>,
        player_index: i32,
    ) -> BestWeightSelectionTurnTaker<'a, GameState> {
        return BestWeightSelectionTurnTaker {
            game_rules_authority: game_rules_authority,
            game_state_weights_calculator: game_state_weights_calculator,
            player_index: player_index,
        };
    }
}

impl<'a, GameState: BasicGameState> TurnTaker<GameState>
    for BestWeightSelectionTurnTaker<'a, GameState>
{
    fn decide_next_game_state(
        &mut self,
        current_game_state: &GameState,
    ) -> Result<GameState, DecideNextStateError> {
        let available_next_game_states = self
            .game_rules_authority
            .find_available_next_game_states(self.player_index, &current_game_state);

        if available_next_game_states.is_empty() {
            return Err(DecideNextStateError::NoAvailableStatesError);
        }

        let weights_for_available_next_game_states = self
            .game_state_weights_calculator
            .weigh_game_states(self.player_index, &available_next_game_states);

        let mut best_available_index = 0;

        for i in 1..weights_for_available_next_game_states.len() {
            let weight = weights_for_available_next_game_states[i];

            if weight > weights_for_available_next_game_states[best_available_index] {
                best_available_index = i;
            }
        }

        return Ok(available_next_game_states[best_available_index].clone());
    }
}

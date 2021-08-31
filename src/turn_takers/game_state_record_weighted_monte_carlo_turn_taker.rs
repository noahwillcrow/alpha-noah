use crate::enums::DecideNextStateError;
use crate::structs::GameStateRecord;
use crate::traits::{
    AvailableNextGameStatesFinder, BasicGameState, BasicSerializedGameState,
    GameStateRecordWeightsCalculator, GameStateRecordsProvider, GameStateSerializer, TurnTaker,
};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::cell::RefCell;

pub struct GameStateRecordWeightedMonteCarloTurnTaker<
    'a,
    GameState: BasicGameState,
    SerializedGameState: BasicSerializedGameState,
> {
    available_next_game_states_finder: &'a dyn AvailableNextGameStatesFinder<GameState>,
    game_state_records_provider_refcell:
        &'a RefCell<dyn GameStateRecordsProvider<SerializedGameState>>,
    game_state_record_weights_calculator: &'a dyn GameStateRecordWeightsCalculator,
    game_state_serializer: &'a dyn GameStateSerializer<GameState, SerializedGameState>,
    player_index: i32,
}

impl<'a, GameState: BasicGameState, SerializedGameState: BasicSerializedGameState>
    GameStateRecordWeightedMonteCarloTurnTaker<'a, GameState, SerializedGameState>
{
    pub fn new(
        available_next_game_states_finder: &'a dyn AvailableNextGameStatesFinder<GameState>,
        game_state_records_provider_refcell: &'a RefCell<
            dyn GameStateRecordsProvider<SerializedGameState>,
        >,
        game_state_record_weights_calculator: &'a dyn GameStateRecordWeightsCalculator,
        game_state_serializer: &'a dyn GameStateSerializer<GameState, SerializedGameState>,
        player_index: i32,
    ) -> GameStateRecordWeightedMonteCarloTurnTaker<'a, GameState, SerializedGameState> {
        return GameStateRecordWeightedMonteCarloTurnTaker {
            available_next_game_states_finder: available_next_game_states_finder,
            game_state_records_provider_refcell: game_state_records_provider_refcell,
            game_state_record_weights_calculator: game_state_record_weights_calculator,
            game_state_serializer: game_state_serializer,
            player_index: player_index,
        };
    }
}

impl<'a, GameState: BasicGameState, SerializedGameState: BasicSerializedGameState>
    TurnTaker<GameState>
    for GameStateRecordWeightedMonteCarloTurnTaker<'a, GameState, SerializedGameState>
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

        let mut game_state_records_for_available_next_game_states: Vec<GameStateRecord> = vec![];
        for available_next_game_state in available_next_game_states.iter() {
            let serialized_available_next_game_state = self
                .game_state_serializer
                .serialize_game_state(self.player_index, &available_next_game_state);

            match self
                .game_state_records_provider_refcell
                .borrow_mut()
                .get_game_state_record(&serialized_available_next_game_state)
            {
                Some(game_state_record) => {
                    game_state_records_for_available_next_game_states.push(game_state_record)
                }
                None => game_state_records_for_available_next_game_states
                    .push(GameStateRecord::new_zeros()),
            }
        }

        let weights_for_available_next_game_states = self
            .game_state_record_weights_calculator
            .weigh_game_state_records(&game_state_records_for_available_next_game_states);

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

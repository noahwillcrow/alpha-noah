use crate::internal::utility_functions;
use crate::structs::GameStateRecord;
use crate::traits::{
    BasicGameState, BasicSerializedGameState, GameStateRecordsProvider, GameStateSerializer,
    GameStateWeightsCalculator,
};
use std::cell::RefCell;
use std::cmp;

pub struct RecordValuesWeightedSumGameStateWeightsCalculator<
    'a,
    GameState: BasicGameState,
    SerializedGameState: BasicSerializedGameState,
> {
    game_state_records_provider_ref_cell:
        &'a RefCell<dyn GameStateRecordsProvider<SerializedGameState>>,
    game_state_serializer: &'a dyn GameStateSerializer<GameState, SerializedGameState>,
    // actual results weights
    pub draws_weight: f32,
    pub losses_weight: f32,
    pub wins_weight: f32,

    // total visits
    pub visits_deficit_weight: f32,
}

impl<'a, GameState: BasicGameState, SerializedGameState: BasicSerializedGameState>
    RecordValuesWeightedSumGameStateWeightsCalculator<'a, GameState, SerializedGameState>
{
    pub fn new(
        game_state_records_provider_ref_cell: &'a RefCell<
            dyn GameStateRecordsProvider<SerializedGameState>,
        >,
        game_state_serializer: &'a dyn GameStateSerializer<GameState, SerializedGameState>,
        draws_weight: f32,
        losses_weight: f32,
        wins_weight: f32,
        visits_deficit_weight: f32,
    ) -> RecordValuesWeightedSumGameStateWeightsCalculator<'a, GameState, SerializedGameState> {
        return RecordValuesWeightedSumGameStateWeightsCalculator {
            game_state_records_provider_ref_cell: game_state_records_provider_ref_cell,
            game_state_serializer: game_state_serializer,
            draws_weight: draws_weight,
            losses_weight: losses_weight,
            wins_weight: wins_weight,
            visits_deficit_weight: visits_deficit_weight,
        };
    }
}

impl<'a, GameState: BasicGameState, SerializedGameState: BasicSerializedGameState>
    GameStateWeightsCalculator<GameState>
    for RecordValuesWeightedSumGameStateWeightsCalculator<'a, GameState, SerializedGameState>
{
    fn weigh_game_states(
        &self,
        responsible_player_index: i32,
        game_states: &Vec<GameState>,
    ) -> Vec<f32> {
        let mut game_state_records: Vec<GameStateRecord> = vec![];
        for game_state in game_states.iter() {
            let serialized_game_state = self
                .game_state_serializer
                .serialize_game_state(responsible_player_index, game_state);
            let game_state_record_result = self
                .game_state_records_provider_ref_cell
                .borrow_mut()
                .get_game_state_record(&serialized_game_state);

            match game_state_record_result {
                Some(game_state_record) => game_state_records.push(game_state_record),
                None => game_state_records.push(GameStateRecord::new_zeros()),
            }
        }

        let mut highest_visits_count_available = 0;

        for game_state_record in game_state_records.iter() {
            let visits_count = utility_functions::count_visits(&game_state_record);
            highest_visits_count_available = cmp::max(highest_visits_count_available, visits_count);
        }

        let mut weights: Vec<f32> = vec![];
        let mut min_weight: f32 = f32::MAX;

        for game_state_record in game_state_records.iter() {
            let visits_count = utility_functions::count_visits(&game_state_record);
            let visits_deficit = highest_visits_count_available - visits_count;
            let total_weight = (self.draws_weight * game_state_record.draws_count as f32)
                + (self.losses_weight * game_state_record.losses_count as f32)
                + (self.wins_weight * game_state_record.wins_count as f32)
                + (self.visits_deficit_weight * visits_deficit as f32);

            weights.push(total_weight);

            if total_weight < min_weight {
                min_weight = total_weight;
            }
        }

        if min_weight < 1.0 {
            // It is possible that all of our weights are negative or zero
            // so now we'll go through and raise them all to at least 1.
            // To do this, we'll find the lowest value and then raise everything
            // else by the same value.
            let addend = 1.0 - min_weight;
            for i in 0..weights.len() {
                weights[i] += addend;
            }
        }

        return weights;
    }
}

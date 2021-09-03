use crate::internal::utility_functions;
use crate::structs::GameStateRecord;
use crate::traits::{
    BasicGameState, BasicSerializedGameState, GameStateRecordsFetcher, GameStateSerializer,
    GameStateWeightsCalculator,
};
use std::cmp;

pub struct RecordValuesWeightedSumGameStateWeightsCalculator<
    'a,
    GameState: BasicGameState,
    SerializedGameState: BasicSerializedGameState,
> {
    game_state_records_fetcher: &'a dyn GameStateRecordsFetcher<SerializedGameState>,
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
        game_state_records_fetcher: &'a dyn GameStateRecordsFetcher<SerializedGameState>,
        game_state_serializer: &'a dyn GameStateSerializer<GameState, SerializedGameState>,
        draws_weight: f32,
        losses_weight: f32,
        wins_weight: f32,
        visits_deficit_weight: f32,
    ) -> RecordValuesWeightedSumGameStateWeightsCalculator<'a, GameState, SerializedGameState> {
        return RecordValuesWeightedSumGameStateWeightsCalculator {
            game_state_records_fetcher: game_state_records_fetcher,
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
                .game_state_records_fetcher
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

        for game_state_record in game_state_records.iter() {
            let visits_count = utility_functions::count_visits(&game_state_record);
            let visits_deficit = highest_visits_count_available - visits_count;
            let total_weight = (self.draws_weight * game_state_record.draws_count as f32)
                + (self.losses_weight * game_state_record.losses_count as f32)
                + (self.wins_weight * game_state_record.wins_count as f32)
                + (self.visits_deficit_weight * visits_deficit as f32);

            weights.push(total_weight);
        }

        return weights;
    }
}

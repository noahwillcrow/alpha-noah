use crate::internal::utility_functions;
use crate::structs::GameStateRecord;
use crate::traits::GameStateRecordWeightsCalculator;
use std::cmp;

pub struct WeightedSumGameStateRecordWeightsCalculator {
    // actual results weights
    pub draws_weight: f32,
    pub losses_weight: f32,
    pub wins_weight: f32,

    // total visits
    pub visits_deficit_weight: f32,
}

impl GameStateRecordWeightsCalculator for WeightedSumGameStateRecordWeightsCalculator {
    fn weigh_game_state_records(&self, game_state_records: &Vec<GameStateRecord>) -> Vec<f32> {
        let mut highest_visits_count_available = 0;

        for game_state_record in game_state_records.iter() {
            let visits_count = utility_functions::count_visits(&game_state_record);
            highest_visits_count_available = cmp::max(highest_visits_count_available, visits_count);
        }

        let mut weights: Vec<f32> = vec![];

        for game_state_record in game_state_records.iter() {
            let visits_count = utility_functions::count_visits(&game_state_record);
            let visits_deficit = highest_visits_count_available - visits_count;

            weights.push(
                (self.draws_weight * game_state_record.draws_count as f32)
                    + (self.losses_weight * game_state_record.losses_count as f32)
                    + (self.wins_weight * game_state_record.wins_count as f32)
                    + (self.visits_deficit_weight * visits_deficit as f32),
            );
        }

        return weights;
    }
}

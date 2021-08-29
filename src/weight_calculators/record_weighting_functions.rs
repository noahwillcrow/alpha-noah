use crate::core::state_record::StateRecord;
use crate::core::utilities;
use std::cmp;

#[allow(dead_code)]
pub fn zero(_: &StateRecord) -> f32 {
    return 0.0;
}

#[allow(dead_code)]
pub fn one(_: &StateRecord) -> f32 {
    return 1.0;
}

#[allow(dead_code)]
pub fn wins_plus_1_ratio(state_record: &StateRecord) -> f32 {
    let visits_count = utilities::count_visits(&state_record);
    return (state_record.wins_count + 1) as f32 / (visits_count + 1) as f32;
}

#[allow(dead_code)]
pub fn wins_minus_losses_floored(state_record: &StateRecord) -> f32 {
    return cmp::max(1, state_record.wins_count - state_record.losses_count) as f32;
}

#[allow(dead_code)]
pub fn create_linear_weighted_closure(
    draws_weight: f32,
    losses_weight: f32,
    wins_weight: f32,
) -> impl Fn(&StateRecord) -> f32 {
    return move |state_record| {
        let final_weight = draws_weight * state_record.draws_count as f32
            + losses_weight * state_record.losses_count as f32
            + wins_weight * state_record.wins_count as f32;

        if final_weight < 1.0 {
            return 1.0;
        }

        return final_weight;
    };
}

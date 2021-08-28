use super::state_record::StateRecord;
use rand;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::cmp;
use std::collections::HashMap;
use std::hash::Hash;

pub fn execute_standard_turn_based_game<State, HashedState: Eq + Hash>(
    initial_state: State,
    players_count: i32,
    get_state_record: fn(state: &HashedState) -> Option<StateRecord>,
    hash_state: fn(current_player_index: i32, state: &State) -> HashedState,
    update_state_record: fn(state_hash: &HashedState, did_draw: bool, did_win: bool),
    find_available_states: fn(current_player_index: i32, state: &State) -> &[State],
    weigh_record: fn(state_record: &StateRecord) -> f32,
    weight_visits: fn(
        visits_for_state: i32,
        min_available_visits: i32,
        max_available_visits: i32,
    ) -> f32,
    get_winning_player_index_for_state: fn(state: &State) -> Option<i32>,
) {
    let mut states_paths_by_player: Vec<Vec<HashedState>> = vec![];
    for _ in 0..players_count {
        states_paths_by_player.push(vec![]);
    }

    let mut current_player_index = 0;
    let mut current_state = &initial_state;
    let mut winning_player_index = -1;

    while winning_player_index == -1 {
        let available_states = &find_available_states(current_player_index, &current_state);
        let decide_next_state_result = decide_next_state(
            current_player_index,
            get_state_record,
            hash_state,
            available_states,
            weigh_record,
            weight_visits,
        );

        match decide_next_state_result {
            Ok(next_state) => {
                let next_state_hash = hash_state(current_player_index, &next_state);
                states_paths_by_player[current_player_index as usize].push(next_state_hash);
                current_player_index = (current_player_index + 1) % players_count;
                current_state = next_state;
                match get_winning_player_index_for_state(&next_state) {
                    None => (),
                    Some(result_index) => (winning_player_index = result_index),
                }
            }
            _ => {
                panic!("How did we end up with no available states but not a terminal condition!?")
            }
        }
    }

    update_state_records(
        update_state_record,
        &states_paths_by_player,
        winning_player_index,
    );
}

enum DecideNextStateError {
    NoAvailableStatesError,
}

fn decide_next_state<State, HashedState: Eq + Hash>(
    current_player_index: i32,
    get_state_record: fn(state: &HashedState) -> Option<StateRecord>,
    hash_state: fn(current_player_index: i32, state: &State) -> HashedState,
    available_states: &[State],
    weigh_record: fn(state_record: &StateRecord) -> f32,
    weight_visits: fn(
        visits_for_state: i32,
        min_available_visits: i32,
        max_available_visits: i32,
    ) -> f32,
) -> Result<&State, DecideNextStateError> {
    let available_states_count = available_states.len();
    if available_states_count == 0 {
        return Err(DecideNextStateError::NoAvailableStatesError);
    }

    let mut max_available_visits = 0;
    let mut min_available_visits = i32::MAX;

    let mut available_state_records_by_hash: HashMap<HashedState, StateRecord> = HashMap::new();

    for available_state in available_states.iter() {
        let available_state_hash = hash_state(current_player_index, &available_state);

        match get_state_record(&available_state_hash) {
            None => (),
            Some(available_state_record) => {
                let visits_count = available_state_record.draws_count
                    + available_state_record.losses_count
                    + available_state_record.wins_count;
                max_available_visits = cmp::max(max_available_visits, visits_count);
                min_available_visits = cmp::min(min_available_visits, visits_count);

                available_state_records_by_hash
                    .insert(available_state_hash, available_state_record);
            }
        }
    }

    let mut available_state_weights = vec![0.0; available_states_count];
    for (i, available_state) in available_states.iter().enumerate() {
        let mut available_state_record = &StateRecord::new(0, 0, 0);

        let available_state_hash = hash_state(current_player_index, &available_state);
        match available_state_records_by_hash.get(&available_state_hash) {
            None => (),
            Some(existing_value) => {
                available_state_record = &existing_value;
            }
        }

        let visits_count = available_state_record.draws_count
            + available_state_record.losses_count
            + available_state_record.wins_count;

        let available_state_record_weight = weigh_record(available_state_record);
        let available_state_visits_weight =
            weight_visits(visits_count, min_available_visits, max_available_visits);
        let available_state_total_weight =
            available_state_record_weight + available_state_visits_weight;
        available_state_weights[i] = available_state_total_weight;
    }

    let dist = WeightedIndex::new(&available_state_weights).unwrap();
    let mut rng = thread_rng();
    let chosen_state = &available_states[dist.sample(&mut rng)];
    return Ok(chosen_state);
}

fn update_state_records<HashedState: Eq + Hash>(
    update_state_record: fn(state_hash: &HashedState, did_draw: bool, did_win: bool),
    states_paths_by_player: &[Vec<HashedState>],
    winning_player_index: i32,
) {
    let did_draw = winning_player_index == -1;
    for (player_index, states_path_for_player) in states_paths_by_player.iter().enumerate() {
        let did_win = winning_player_index == player_index as i32;
        for state_hash in states_path_for_player.iter() {
            update_state_record(&state_hash, did_draw, did_win)
        }
    }
}

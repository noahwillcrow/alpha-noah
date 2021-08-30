use crate::core::state_record::StateRecord;
use crate::core::state_record_provider::StateRecordProvider;
use crate::core::utilities;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::cmp;
use std::collections::HashMap;
use std::hash::Hash;

pub fn execute_standard_turn_based_game<
    State: Clone,
    HashedState: Eq + Hash + Clone,
    WeighRecordFn: Fn(&StateRecord) -> f32,
    WeighVisitsFn: Fn(i32, i32, i32) -> f32,
>(
    initial_state: State,
    number_of_players: i32,
    state_record_provider: &mut dyn StateRecordProvider<HashedState>,
    hash_state: fn(current_player_index: i32, state: &State) -> HashedState,
    fill_vector_with_available_states: fn(
        current_player_index: i32,
        current_state: &State,
        available_states: &mut Vec<State>,
    ),
    weigh_record: &WeighRecordFn,
    weight_visits: &WeighVisitsFn,
    get_terminal_state: fn(current_player_index: i32, state: &State) -> Option<i32>,
    max_number_of_turns: i32,
    is_reaching_max_number_of_turns_a_draw: bool,
) -> i32 {
    let mut states_paths_by_player: Vec<Vec<HashedState>> = vec![];
    for _ in 0..number_of_players {
        states_paths_by_player.push(vec![]);
    }

    let mut current_player_index = 0;
    let mut current_state = initial_state;
    let mut winning_player_index = -2; // -1 represents a draw, -2 represents undetermined
    let mut turns_counter = 0;

    while winning_player_index == -2 && turns_counter < max_number_of_turns {
        turns_counter += 1;

        let mut available_states: Vec<State> = vec![];
        fill_vector_with_available_states(
            current_player_index,
            &current_state,
            &mut available_states,
        );

        let decide_next_state_index_result = decide_next_state_index(
            current_player_index,
            state_record_provider,
            hash_state,
            &available_states,
            weigh_record,
            weight_visits,
        );

        match decide_next_state_index_result {
            Ok(next_state_index) => {
                let next_state = available_states[next_state_index].clone();
                let next_state_hash = hash_state(current_player_index, &next_state);
                states_paths_by_player[current_player_index as usize].push(next_state_hash);
                current_player_index = (current_player_index + 1) % number_of_players;
                current_state = next_state;
                match get_terminal_state(current_player_index, &current_state) {
                    None => (),
                    Some(result_index) => {
                        winning_player_index = result_index;
                    }
                }
            }
            _ => {
                panic!("How did we end up with no available states but not a terminal condition!?")
            }
        }
    }

    if winning_player_index == -2
        && is_reaching_max_number_of_turns_a_draw
        && turns_counter == max_number_of_turns
    {
        winning_player_index = -1;
    }

    if winning_player_index > -2 {
        update_state_records(
            state_record_provider,
            &states_paths_by_player,
            winning_player_index,
            number_of_players,
        );
    }

    return winning_player_index;
}

enum DecideNextStateError {
    NoAvailableStatesError,
}

fn decide_next_state_index<
    State,
    HashedState: Eq + Hash + Clone,
    WeighRecordFn: Fn(&StateRecord) -> f32,
    WeighVisitsFn: Fn(i32, i32, i32) -> f32,
>(
    current_player_index: i32,
    state_record_provider: &dyn StateRecordProvider<HashedState>,
    hash_state: fn(current_player_index: i32, state: &State) -> HashedState,
    available_states: &[State],
    weigh_record: &WeighRecordFn,
    weight_visits: &WeighVisitsFn,
) -> Result<usize, DecideNextStateError> {
    let available_states_count = available_states.len();
    if available_states_count == 0 {
        return Err(DecideNextStateError::NoAvailableStatesError);
    }

    let mut max_available_visits = 0;
    let mut min_available_visits = i32::MAX;

    let mut available_state_records_by_hash: HashMap<HashedState, StateRecord> = HashMap::new();

    for available_state in available_states.iter() {
        let available_state_hash = hash_state(current_player_index, &available_state);

        match state_record_provider
            .get_state_record(current_player_index, available_state_hash.clone())
        {
            None => (),
            Some(available_state_record) => {
                let visits_count = utilities::count_visits(&available_state_record);
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

        let visits_count = utilities::count_visits(&available_state_record);

        let available_state_record_weight = weigh_record(available_state_record);
        let available_state_visits_weight =
            weight_visits(visits_count, min_available_visits, max_available_visits);
        let available_state_total_weight =
            available_state_record_weight + available_state_visits_weight;
        available_state_weights[i] = available_state_total_weight;
    }

    let dist = WeightedIndex::new(&available_state_weights).unwrap();
    let mut rng = thread_rng();
    return Ok(dist.sample(&mut rng));
}

fn update_state_records<HashedState: Eq + Hash + Clone>(
    state_record_provider: &mut dyn StateRecordProvider<HashedState>,
    states_paths_by_player: &[Vec<HashedState>],
    winning_player_index: i32,
    number_of_players: i32,
) {
    for states_path_for_player in states_paths_by_player.iter() {
        for state_hash in states_path_for_player.iter() {
            state_record_provider.update_state_record(
                &state_hash,
                winning_player_index,
                number_of_players,
            )
        }
    }
}

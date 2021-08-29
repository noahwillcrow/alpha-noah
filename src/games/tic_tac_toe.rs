use crate::core::state_record::StateRecord;
use crate::core::state_record_provider::StateRecordProvider;
use std::collections::HashMap;

type Position = (usize, usize);
static WINNING_POSITION_SETS: &'static [(Position, Position, Position)] = &[
    // rows
    ((0, 0), (0, 1), (0, 2)),
    ((1, 0), (1, 1), (1, 2)),
    ((2, 0), (2, 1), (2, 2)),
    // columns
    ((0, 0), (1, 0), (2, 0)),
    ((0, 1), (1, 1), (2, 1)),
    ((0, 2), (1, 2), (2, 2)),
    // diagonals
    ((0, 0), (1, 1), (2, 2)),
    ((0, 2), (1, 1), (2, 0)),
];

pub type TicTacToeState = Vec<Vec<i32>>;

pub struct TicTacToeStateRecordProvider {
    state_records: HashMap<String, StateRecord>,
}

impl StateRecordProvider<String> for TicTacToeStateRecordProvider {
    fn get_state_record(&self, state_hash: String) -> Option<StateRecord> {
        match self.state_records.get(&state_hash[..]) {
            Some(&state_record) => Some(state_record.clone()),
            None => None,
        }
    }

    fn update_state_record(&mut self, state_hash: &String, did_win: bool, did_draw: bool) {
        // let's not let one player ever learn!
        // if state_hash[..1].eq("1") {
        //     return;
        // }

        // no players ever learn!
        // return;

        let new_state_record: StateRecord;

        match self.state_records.get(&state_hash[..]) {
            Some(&state_record) => {
                new_state_record = StateRecord::new(
                    state_record.draws_count + (if did_draw { 1 } else { 0 }),
                    state_record.losses_count,
                    state_record.wins_count + (if did_win { 1 } else { 0 }),
                )
            }
            None => {
                new_state_record =
                    StateRecord::new(if did_draw { 1 } else { 0 }, 0, if did_win { 1 } else { 0 })
            }
        }

        self.state_records
            .insert(state_hash.clone(), new_state_record);
    }
}

pub fn create_initial_state() -> TicTacToeState {
    return vec![vec![-1, -1, -1], vec![-1, -1, -1], vec![-1, -1, -1]];
}

pub fn create_state_record_provider() -> impl StateRecordProvider<String> {
    return TicTacToeStateRecordProvider {
        state_records: HashMap::new(),
    };
}

pub fn fill_vector_with_available_states(
    current_player_index: i32,
    current_state: &TicTacToeState,
    available_states: &mut Vec<TicTacToeState>,
) {
    for i in 0..current_state.len() {
        for j in 0..current_state.len() {
            if current_state[i][j] == -1 {
                // the space is empty
                let mut available_state = current_state.clone();
                available_state[i][j] = current_player_index;
                available_states.push(available_state);
            }
        }
    }
}

pub fn get_terminal_state(state: &TicTacToeState) -> Option<i32> {
    for winning_position_set in WINNING_POSITION_SETS.iter() {
        let position_values = (
            state[winning_position_set.0 .0][winning_position_set.0 .1],
            state[winning_position_set.1 .0][winning_position_set.1 .1],
            state[winning_position_set.2 .0][winning_position_set.2 .1],
        );

        if position_values.0 > -1
            && position_values.0 == position_values.1
            && position_values.0 == position_values.2
        {
            return Some(position_values.0 as i32);
        }
    }

    let mut is_board_full = true;
    'rows: for i in 0..state.len() {
        for j in 0..state.len() {
            if state[i][j] == -1 {
                is_board_full = false;
                break 'rows;
            }
        }
    }

    if is_board_full {
        return Some(-1);
    }

    return None;
}

pub fn hash_state(current_player_index: i32, current_state: &TicTacToeState) -> String {
    let mut state_raw_value: i32 = 0;
    let mut ternary_digit_multiplier: i32 = 1;
    for i in 0..current_state.len() {
        for j in 0..current_state.len() {
            let location_value: i32 = current_state[i][j] + 1;
            state_raw_value += location_value * ternary_digit_multiplier;
            ternary_digit_multiplier *= 3;
        }
    }

    let state_hash = format!("{}-{:x}", current_player_index, state_raw_value);
    return state_hash;
}

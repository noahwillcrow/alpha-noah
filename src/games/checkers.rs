use crate::core::state_record::StateRecord;
use crate::core::state_record_provider::StateRecordProvider;
use byte_string::ByteString;
use std::collections::{HashMap, HashSet};

// Working state format is a 2D array of bytes with 0 for unoccupied, 1 for first player's standard piece, 11 for first player's double piece,
// 2 for the second player's piece, and 2 for the second player's double piece

// Each state hashes to a maximum of 25 bytes - a byte to represent who's turn just finished (just 0 or 1) and one byte per piece up to 24 pieces.
// Each piece is hashed to use two bits to represent its type:
// - 00 for first player standard
// - 01 for first player double
// - 10 for second player standard
// - 11 for second player double
// The other six bits come afterwards and are used to represent the location on the 8x8 board (2^6 = 64 = 8x8)

const MAX_ROW: i8 = 7;
const MAX_COL: i8 = 7;

struct MoveSearchParameters {
    pub single_piece_value: u8,
    pub double_piece_value: u8,
    pub single_piece_available_directions: [(i8, i8); 2],
    pub double_piece_available_directions: [(i8, i8); 4],
    pub double_row: usize,
}

pub type CheckersState = Vec<Vec<u8>>;

pub struct CheckersStateRecordProvider {
    state_records: HashMap<ByteString, StateRecord>,
}

impl StateRecordProvider<ByteString> for CheckersStateRecordProvider {
    fn get_state_record(&self, state_hash: ByteString) -> Option<StateRecord> {
        match self.state_records.get(&state_hash[..]) {
            Some(&state_record) => Some(state_record.clone()),
            None => None,
        }
    }

    fn update_state_record(&mut self, state_hash: &ByteString, did_win: bool, did_draw: bool) {
        // let's not let one player ever learn!
        // if state_hash[0] == 1 {
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

pub fn create_initial_state() -> CheckersState {
    return vec![
        vec![0, 1, 0, 1, 0, 1, 0, 1],
        vec![1, 0, 1, 0, 1, 0, 1, 0],
        vec![0, 1, 0, 1, 0, 1, 0, 1],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![2, 0, 2, 0, 2, 0, 2, 0],
        vec![0, 2, 0, 2, 0, 2, 0, 2],
        vec![2, 0, 2, 0, 2, 0, 2, 0],
    ];
}

pub fn create_state_record_provider() -> impl StateRecordProvider<ByteString> {
    return CheckersStateRecordProvider {
        state_records: HashMap::new(),
    };
}

pub fn fill_vector_with_available_states(
    current_player_index: i32,
    current_state: &CheckersState,
    available_states: &mut Vec<CheckersState>,
) {
    let mut available_simple_move_states: Vec<CheckersState> = vec![];
    let mut available_capture_move_states: Vec<CheckersState> = vec![];

    let move_search_params = get_player_specific_move_search_parameters(current_player_index);
    let mut owned_pieces_info: Vec<((usize, usize), &[(i8, i8)])> = vec![];
    fill_vector_with_current_player_owned_pieces_info_for_move_search(
        current_state,
        &move_search_params,
        &mut owned_pieces_info,
    );

    for (current_coor, available_directions) in owned_pieces_info.iter() {
        // find simple moves
        fill_vector_with_available_simple_move_states_for_piece(
            current_state,
            current_coor,
            available_directions,
            move_search_params.single_piece_value,
            move_search_params.double_piece_value,
            move_search_params.double_row,
            &mut available_simple_move_states,
        );

        // find capture moves
        fill_vector_with_available_capture_move_states_for_piece(
            current_player_index,
            current_state,
            current_coor,
            available_directions,
            move_search_params.single_piece_value,
            move_search_params.double_piece_value,
            move_search_params.double_row,
            &mut available_capture_move_states,
        );
    }

    if available_capture_move_states.is_empty() {
        available_states.append(&mut available_simple_move_states);
    } else {
        available_states.append(&mut available_capture_move_states);
    }
}

pub fn get_terminal_state(current_player_index: i32, state: &CheckersState) -> Option<i32> {
    let mut next_turn_available_states: Vec<CheckersState> = vec![];

    fill_vector_with_available_states(current_player_index, state, &mut next_turn_available_states);

    if next_turn_available_states.is_empty() {
        return Some(current_player_index);
    }

    return None;
}

pub fn hash_state(current_player_index: i32, current_state: &CheckersState) -> ByteString {
    let mut state_hash_bytes = vec![current_player_index as u8];

    for i in 0..current_state.len() {
        for j in 0..current_state.len() {
            let position_value = current_state[i][j];
            if position_value > 0 {
                let mut piece_byte = (i * j) as u8;

                match position_value {
                    1 => (),
                    11 => piece_byte &= 0b01_00_00_00,
                    2 => piece_byte &= 0b10_00_00_00,
                    22 => piece_byte &= 0b11_00_00_00,
                    _ => panic!(
                        "Encountered illegal position value {} at position ({}, {})",
                        position_value, i, j
                    ),
                }

                state_hash_bytes.push(piece_byte);
            }
        }
    }

    return ByteString::from(state_hash_bytes);
}

fn is_valid_board_coordinate(row: i8, col: i8) -> bool {
    return row >= 0 && row <= MAX_ROW && col >= 0 && col <= MAX_COL;
}

fn get_player_specific_move_search_parameters(current_player_index: i32) -> MoveSearchParameters {
    let forward_row_direction: i8 = if current_player_index == 0 { 1 } else { -1 };

    return MoveSearchParameters {
        single_piece_value: (if current_player_index == 0 { 1 } else { 2 }) as u8,
        double_piece_value: if current_player_index == 0 { 11 } else { 22 },
        single_piece_available_directions: [
            (forward_row_direction, 1 as i8),
            (forward_row_direction, -1 as i8),
        ],
        double_piece_available_directions: [
            (forward_row_direction, 1 as i8),
            (forward_row_direction, -1 as i8),
            (-forward_row_direction, 1 as i8),
            (-forward_row_direction, -1 as i8),
        ],
        double_row: if current_player_index == 0 {
            MAX_ROW as usize
        } else {
            0
        },
    };
}

fn fill_vector_with_current_player_owned_pieces_info_for_move_search<'l>(
    current_state: &CheckersState,
    move_search_params: &'l MoveSearchParameters,
    owned_pieces_info: &mut Vec<((usize, usize), &'l [(i8, i8)])>,
) {
    for row in 0..current_state.len() {
        for col in 0..current_state[row].len() {
            let current_state_space_value = current_state[row][col];
            let available_directions: &[(i8, i8)];
            if current_state_space_value == move_search_params.single_piece_value {
                available_directions = &move_search_params.single_piece_available_directions;
            } else if current_state_space_value == move_search_params.double_piece_value {
                available_directions = &move_search_params.double_piece_available_directions;
            } else {
                continue;
            }

            let current_space_coor = (row, col);
            owned_pieces_info.push((current_space_coor, available_directions));
        }
    }
}

fn fill_vector_with_available_simple_move_states_for_piece(
    current_state: &CheckersState,
    current_coor: &(usize, usize),
    available_directions: &[(i8, i8)],
    single_piece_value: u8,
    double_piece_value: u8,
    double_row: usize,
    available_simple_move_states: &mut Vec<CheckersState>,
) {
    let current_state_space_value = current_state[current_coor.0][current_coor.1];

    for direction in available_directions.iter() {
        let simple_move_coor = (
            current_coor.0 as i8 + direction.0,
            current_coor.1 as i8 + direction.1,
        );

        if !is_valid_board_coordinate(simple_move_coor.0, simple_move_coor.1) {
            continue;
        }

        let simple_move_space_value =
            current_state[simple_move_coor.0 as usize][simple_move_coor.1 as usize];
        if simple_move_space_value == 0 {
            let mut simple_move_state = current_state.clone();
            simple_move_state[current_coor.0][current_coor.1] = 0;

            if current_state_space_value == single_piece_value
                && simple_move_coor.0 as usize == double_row
            {
                // doublin' that piece!
                simple_move_state[simple_move_coor.0 as usize][simple_move_coor.1 as usize] =
                    double_piece_value;
            } else {
                simple_move_state[simple_move_coor.0 as usize][simple_move_coor.1 as usize] =
                    current_state_space_value;
            }

            available_simple_move_states.push(simple_move_state);
        }
    }
}

fn fill_vector_with_available_capture_move_states_for_piece(
    current_player_index: i32,
    current_state: &CheckersState,
    current_coor: &(usize, usize),
    available_directions: &[(i8, i8)],
    single_piece_value: u8,
    double_piece_value: u8,
    double_row: usize,
    available_capture_move_states: &mut Vec<CheckersState>,
) {
    let mut capture_possibilities_stack: Vec<(CheckersState, (usize, usize), (i8, i8))> = vec![];
    for direction in available_directions.iter() {
        capture_possibilities_stack.push((
            current_state.clone(),
            (current_coor.0, current_coor.1),
            (direction.0, direction.1),
        ));
    }

    let mut visited_state_hashes: HashSet<ByteString> = HashSet::new();

    'inf_loop: loop {
        match capture_possibilities_stack.pop() {
            Some((start_state, start_space_coor, capture_dir)) => {
                let start_state_hash = hash_state(current_player_index, &start_state);
                if visited_state_hashes.contains(&start_state_hash) {
                    // already visited this state and explored it
                    continue 'inf_loop;
                }
                visited_state_hashes.insert(start_state_hash);

                if !is_valid_board_coordinate(
                    start_space_coor.0 as i8 + (capture_dir.0 * 2),
                    start_space_coor.1 as i8 + (capture_dir.1 * 2),
                ) {
                    // not a valid jump as it would go out of bounds
                    continue 'inf_loop;
                }

                let simple_move_coor = (
                    start_space_coor.0 as i8 + capture_dir.0,
                    start_space_coor.1 as i8 + capture_dir.1,
                );
                let simple_move_space_value =
                    current_state[simple_move_coor.0 as usize][simple_move_coor.1 as usize];
                if simple_move_space_value == single_piece_value
                    || simple_move_space_value == double_piece_value
                {
                    // not a valid jump as you jump over your own pieces
                    continue 'inf_loop;
                }

                let capture_move_space_coor = (
                    (start_space_coor.0 as i8 + (capture_dir.0 * 2)) as usize,
                    (start_space_coor.1 as i8 + (capture_dir.1 * 2)) as usize,
                );
                let capture_move_space_value = start_state[capture_move_space_coor.0 as usize]
                    [capture_move_space_coor.1 as usize];
                if capture_move_space_value > 0 {
                    // not a valid jump as the space two ahead is not empty
                    continue 'inf_loop;
                }

                // a capture is possible!
                let mut capture_move_state = start_state.clone();
                capture_move_state[start_space_coor.0][start_space_coor.1] = 0;

                let captured_piece_space_coor = (
                    (start_space_coor.0 as i8 + capture_dir.0) as usize,
                    (start_space_coor.1 as i8 + capture_dir.1) as usize,
                );
                capture_move_state[captured_piece_space_coor.0][captured_piece_space_coor.1] = 0;

                let start_state_space_value = start_state[start_space_coor.0][start_space_coor.1];
                if start_state_space_value == single_piece_value
                    && capture_move_space_coor.0 == double_row
                {
                    // doublin' that piece!
                    capture_move_state[capture_move_space_coor.0][capture_move_space_coor.1] =
                        double_piece_value;
                    // when doubling a piece, the turn ends there - no further jumps allowed
                } else {
                    capture_move_state[capture_move_space_coor.0 as usize]
                        [capture_move_space_coor.1 as usize] = start_state_space_value;

                    // now let's explore for multi-jump possibilities by pushing them onto the stack
                    for next_jump_direction in available_directions {
                        capture_possibilities_stack.push((
                            capture_move_state.clone(),
                            capture_move_space_coor,
                            (next_jump_direction.0, next_jump_direction.1),
                        ));
                    }
                }

                available_capture_move_states.push(capture_move_state);
            }
            None => break 'inf_loop,
        }
    }
}

use crate::games::checkers::GameStateType as CheckersGameState;
use std::collections::HashSet;

pub const MAX_ROW: i8 = 7;
pub const MAX_COL: i8 = 7;

pub struct MoveSearchParameters {
    pub forward_row_direction: i8,
    pub single_piece_value: u8,
    pub double_piece_value: u8,
    pub single_piece_available_directions: [(i8, i8); 2],
    pub double_piece_available_directions: [(i8, i8); 4],
    pub double_row: usize,
}

pub fn fill_vector_with_current_player_owned_pieces_info_for_move_search<'l>(
    current_game_state: &CheckersGameState,
    move_search_params: &'l MoveSearchParameters,
    owned_pieces_info: &mut Vec<((usize, usize), &'l [(i8, i8)])>,
) {
    for row in 0..current_game_state.len() {
        for col in 0..current_game_state[row].len() {
            let current_game_state_space_value = current_game_state[row][col];
            let available_directions: &[(i8, i8)];
            if current_game_state_space_value == move_search_params.single_piece_value {
                available_directions = &move_search_params.single_piece_available_directions;
            } else if current_game_state_space_value == move_search_params.double_piece_value {
                available_directions = &move_search_params.double_piece_available_directions;
            } else {
                continue;
            }

            let current_space_coor = (row, col);
            owned_pieces_info.push((current_space_coor, available_directions));
        }
    }
}

pub fn fill_vector_with_available_simple_move_states_for_piece(
    current_game_state: &CheckersGameState,
    start_coor: &(usize, usize),
    available_directions: &[(i8, i8)],
    single_piece_value: u8,
    double_piece_value: u8,
    double_row: usize,
    available_simple_move_states: &mut Vec<CheckersGameState>,
    max_number_of_moves_to_find: i32,
) {
    let current_game_state_space_value = current_game_state[start_coor.0][start_coor.1];

    for direction in available_directions.iter() {
        let simple_move_coor = (
            start_coor.0 as i8 + direction.0,
            start_coor.1 as i8 + direction.1,
        );

        if !is_valid_board_coordinate(simple_move_coor.0, simple_move_coor.1) {
            continue;
        }

        let simple_move_space_value =
            current_game_state[simple_move_coor.0 as usize][simple_move_coor.1 as usize];
        if simple_move_space_value == 0 {
            let mut simple_move_state = current_game_state.clone();
            simple_move_state[start_coor.0][start_coor.1] = 0;

            if current_game_state_space_value == single_piece_value
                && simple_move_coor.0 as usize == double_row
            {
                // doublin' that piece!
                simple_move_state[simple_move_coor.0 as usize][simple_move_coor.1 as usize] =
                    double_piece_value;
            } else {
                simple_move_state[simple_move_coor.0 as usize][simple_move_coor.1 as usize] =
                    current_game_state_space_value;
            }

            available_simple_move_states.push(simple_move_state);

            if available_simple_move_states.len() == max_number_of_moves_to_find as usize {
                // we have found the max number of turns we want so let's just end here
                return;
            }
        }
    }
}

pub fn fill_vector_with_available_capture_move_states_for_piece(
    current_player_index: i32,
    current_game_state: &CheckersGameState,
    start_coor: &(usize, usize),
    available_directions: &[(i8, i8)],
    single_piece_value: u8,
    double_piece_value: u8,
    double_row: usize,
    available_capture_move_states: &mut Vec<CheckersGameState>,
    max_number_of_moves_to_find: i32,
) {
    let mut capture_possibilities_stack: Vec<(CheckersGameState, (usize, usize), (i8, i8))> =
        vec![];
    for direction in available_directions.iter() {
        capture_possibilities_stack.push((
            current_game_state.clone(),
            (start_coor.0, start_coor.1),
            (direction.0, direction.1),
        ));
    }

    let mut visited_options: HashSet<(Vec<u8>, (usize, usize), (i8, i8))> = HashSet::new();

    while let Some((start_game_state, move_from_coor, capture_dir)) =
        capture_possibilities_stack.pop()
    {
        let serialized_start_game_state =
            serialize_game_state(current_player_index, &start_game_state);
        let visited_options_key = (
            serialized_start_game_state.clone(),
            move_from_coor,
            capture_dir,
        );
        if visited_options.contains(&visited_options_key) {
            // already visited this state and explored it
            continue;
        }
        visited_options.insert(visited_options_key);

        if !is_valid_board_coordinate(
            move_from_coor.0 as i8 + (capture_dir.0 * 2),
            move_from_coor.1 as i8 + (capture_dir.1 * 2),
        ) {
            // not a valid jump as it would go out of bounds
            continue;
        }

        let captured_piece_coor = (
            (move_from_coor.0 as i8 + capture_dir.0) as usize,
            (move_from_coor.1 as i8 + capture_dir.1) as usize,
        );
        let captured_piece_space_value =
            current_game_state[captured_piece_coor.0][captured_piece_coor.1];
        if captured_piece_space_value == 0
            || captured_piece_space_value == single_piece_value
            || captured_piece_space_value == double_piece_value
        {
            // not a valid jump as you jump over your own pieces or empty spaces
            continue;
        }

        let capture_move_space_coor = (
            (move_from_coor.0 as i8 + (capture_dir.0 * 2)) as usize,
            (move_from_coor.1 as i8 + (capture_dir.1 * 2)) as usize,
        );
        let capture_move_space_value =
            start_game_state[capture_move_space_coor.0][capture_move_space_coor.1];
        if capture_move_space_value > 0 {
            // not a valid jump as the space two ahead is not empty
            continue;
        }

        // a capture is possible!
        let mut capture_move_state = start_game_state.clone();
        capture_move_state[move_from_coor.0][move_from_coor.1] = 0;
        capture_move_state[captured_piece_coor.0][captured_piece_coor.1] = 0;

        let start_game_state_space_value = start_game_state[move_from_coor.0][move_from_coor.1];
        if start_game_state_space_value == single_piece_value
            && capture_move_space_coor.0 == double_row
        {
            // doublin' that piece!
            capture_move_state[capture_move_space_coor.0][capture_move_space_coor.1] =
                double_piece_value;
            // when doubling a piece, the turn ends there - no further jumps allowed
        } else {
            capture_move_state[capture_move_space_coor.0 as usize]
                [capture_move_space_coor.1 as usize] = start_game_state_space_value;

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

        if available_capture_move_states.len() == max_number_of_moves_to_find as usize {
            // we have found the max number of turns we want so let's just end here
            return;
        }
    }
}

pub fn get_player_specific_move_search_parameters(
    current_player_index: i32,
) -> MoveSearchParameters {
    let forward_row_direction: i8 = if current_player_index == 0 { 1 } else { -1 };

    return MoveSearchParameters {
        forward_row_direction: forward_row_direction,
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

pub fn is_valid_board_coordinate(row: i8, col: i8) -> bool {
    return row >= 0 && row <= MAX_ROW && col >= 0 && col <= MAX_COL;
}

pub fn serialize_game_state(
    responsible_player_index: i32,
    game_state: &CheckersGameState,
) -> Vec<u8> {
    let mut number_of_pieces: u8 = 0;
    let mut serialized_game_state = vec![0 as u8];

    for i in 0..game_state.len() {
        for j in 0..game_state.len() {
            let position_value = game_state[i][j];
            if position_value > 0 {
                number_of_pieces += 1;

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

                serialized_game_state.push(piece_byte);
            }
        }
    }

    serialized_game_state[0] = ((responsible_player_index as u8) << 7) + number_of_pieces;

    return serialized_game_state;
}

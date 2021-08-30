// Working state format is a 2D array of bytes with 0 for unoccupied, 1 for first player's mark, and 2 for second player's mark

// Each state hashes to 2 bytes - just encoding the base_3 sum of the elements from the 2D array working state
// There are (3^9 - 1) total possible states according to a naive calculation (when only allowing for legal states, actually far fewer)
// log_2(3^9 - 1) < 16, so 2 bytes is sufficient to represent all possible values

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

pub type TicTacToeState = Vec<Vec<u8>>;

pub fn create_initial_state() -> TicTacToeState {
    return vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]];
}

pub fn fill_vector_with_available_states(
    current_player_index: i32,
    current_state: &TicTacToeState,
    available_states: &mut Vec<TicTacToeState>,
) {
    for i in 0..current_state.len() {
        for j in 0..current_state.len() {
            if current_state[i][j] == 0 {
                // the space is empty
                let mut available_state = current_state.clone();
                available_state[i][j] = current_player_index as u8 + 1;
                available_states.push(available_state);
            }
        }
    }
}

pub fn get_terminal_state(_: i32, state: &TicTacToeState) -> Option<i32> {
    for winning_position_set in WINNING_POSITION_SETS.iter() {
        let position_values = (
            state[winning_position_set.0 .0][winning_position_set.0 .1],
            state[winning_position_set.1 .0][winning_position_set.1 .1],
            state[winning_position_set.2 .0][winning_position_set.2 .1],
        );

        if position_values.0 > 0
            && position_values.0 == position_values.1
            && position_values.0 == position_values.2
        {
            return Some(position_values.0 as i32 - 1);
        }
    }

    let mut is_board_full = true;
    'rows: for i in 0..state.len() {
        for j in 0..state.len() {
            if state[i][j] == 0 {
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

pub fn hash_state(_: i32, current_state: &TicTacToeState) -> Vec<u8> {
    let mut state_raw_value: u16 = 0;
    let mut ternary_digit_multiplier: u16 = 1;
    for i in 0..current_state.len() {
        for j in 0..current_state.len() {
            let location_value = current_state[i][j] as u16;
            state_raw_value += location_value * ternary_digit_multiplier;
            ternary_digit_multiplier *= 3;
        }
    }

    return state_raw_value.to_be_bytes().to_vec();
}

pub fn create_new_state_from_user_input(
    current_player_index: i32,
    current_state: &TicTacToeState,
    user_input_str: &str,
) -> Result<TicTacToeState, i32> {
    let input_coor: (usize, usize);

    match &user_input_str[..3] {
        "0,0" => input_coor = (0, 0),
        "0,1" => input_coor = (0, 1),
        "0,2" => input_coor = (0, 2),
        "1,0" => input_coor = (1, 0),
        "1,1" => input_coor = (1, 1),
        "1,2" => input_coor = (1, 2),
        "2,0" => input_coor = (2, 0),
        "2,1" => input_coor = (2, 1),
        "2,2" => input_coor = (2, 2),
        _ => return Err(1),
    }

    let mut new_state = current_state.clone();
    if new_state[input_coor.0][input_coor.1] > 0 {
        return Err(2);
    }

    new_state[input_coor.0][input_coor.1] = current_player_index as u8 + 1;

    return Ok(new_state);
}

pub fn convert_state_to_cli_string(current_state: &TicTacToeState) -> String {
    return format!(
        "\
        {}|{}|{}\n\
        {}|{}|{}\n\
        {}|{}|{}\n\
    ",
        convert_space_value_to_cli_string(current_state[0][0]),
        convert_space_value_to_cli_string(current_state[0][1]),
        convert_space_value_to_cli_string(current_state[0][2]),
        convert_space_value_to_cli_string(current_state[1][0]),
        convert_space_value_to_cli_string(current_state[1][1]),
        convert_space_value_to_cli_string(current_state[1][2]),
        convert_space_value_to_cli_string(current_state[2][0]),
        convert_space_value_to_cli_string(current_state[2][1]),
        convert_space_value_to_cli_string(current_state[2][2])
    );
}

fn convert_space_value_to_cli_string(space_value: u8) -> String {
    return String::from(match space_value {
        1 => "x",
        2 => "o",
        _ => " ",
    });
}

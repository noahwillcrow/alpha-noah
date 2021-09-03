use crate::games::checkers::internal::*;
use crate::games::checkers::GameStateType as CheckersGameState;
use crate::traits::UserInputGameStateCreator as TUserInputGameStateCreator;
use regex::Regex;

const COORDINATE_CAPTURE_PATTERN: &str = r"\d,\d";
const VALID_INPUT_PATTERN: &str = r"^(\d,\d;)+(\d,\d)\n$";

pub struct UserInputGameStateCreator {
    coordinate_capture_regex: Regex,
    valid_input_regex: Regex,
}

impl UserInputGameStateCreator {
    pub fn new() -> UserInputGameStateCreator {
        return UserInputGameStateCreator {
            coordinate_capture_regex: Regex::new(COORDINATE_CAPTURE_PATTERN).unwrap(),
            valid_input_regex: Regex::new(VALID_INPUT_PATTERN).unwrap(),
        };
    }
}

impl TUserInputGameStateCreator<CheckersGameState, String> for UserInputGameStateCreator {
    fn create_new_game_state_from_user_input(
        &self,
        current_player_index: i32,
        current_game_state: &CheckersGameState,
        user_input: String,
    ) -> Result<CheckersGameState, String> {
        if !self.valid_input_regex.is_match(&user_input) {
            return Err(String::from("Invalid input format. Valid format is row,col;row,col;row,col where the first pair is the current space of the piece and the subsequent pairs are the moves."));
        }

        let mut input_coordinates: Vec<(usize, usize)> = vec![];

        for capture in self.coordinate_capture_regex.captures_iter(&user_input) {
            let input_coordinate_bytes = &mut capture[0].bytes();
            input_coordinates.push((
                (input_coordinate_bytes.nth(0).unwrap() - b'0') as usize,
                (input_coordinate_bytes.nth(1).unwrap() - b'0') as usize,
            ));
        }

        let player_move_search_parameters =
            get_player_specific_move_search_parameters(current_player_index);

        let start_coor = input_coordinates[0];
        let active_piece_space_value = current_game_state[start_coor.0][start_coor.1];
        if active_piece_space_value != player_move_search_parameters.single_piece_value
            && active_piece_space_value != player_move_search_parameters.double_piece_value
        {
            return Err(String::from("Illegal move. You do not own that piece."));
        }

        let first_move_coor = input_coordinates[1];

        let is_simple_move = (first_move_coor.0 as i8 - start_coor.0 as i8).abs() == 1
            && (first_move_coor.1 as i8 - start_coor.1 as i8).abs() == 1;
        if is_simple_move {
            if input_coordinates.len() > 2 {
                return Err(String::from(
                    "Illegal move. You must capture on each step to do more than one hop.",
                ));
            }

            return perform_simple_move(
                current_player_index,
                current_game_state,
                active_piece_space_value,
                start_coor,
                first_move_coor,
                &player_move_search_parameters,
            );
        }

        // must be a capture move
        return perform_capture_move(
            current_game_state,
            &input_coordinates,
            &player_move_search_parameters,
        );
    }
}

fn perform_simple_move(
    current_player_index: i32,
    current_game_state: &CheckersGameState,
    active_piece_space_value: u8,
    start_coor: (usize, usize),
    move_to_coor: (usize, usize),
    player_move_search_parameters: &MoveSearchParameters,
) -> Result<CheckersGameState, String> {
    let mut owned_pieces_info: Vec<((usize, usize), &[(i8, i8)])> = vec![];
    fill_vector_with_current_player_owned_pieces_info_for_move_search(
        current_game_state,
        &player_move_search_parameters,
        &mut owned_pieces_info,
    );

    let mut available_capture_move_states: Vec<CheckersGameState> = vec![];
    for (owned_piece_coor, available_directions) in owned_pieces_info.iter() {
        fill_vector_with_available_capture_move_states_for_piece(
            current_player_index,
            current_game_state,
            owned_piece_coor,
            available_directions,
            player_move_search_parameters.single_piece_value,
            player_move_search_parameters.double_piece_value,
            player_move_search_parameters.double_row,
            &mut available_capture_move_states,
            1,
        );

        if !available_capture_move_states.is_empty() {
            return Err(String::from(
                "Illegal move. When a capture is available, you must capture.",
            ));
        }
    }

    if !is_valid_board_coordinate(move_to_coor.0 as i8, move_to_coor.1 as i8) {
        return Err(format!(
            "Illegal move. The space ({},{}) is not a legal board location.",
            move_to_coor.0, move_to_coor.1
        ));
    }

    if current_game_state[move_to_coor.0][move_to_coor.1] != 0 {
        return Err(String::from(
            "Illegal move. You cannot move to an occupied space.",
        ));
    }

    let is_using_double_piece =
        active_piece_space_value == player_move_search_parameters.double_piece_value;

    if !is_using_double_piece {
        let row_direction = move_to_coor.0 as i8 - start_coor.0 as i8;
        if row_direction != player_move_search_parameters.forward_row_direction {
            return Err(String::from(
                "Illegal move. Only doubled pieces can move backwards.",
            ));
        }
    }

    let mut simple_move_state = current_game_state.clone();
    simple_move_state[start_coor.0][start_coor.1] = 0;

    if !is_using_double_piece && move_to_coor.0 as usize == player_move_search_parameters.double_row
    {
        // doublin' that piece!
        simple_move_state[move_to_coor.0 as usize][move_to_coor.1 as usize] =
            player_move_search_parameters.double_piece_value;
    } else {
        simple_move_state[move_to_coor.0 as usize][move_to_coor.1 as usize] =
            active_piece_space_value;
    }

    return Ok(simple_move_state);
}

fn perform_capture_move(
    current_game_state: &CheckersGameState,
    input_coordinates: &Vec<(usize, usize)>,
    player_move_search_parameters: &MoveSearchParameters,
) -> Result<CheckersGameState, String> {
    let start_coor = input_coordinates[0];
    let mut active_piece_space_value = current_game_state[start_coor.0][start_coor.1];
    let is_using_double_piece =
        active_piece_space_value == player_move_search_parameters.double_piece_value;

    let mut move_from_coor = start_coor;
    let mut working_game_state = current_game_state.clone();

    for i in 1..input_coordinates.len() {
        let move_to_coor = input_coordinates[i];
        if !is_valid_board_coordinate(move_to_coor.0 as i8, move_to_coor.1 as i8) {
            return Err(format!(
                "Illegal move. The space ({},{}) is not a legal board location.",
                move_to_coor.0, move_to_coor.1
            ));
        }
        let move_displacement = (
            move_to_coor.0 as i8 - move_from_coor.0 as i8,
            move_to_coor.1 as i8 - move_from_coor.1 as i8,
        );

        if move_displacement.0.abs() != 2 || move_displacement.1.abs() != 2 {
            return Err(String::from(
                "Illegal move. All captures must result in moving 2 rows and 2 columns at once.",
            ));
        }

        let move_direction = (move_displacement.0 / 2, move_displacement.1 / 2);
        if !is_using_double_piece
            && move_direction.0 != player_move_search_parameters.forward_row_direction
        {
            return Err(String::from(
                "Illegal move. Only doubled pieces can move backwards.",
            ));
        }

        let captured_piece_coor = (
            (move_from_coor.0 as i8 + move_direction.0) as usize,
            (move_from_coor.1 as i8 + move_direction.1) as usize,
        );
        let captured_piece_value = working_game_state[captured_piece_coor.0][captured_piece_coor.1];
        if captured_piece_value == 0 {
            return Err(String::from(
                "Illegal move. You must capture on every hop during a capture move.",
            ));
        }

        if captured_piece_value == player_move_search_parameters.single_piece_value
            || captured_piece_value == player_move_search_parameters.double_piece_value
        {
            return Err(String::from(
                "Illegal move. You cannot hop over your own pieces.",
            ));
        }

        let move_to_space_value = working_game_state[move_to_coor.0][move_to_coor.1];
        if move_to_space_value != 0 {
            return Err(String::from(
                "Illegal move. You cannot move to an occupied space.",
            ));
        }

        let will_piece_become_doubled =
            !is_using_double_piece && move_to_coor.0 == player_move_search_parameters.double_row;
        if will_piece_become_doubled && i < input_coordinates.len() - 1 {
            return Err(String::from(
                "Illegal move. You cannot move again after doubling a piece.",
            ));
        }

        if will_piece_become_doubled {
            active_piece_space_value = player_move_search_parameters.double_piece_value;
        }

        // It's a legal hop!
        working_game_state[move_from_coor.0][move_from_coor.1] = 0;
        working_game_state[captured_piece_coor.0][captured_piece_coor.1] = 0;
        working_game_state[move_to_coor.0][move_to_coor.1] = active_piece_space_value;

        move_from_coor = move_to_coor;
    }

    return Ok(working_game_state);
}

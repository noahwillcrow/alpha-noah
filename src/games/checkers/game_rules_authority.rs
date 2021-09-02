use crate::games::checkers::internal::*;
use crate::games::checkers::GameStateType as CheckersGameState;
use crate::traits::GameRulesAuthority as TGameRulesAuthority;

const MAX_NUMBER_OF_MOVES_TO_FIND: i32 = 10_000;

pub struct GameRulesAuthority {}

impl TGameRulesAuthority<CheckersGameState> for GameRulesAuthority {
    fn analyze_game_state_for_terminality(
        &self,
        game_state: &CheckersGameState,
        next_player_index: i32,
    ) -> Option<i32> {
        let move_search_params = get_player_specific_move_search_parameters(next_player_index);
        let mut owned_pieces_info: Vec<((usize, usize), &[(i8, i8)])> = vec![];
        fill_vector_with_current_player_owned_pieces_info_for_move_search(
            game_state,
            &move_search_params,
            &mut owned_pieces_info,
        );

        let mut available_move_states: Vec<CheckersGameState> = vec![];

        // see if any simple moves exist for any pieces first since that's cheaper
        for (current_coor, available_directions) in owned_pieces_info.iter() {
            fill_vector_with_available_simple_move_states_for_piece(
                game_state,
                current_coor,
                available_directions,
                move_search_params.single_piece_value,
                move_search_params.double_piece_value,
                move_search_params.double_row,
                &mut available_move_states,
                1,
            );
            if !available_move_states.is_empty() {
                // at least one move exists so this player hasn't lost yet
                return None;
            }
        }

        // no simple moves were found so let's see if we have
        // any capture moves (which are more expensive to find) are available
        for (current_coor, available_directions) in owned_pieces_info.iter() {
            fill_vector_with_available_capture_move_states_for_piece(
                next_player_index,
                game_state,
                current_coor,
                available_directions,
                move_search_params.single_piece_value,
                move_search_params.double_piece_value,
                move_search_params.double_row,
                &mut available_move_states,
                1,
            );
            if !available_move_states.is_empty() {
                // at least one move exists so this player hasn't lost yet
                return None;
            }
        }

        // no available moves have been found
        // so the next player can't move for their coming turn
        // which means they lost
        // so let's return Some(other_player_index) to report that the game has ended and the other player has won
        let other_player_index = (next_player_index + 1) % 2;
        return Some(other_player_index);
    }

    fn find_available_next_game_states(
        &self,
        current_player_index: i32,
        current_game_state: &CheckersGameState,
    ) -> Vec<CheckersGameState> {
        let mut available_next_states: Vec<CheckersGameState> = vec![];

        let move_search_params = get_player_specific_move_search_parameters(current_player_index);
        let mut owned_pieces_info: Vec<((usize, usize), &[(i8, i8)])> = vec![];
        fill_vector_with_current_player_owned_pieces_info_for_move_search(
            current_game_state,
            &move_search_params,
            &mut owned_pieces_info,
        );

        // find any available capture moves
        for (current_coor, available_directions) in owned_pieces_info.iter() {
            fill_vector_with_available_capture_move_states_for_piece(
                current_player_index,
                current_game_state,
                current_coor,
                available_directions,
                move_search_params.single_piece_value,
                move_search_params.double_piece_value,
                move_search_params.double_row,
                &mut available_next_states,
                MAX_NUMBER_OF_MOVES_TO_FIND,
            );
        }
        if !available_next_states.is_empty() {
            // There are capture available, which means no simple moves are allowed
            // Just return and don't waste time finding those moves
            return available_next_states;
        }

        // no capture moves are available
        // so let's see what simple moves exist
        for (current_coor, available_directions) in owned_pieces_info.iter() {
            fill_vector_with_available_simple_move_states_for_piece(
                current_game_state,
                current_coor,
                available_directions,
                move_search_params.single_piece_value,
                move_search_params.double_piece_value,
                move_search_params.double_row,
                &mut available_next_states,
                MAX_NUMBER_OF_MOVES_TO_FIND,
            );
        }

        return available_next_states;
    }
}

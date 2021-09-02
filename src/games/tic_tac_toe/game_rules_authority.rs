use crate::games::tic_tac_toe::GameStateType as TicTacToeGameState;
use crate::traits::GameRulesAuthority as TGameRulesAuthority;

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

pub struct GameRulesAuthority {}

impl TGameRulesAuthority<TicTacToeGameState> for GameRulesAuthority {
    fn analyze_game_state_for_terminality(
        &self,
        game_state: &TicTacToeGameState,
        _next_player_index: i32,
    ) -> Option<i32> {
        for winning_position_set in WINNING_POSITION_SETS.iter() {
            let position_values = (
                game_state[winning_position_set.0 .0][winning_position_set.0 .1],
                game_state[winning_position_set.1 .0][winning_position_set.1 .1],
                game_state[winning_position_set.2 .0][winning_position_set.2 .1],
            );
            if position_values.0 > 0
                && position_values.0 == position_values.1
                && position_values.0 == position_values.2
            {
                return Some(position_values.0 as i32 - 1);
            }
        }

        let mut is_board_full = true;
        'rows: for i in 0..game_state.len() {
            for j in 0..game_state.len() {
                if game_state[i][j] == 0 {
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

    fn find_available_next_game_states(
        &self,
        current_player_index: i32,
        current_game_state: &TicTacToeGameState,
    ) -> Vec<TicTacToeGameState> {
        let mut available_next_game_states: Vec<TicTacToeGameState> = vec![];

        for i in 0..current_game_state.len() {
            for j in 0..current_game_state.len() {
                if current_game_state[i][j] == 0 {
                    // the space is empty
                    let mut available_next_game_state = current_game_state.clone();
                    available_next_game_state[i][j] = current_player_index as u8 + 1;
                    available_next_game_states.push(available_next_game_state);
                }
            }
        }

        return available_next_game_states;
    }
}

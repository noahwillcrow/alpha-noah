use crate::games::tic_tac_toe::GameStateType as TicTacToeGameState;
use crate::traits::AvailableNextGameStatesFinder as TAvailableNextGameStatesFinder;

pub struct AvailableNextGameStatesFinder {}

impl TAvailableNextGameStatesFinder<TicTacToeGameState> for AvailableNextGameStatesFinder {
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

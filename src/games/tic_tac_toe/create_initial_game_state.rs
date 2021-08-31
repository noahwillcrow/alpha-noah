use crate::games::tic_tac_toe::GameStateType as TicTacToeGameState;

pub fn create_initial_game_state() -> TicTacToeGameState {
    return vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]];
}

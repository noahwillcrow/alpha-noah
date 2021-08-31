use crate::games::tic_tac_toe::GameStateType as TicTacToeGameState;
use crate::traits::GameStateSerializer;

/// Each state hashes to 2 bytes - just encoding the base_3 sum of the elements from the 2D array working state
/// There are (3^9 - 1) total possible states according to a naive calculation (when only allowing for legal states, actually far fewer)
/// log_2(3^9 - 1) < 16, so 2 bytes is sufficient to represent all possible values
pub struct ByteArrayGameStateSerializer {}

impl GameStateSerializer<TicTacToeGameState, Vec<u8>> for ByteArrayGameStateSerializer {
    fn serialize_game_state(
        &self,
        _responsible_player_index: i32,
        game_state: &TicTacToeGameState,
    ) -> Vec<u8> {
        let mut state_raw_value: u16 = 0;
        let mut ternary_digit_multiplier: u16 = 1;
        for i in 0..game_state.len() {
            for j in 0..game_state.len() {
                let location_value = game_state[i][j] as u16;
                state_raw_value += location_value * ternary_digit_multiplier;
                ternary_digit_multiplier *= 3;
            }
        }

        return state_raw_value.to_be_bytes().to_vec();
    }
}

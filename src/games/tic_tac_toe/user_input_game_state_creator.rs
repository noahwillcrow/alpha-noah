use crate::games::tic_tac_toe::GameStateType as TicTacToeGameState;
use crate::traits::UserInputGameStateCreator as TUserInputGameStateCreator;

pub struct UserInputGameStateCreator {}

impl TUserInputGameStateCreator<TicTacToeGameState, String> for UserInputGameStateCreator {
    fn create_new_game_state_from_user_input(
        &mut self,
        current_player_index: i32,
        current_game_state: &TicTacToeGameState,
        user_input: String,
    ) -> Result<TicTacToeGameState, String> {
        let input_coor: (usize, usize);
        match &user_input[..3] {
            "0,0" => input_coor = (0, 0),
            "0,1" => input_coor = (0, 1),
            "0,2" => input_coor = (0, 2),
            "1,0" => input_coor = (1, 0),
            "1,1" => input_coor = (1, 1),
            "1,2" => input_coor = (1, 2),
            "2,0" => input_coor = (2, 0),
            "2,1" => input_coor = (2, 1),
            "2,2" => input_coor = (2, 2),
            _ => return Err(String::from("Invalid input format. Valid format is row,col using 0-based indices from the top-left.")),
        }

        let mut new_state = current_game_state.clone();
        if new_state[input_coor.0][input_coor.1] > 0 {
            return Err(String::from("That space is already taken!"));
        }

        new_state[input_coor.0][input_coor.1] = current_player_index as u8 + 1;
        return Ok(new_state);
    }
}

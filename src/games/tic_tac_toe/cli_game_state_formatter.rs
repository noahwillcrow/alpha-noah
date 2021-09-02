use crate::games::tic_tac_toe::GameStateType as TicTacToeGameState;
use crate::traits::CLIGameStateFormatter as TCLIGameStateFormatter;

pub struct CLIGameStateFormatter {}

impl TCLIGameStateFormatter<TicTacToeGameState> for CLIGameStateFormatter {
    fn format_game_state_for_cli(&self, game_state: &TicTacToeGameState) -> String {
        return format!(
            "    0|1|2\n\
            0 - {}|{}|{}\n\
            1 - {}|{}|{}\n\
            2 - {}|{}|{}\n",
            convert_space_value_to_cli_string(game_state[0][0]),
            convert_space_value_to_cli_string(game_state[0][1]),
            convert_space_value_to_cli_string(game_state[0][2]),
            convert_space_value_to_cli_string(game_state[1][0]),
            convert_space_value_to_cli_string(game_state[1][1]),
            convert_space_value_to_cli_string(game_state[1][2]),
            convert_space_value_to_cli_string(game_state[2][0]),
            convert_space_value_to_cli_string(game_state[2][1]),
            convert_space_value_to_cli_string(game_state[2][2])
        );
    }
}

fn convert_space_value_to_cli_string(space_value: u8) -> String {
    return String::from(match space_value {
        1 => "x",
        2 => "o",
        _ => " ",
    });
}

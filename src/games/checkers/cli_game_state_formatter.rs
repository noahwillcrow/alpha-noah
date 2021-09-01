use crate::games::checkers::internal::*;
use crate::games::checkers::GameStateType as CheckersGameState;
use crate::traits::CLIGameStateFormatter as TCLIGameStateFormatter;

pub struct CLIGameStateFormatter {}

impl TCLIGameStateFormatter<CheckersGameState> for CLIGameStateFormatter {
    fn format_game_state_for_cli(&self, game_state: &CheckersGameState) -> String {
        let mut string_pieces: Vec<String> = vec![];

        string_pieces.push(String::from(" |0|1|2|3|4|5|6|7|\n"));

        for row in 0..MAX_ROW + 1 {
            string_pieces.push(format!("{}", row));
            for col in 0..MAX_COL + 1 {
                if (row + col) % 2 == 0 {
                    // not a usable space
                    string_pieces.push(String::from("|■"));
                } else {
                    string_pieces.push(format!(
                        "|{}",
                        convert_space_value_to_cli_string(game_state[row as usize][col as usize])
                    ));
                }
            }
            string_pieces.push(String::from("|\n"));
        }

        return string_pieces.join("");
    }
}

fn convert_space_value_to_cli_string(space_value: u8) -> String {
    return String::from(match space_value {
        1 => "b",
        11 => "B",
        2 => "r",
        22 => "R",
        _ => " ",
    });
}

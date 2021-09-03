use crate::enums::DecideNextStateError;
use crate::traits::{BasicGameState, CLIGameStateFormatter, TurnTaker, UserInputGameStateCreator};
use std::io::Write;

pub struct CLIInputPlayerTurnTaker<'a, GameState: BasicGameState> {
    cli_game_state_formatter: &'a dyn CLIGameStateFormatter<GameState>,
    player_index: i32,
    user_input_game_state_creator: &'a dyn UserInputGameStateCreator<GameState, String>,
}

impl<'a, GameState: BasicGameState> CLIInputPlayerTurnTaker<'a, GameState> {
    pub fn new(
        cli_game_state_formatter: &'a dyn CLIGameStateFormatter<GameState>,
        player_index: i32,
        user_input_game_state_creator: &'a dyn UserInputGameStateCreator<GameState, String>,
    ) -> CLIInputPlayerTurnTaker<'a, GameState> {
        return CLIInputPlayerTurnTaker {
            cli_game_state_formatter: cli_game_state_formatter,
            player_index: player_index,
            user_input_game_state_creator: user_input_game_state_creator,
        };
    }
}

impl<'a, GameState: BasicGameState> TurnTaker<GameState>
    for CLIInputPlayerTurnTaker<'a, GameState>
{
    fn decide_next_game_state(
        &self,
        current_game_state: &GameState,
    ) -> Result<GameState, DecideNextStateError> {
        println!("Player {}, it's your turn!", self.player_index + 1);
        println!(
            "Current game state:\n{}",
            self.cli_game_state_formatter
                .format_game_state_for_cli(&current_game_state)
        );

        loop {
            print!("Please enter your desired move: ");
            std::io::stdout().flush().unwrap();
            let mut user_input_string = String::new();
            match std::io::stdin().read_line(&mut user_input_string) {
                Ok(_) => (),
                Err(_) => {
                    println!("Failed to read user input.");
                    continue;
                }
            }

            let create_new_game_state_result = self
                .user_input_game_state_creator
                .create_new_game_state_from_user_input(
                    self.player_index,
                    &current_game_state,
                    user_input_string,
                );
            match create_new_game_state_result {
                Ok(new_game_state) => {
                    println!("User input accepted.");
                    return Ok(new_game_state);
                }
                Err(error_message) => {
                    println!("{}", error_message);
                    continue;
                }
            }
        }
    }
}

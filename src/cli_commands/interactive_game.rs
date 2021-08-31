use crate::cli_commands::enums::game::Game;
use std::io;
use std::io::Write;

pub fn interactive_game(args: Vec<String>) -> Result<(), rusqlite::Error> {
    let mut game = Game::TicTacToe;
    let mut human_player_index = 0;
    let mut draws_weight = 5.0;
    let mut losses_weight = -10.0;
    let mut wins_weight = 10.0;

    {
        let mut arg_parser = argparse::ArgumentParser::new();

        arg_parser.refer(&mut game).required().add_option(
            &["-g", "--game"],
            argparse::Store,
            r#"Game to run (either "checkers" or "tic-tac-toe")"#,
        );

        arg_parser
            .refer(&mut human_player_index)
            .required()
            .add_option(
                &["-h", "--human-player-index"],
                argparse::Store,
                "Player index for the human player (0 or 1)",
            );

        arg_parser.refer(&mut draws_weight).add_option(
            &["-d", "--drawsweight"],
            argparse::Parse,
            "Weight of draws for state decisions",
        );

        arg_parser.refer(&mut losses_weight).add_option(
            &["-l", "--lossesweight"],
            argparse::Parse,
            "Weight of losses for state decisions",
        );

        arg_parser.refer(&mut wins_weight).add_option(
            &["-w", "--winsweight"],
            argparse::Parse,
            "Weight of wins for state decisions",
        );

        match arg_parser.parse(args, &mut std::io::stdout(), &mut std::io::stderr()) {
            Ok(()) => (),
            Err(x) => {
                println!("Failed to parse arguments, please try again");
                std::process::exit(x);
            }
        }
    }

    let record_weighting_function =
        crate::weight_calculators::record_weighting_functions::create_linear_weighted_closure(
            draws_weight,
            losses_weight,
            wins_weight,
        );

    match game {
        Game::TicTacToe => {
            let cal_max_capacity: usize = 10_000_000;
            let mut sqlite_state_record_dal =
                crate::persistence::sqlite_state_record_dal::SqliteStateRecordDAL::new(
                    String::from("tic-tac-toe"),
                    String::from("./GamesHistory.db"),
                )?;
            let mut cal_state_record_provider =
                crate::persistence::cal_state_record_provider::CALStateRecordProvider::new(
                    cal_max_capacity,
                    &mut sqlite_state_record_dal,
                );

            let mut current_player_index = -1;
            let mut current_state = crate::games::tic_tac_toe::create_initial_state();

            loop {
                current_player_index = (current_player_index + 1) % 2;

                if current_player_index == human_player_index {
                    'user_input_loop: loop {
                        println!("Current game board:");
                        print!(
                            "{}",
                            crate::games::tic_tac_toe::convert_state_to_cli_string(&current_state)
                        );
                        print!("Please enter desired move: ");
                        io::stdout().flush().unwrap();

                        let mut user_input_string = String::new();
                        match io::stdin().read_line(&mut user_input_string) {
                            Ok(_) => (),
                            Err(_) => {
                                println!(
                                    "Failed to read user input, please try again. User input: {}",
                                    user_input_string
                                );
                                continue 'user_input_loop;
                            }
                        }

                        match crate::games::tic_tac_toe::create_new_state_from_user_input(
                            current_player_index,
                            &current_state,
                            &user_input_string,
                        ) {
                            Ok(new_state) => {
                                println!("User input accepted.");
                                current_state = new_state;
                                break 'user_input_loop;
                            }
                            Err(_) => {
                                println!(
                                    "User input was invalid, please try again. User input: {}",
                                    user_input_string
                                );
                                continue 'user_input_loop;
                            }
                        }
                    }
                } else {
                    println!(
                        "Deciding CPU turn for player index {}.",
                        current_player_index
                    );

                    match crate::core::alpha_noah::decide_next_state(
                        current_player_index,
                        current_state,
                        &mut cal_state_record_provider,
                        crate::games::tic_tac_toe::hash_state,
                        crate::games::tic_tac_toe::fill_vector_with_available_states,
                        &record_weighting_function,
                        &crate::weight_calculators::visits_weighting_functions::difference_from_max,
                    ) {
                        Ok(new_state) => current_state = new_state,
                        Err(
                            crate::core::alpha_noah::DecideNextStateError::NoAvailableStatesError,
                        ) => {
                            panic!("How did we end up with no available states but not a terminal condition!?")
                        }
                    }
                }

                match crate::games::tic_tac_toe::get_terminal_state(
                    current_player_index,
                    &current_state,
                ) {
                    None => (),
                    Some(winning_player_index) => {
                        if winning_player_index == human_player_index {
                            println!("You won!");
                        } else if winning_player_index == -1 {
                            println!("The game ends in a draw.");
                        } else {
                            println!("You lost :(");
                        }

                        println!("Final game board:");
                        print!(
                            "{}",
                            crate::games::tic_tac_toe::convert_state_to_cli_string(&current_state)
                        );

                        return Ok(());
                    }
                }
            }
        }
        _ => panic!("The desired game is not enabled for interactive play"),
    }
}

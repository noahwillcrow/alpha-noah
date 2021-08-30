use argparse;
use std::str::FromStr;
use std::time::Instant;

mod core;
mod games;
mod weight_calculators;

enum Game {
    Checkers,
    TicTacToe,
}

impl FromStr for Game {
    type Err = ();
    fn from_str(src: &str) -> Result<Game, ()> {
        return match src {
            "checkers" => Ok(Game::Checkers),
            "tic-tac-toe" => Ok(Game::TicTacToe),
            _ => Err(()),
        };
    }
}

fn main() {
    let mut game = Game::TicTacToe;
    let mut should_report_simulation_duration = false;
    let mut number_of_games = 100;
    let mut max_number_of_turns = 1000;
    let mut is_reaching_max_number_of_turns_a_draw = true;
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
            .refer(&mut should_report_simulation_duration)
            .add_option(
                &["--reporttime"],
                argparse::StoreTrue,
                "Whether to measure duration of running simulations",
            );

        arg_parser.refer(&mut number_of_games).add_option(
            &["-n", "--numgames"],
            argparse::Parse,
            "Number of games to simulate",
        );

        arg_parser.refer(&mut max_number_of_turns).add_option(
            &["-m", "--maxturns"],
            argparse::Parse,
            "Maximum number of turns to simulate per game",
        );

        arg_parser
            .refer(&mut is_reaching_max_number_of_turns_a_draw)
            .add_option(
                &["--is-max-turns-a-draw"],
                argparse::Parse,
                "Determines whether reaching the max turns limit is a draw",
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

        match arg_parser.parse_args() {
            Ok(()) => (),
            Err(x) => {
                println!("Failed to parse arguments, please try again");
                std::process::exit(x);
            }
        }
    }

    let record_weighting_function =
        weight_calculators::record_weighting_functions::create_linear_weighted_closure(
            draws_weight,
            losses_weight,
            wins_weight,
        );

    let mut win_counts_by_player_index = vec![0; 2];
    let mut update_win_counts = |winning_player_index: i32| {
        if winning_player_index >= 0 {
            win_counts_by_player_index[winning_player_index as usize] += 1;
        }
    };

    let start_instant = Instant::now();

    match game {
        Game::Checkers => {
            let mut state_record_provider =
                core::state_record_provider::InMemoryStateByteStringHashRecordProvider::new();

            for _ in 0..number_of_games {
                update_win_counts(core::alpha_noah::execute_standard_turn_based_game(
                    games::checkers::create_initial_state(),
                    2,
                    &mut state_record_provider,
                    games::checkers::hash_state,
                    games::checkers::fill_vector_with_available_states,
                    &record_weighting_function,
                    &weight_calculators::visits_weighting_functions::difference_from_max,
                    games::checkers::get_terminal_state,
                    max_number_of_turns,
                    is_reaching_max_number_of_turns_a_draw,
                ));
            }
        }
        Game::TicTacToe => {
            let mut state_record_provider =
                core::state_record_provider::InMemoryStateByteStringHashRecordProvider::new();

            for _ in 0..number_of_games {
                update_win_counts(core::alpha_noah::execute_standard_turn_based_game(
                    games::tic_tac_toe::create_initial_state(),
                    2,
                    &mut state_record_provider,
                    games::tic_tac_toe::hash_state,
                    games::tic_tac_toe::fill_vector_with_available_states,
                    &record_weighting_function,
                    &weight_calculators::visits_weighting_functions::difference_from_max,
                    games::tic_tac_toe::get_terminal_state,
                    max_number_of_turns,
                    is_reaching_max_number_of_turns_a_draw,
                ));
            }
        }
    }

    let simulation_duration = start_instant.elapsed();

    println!(
        "Final results are in! The x player won {} games and the o player won {} games.",
        win_counts_by_player_index[0], win_counts_by_player_index[1]
    );

    if should_report_simulation_duration {
        println!(
            "Simulation of {} games took {:?}.",
            number_of_games, simulation_duration
        );
    }
}

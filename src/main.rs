use argparse;
use std::time::Instant;

mod core;
mod games;
mod weight_calculators;

fn main() {
    let mut should_report_simulation_duration = false;
    let mut number_of_games = 100;
    let mut draws_weight = 5.0;
    let mut losses_weight = -10.0;
    let mut wins_weight = 10.0;

    {
        let mut arg_parser = argparse::ArgumentParser::new();
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

    let mut state_record_provider = games::tic_tac_toe::create_state_record_provider();

    let record_weighting_function =
        weight_calculators::record_weighting_functions::create_linear_weighted_closure(
            draws_weight,
            losses_weight,
            wins_weight,
        );

    let mut win_counts_by_player_index = vec![0; 2];

    let start_instant = Instant::now();
    for _ in 1..number_of_games {
        let winning_player_index = core::alpha_noah::execute_standard_turn_based_game(
            games::tic_tac_toe::create_initial_state(),
            2,
            &mut state_record_provider,
            games::tic_tac_toe::hash_state,
            games::tic_tac_toe::fill_vector_with_available_states,
            &record_weighting_function,
            &weight_calculators::visits_weighting_functions::difference_from_max,
            games::tic_tac_toe::get_terminal_state,
        );

        if winning_player_index >= 0 {
            win_counts_by_player_index[winning_player_index as usize] += 1;
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

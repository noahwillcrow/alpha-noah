use crate::cli::enums::Game;
use crate::composites::GameReportsIterativeProcessor;
use crate::game_runners::StandardTurnBasedGameRunner;
use crate::game_state_records_providers::LruCacheFrontedGameStateRecordsProvider;
use crate::games;
use crate::persistence::{SqliteByteArrayLogGameReportsProcessor, SqliteGameStateRecordsDAL};
use crate::training::StandardTrainer;
use crate::traits::GameReportsProcessor;
use crate::turn_takers::{BestWeightSelectionTurnTaker, WeightedRandomSelectionTurnTaker};
use crate::weights_calculators::RecordValuesWeightedSumGameStateWeightsCalculator;
use std::cell::RefCell;
use std::rc::Rc;

pub fn simulate_games<'a>(args: Vec<String>) -> Result<(), ()> {
    let mut game = Game::TicTacToe;
    let mut number_of_games = 100;
    let mut max_number_of_turns = 1000;
    let mut is_reaching_max_number_of_turns_a_draw = true;
    let mut draws_weight = 5.0;
    let mut losses_weight = -10.0;
    let mut wins_weight = 10.0;
    let mut visits_deficit_weight = 20.0;

    {
        let mut arg_parser = argparse::ArgumentParser::new();

        arg_parser.refer(&mut game).required().add_option(
            &["-g", "--game"],
            argparse::Store,
            r#"Game to run (either "checkers" or "tic-tac-toe")"#,
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
            &["--draws-weight"],
            argparse::Parse,
            "Weight of draws for state decisions",
        );

        arg_parser.refer(&mut losses_weight).add_option(
            &["--losses-weight"],
            argparse::Parse,
            "Weight of losses for state decisions",
        );

        arg_parser.refer(&mut wins_weight).add_option(
            &["--wins-weight"],
            argparse::Parse,
            "Weight of wins for state decisions",
        );

        arg_parser.refer(&mut visits_deficit_weight).add_option(
            &["--visits-deficit-weight"],
            argparse::Parse,
            "Weight of visits deficit for state decisions",
        );

        match arg_parser.parse(args, &mut std::io::stdout(), &mut std::io::stderr()) {
            Ok(()) => (),
            Err(x) => {
                println!("Failed to parse arguments, please try again");
                std::process::exit(x);
            }
        }
    }

    let sqlite_db_path = "./GamesHistory.db";

    match game {
        Game::Checkers => {
            let game_name = "checkers";
            let logs_serializer_version = 1;

            let lru_cache_max_capacity: usize = 1_000_000;
            let game_state_records_dal = SqliteGameStateRecordsDAL::new(game_name, sqlite_db_path)
                .expect("Failed to create SqliteGameStateRecordsDAL.");
            let game_state_records_dal_rc = Rc::new(RefCell::new(game_state_records_dal));
            let game_state_records_provider = LruCacheFrontedGameStateRecordsProvider::new(
                lru_cache_max_capacity,
                Rc::clone(&game_state_records_dal_rc),
            );

            let sqlite_game_reports_processor = SqliteByteArrayLogGameReportsProcessor::new(
                game_name,
                logs_serializer_version,
                10_000,
                sqlite_db_path,
            );
            let game_reports_processors_vector: Vec<&dyn GameReportsProcessor<Vec<u8>, ()>> =
                vec![&game_state_records_provider, &sqlite_game_reports_processor];
            let game_reports_processor =
                GameReportsIterativeProcessor::new(game_reports_processors_vector);

            let game_rules_authority = games::checkers::GameRulesAuthority {};
            let game_state_serializer = games::checkers::ByteArrayGameStateSerializer {};

            let mut base_game_runner =
                StandardTurnBasedGameRunner::new(&game_rules_authority, &game_state_serializer);
            let mut trainer = StandardTrainer::new(
                &mut base_game_runner,
                game_name,
                &game_reports_processor,
                true,
                vec![&game_state_records_provider],
            );

            let game_state_weights_calculator =
                RecordValuesWeightedSumGameStateWeightsCalculator::new(
                    &game_state_records_provider,
                    &game_state_serializer,
                    draws_weight,
                    losses_weight,
                    wins_weight,
                    visits_deficit_weight,
                );

            let mut first_player_turn_taker = WeightedRandomSelectionTurnTaker::new(
                &game_rules_authority,
                &game_state_weights_calculator,
                0,
            );

            let mut second_player_turn_taker = WeightedRandomSelectionTurnTaker::new(
                &game_rules_authority,
                &game_state_weights_calculator,
                1,
            );

            trainer
                .train(
                    number_of_games,
                    games::checkers::create_initial_game_state,
                    &mut vec![&mut first_player_turn_taker, &mut second_player_turn_taker],
                    max_number_of_turns,
                    is_reaching_max_number_of_turns_a_draw,
                )
                .expect("Training failed.");
        }
        Game::TicTacToe => {
            let game_name = "tic-tac-toe";
            let logs_serializer_version = 1;

            let lru_cache_max_capacity: usize = 1_000_000;
            let game_state_records_dal = SqliteGameStateRecordsDAL::new(game_name, sqlite_db_path)
                .expect("Failed to create SqliteGameStateRecordsDAL.");
            let game_state_records_dal_rc = Rc::new(RefCell::new(game_state_records_dal));
            let game_state_records_provider = LruCacheFrontedGameStateRecordsProvider::new(
                lru_cache_max_capacity,
                Rc::clone(&game_state_records_dal_rc),
            );

            let sqlite_game_reports_processor = SqliteByteArrayLogGameReportsProcessor::new(
                game_name,
                logs_serializer_version,
                10_000,
                sqlite_db_path,
            );
            let game_reports_processors_vector: Vec<&dyn GameReportsProcessor<Vec<u8>, ()>> =
                vec![&game_state_records_provider, &sqlite_game_reports_processor];
            let game_reports_processor =
                GameReportsIterativeProcessor::new(game_reports_processors_vector);

            let game_rules_authority = games::tic_tac_toe::GameRulesAuthority {};
            let game_state_serializer = games::tic_tac_toe::ByteArrayGameStateSerializer {};

            let mut base_game_runner =
                StandardTurnBasedGameRunner::new(&game_rules_authority, &game_state_serializer);
            let mut trainer = StandardTrainer::new(
                &mut base_game_runner,
                game_name,
                &game_reports_processor,
                true,
                vec![&game_state_records_provider],
            );

            let game_state_weights_calculator =
                RecordValuesWeightedSumGameStateWeightsCalculator::new(
                    &game_state_records_provider,
                    &game_state_serializer,
                    draws_weight,
                    losses_weight,
                    wins_weight,
                    visits_deficit_weight,
                );

            let mut first_player_turn_taker = WeightedRandomSelectionTurnTaker::new(
                &game_rules_authority,
                &game_state_weights_calculator,
                0,
            );

            let mut second_player_turn_taker = BestWeightSelectionTurnTaker::new(
                &game_rules_authority,
                &game_state_weights_calculator,
                1,
            );

            trainer
                .train(
                    number_of_games,
                    games::tic_tac_toe::create_initial_game_state,
                    &mut vec![&mut first_player_turn_taker, &mut second_player_turn_taker],
                    max_number_of_turns,
                    is_reaching_max_number_of_turns_a_draw,
                )
                .expect("Training failed.");
        }
    }

    return Ok(());
}

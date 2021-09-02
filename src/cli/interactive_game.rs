use crate::cli::enums::Game;
use crate::game_runners::StandardTurnBasedGameRunner;
use crate::game_state_records_providers::LruCacheFrontedGameStateRecordsProvider;
use crate::games;
use crate::persistence::{SqliteByteArrayLogGameReportsPersister, SqliteGameStateRecordsDAL};
use crate::training::StandardTrainer;
use crate::traits::TurnTaker;
use crate::turn_takers::{CLIInputPlayerTurnTaker, WeightedGameStatesMonteCarloTurnTaker};
use crate::weights_calculators::RecordValuesWeightedSumGameStateWeightsCalculator;
use std::cell::RefCell;
use std::rc::Rc;

pub fn interactive_game(args: Vec<String>) -> Result<(), ()> {
    let mut game = Game::TicTacToe;
    let mut cli_input_player_index = 0;
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

        arg_parser
            .refer(&mut cli_input_player_index)
            .required()
            .add_option(
                &["-h", "--human-player-index"],
                argparse::Store,
                "Player index for the human player (0 or 1)",
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

            let lru_cache_max_capacity: usize = 100_000;
            let game_state_records_dal = SqliteGameStateRecordsDAL::new(game_name, sqlite_db_path)
                .expect("Failed to create SqliteGameStateRecordsDAL.");
            let game_state_records_dal_rc = Rc::new(RefCell::new(game_state_records_dal));
            let game_state_records_provider = LruCacheFrontedGameStateRecordsProvider::new(
                lru_cache_max_capacity,
                Rc::clone(&game_state_records_dal_rc),
            );
            let game_state_records_provider_ref_cell = RefCell::new(game_state_records_provider);

            let game_reports_persister = SqliteByteArrayLogGameReportsPersister::new(
                game_name,
                logs_serializer_version,
                1,
                sqlite_db_path,
            );
            let game_reports_persister_ref_cell = RefCell::new(game_reports_persister);

            let game_state_serializer = games::checkers::ByteArrayGameStateSerializer {};
            let terminal_game_state_analyzer = games::checkers::TerminalGameStateAnalyzer {};

            let mut base_game_runner = StandardTurnBasedGameRunner::new(
                &game_state_serializer,
                &terminal_game_state_analyzer,
            );
            let mut trainer = StandardTrainer::new(
                &mut base_game_runner,
                game_name,
                &game_reports_persister_ref_cell,
                &game_state_records_provider_ref_cell,
                false,
                vec![
                    &game_reports_persister_ref_cell,
                    &game_state_records_provider_ref_cell,
                ],
            );

            let available_next_game_states_finder =
                games::checkers::AvailableNextGameStatesFinder {};

            let game_state_weights_calculator =
                RecordValuesWeightedSumGameStateWeightsCalculator::new(
                    &game_state_records_provider_ref_cell,
                    &game_state_serializer,
                    draws_weight,
                    losses_weight,
                    wins_weight,
                    visits_deficit_weight,
                );

            let cpu_player_index = (cli_input_player_index + 1) % 2;
            let mut cpu_player_turn_taker = WeightedGameStatesMonteCarloTurnTaker::new(
                &available_next_game_states_finder,
                &game_state_weights_calculator,
                cpu_player_index,
            );

            let cli_game_state_formatter = games::checkers::CLIGameStateFormatter {};
            let mut user_input_game_state_creator =
                games::checkers::UserInputGameStateCreator::new();

            let mut cli_input_player_turn_taker = CLIInputPlayerTurnTaker::new(
                &cli_game_state_formatter,
                cli_input_player_index,
                &mut user_input_game_state_creator,
            );

            let mut turn_takers: Vec<&mut dyn TurnTaker<games::checkers::GameStateType>> =
                vec![&mut cpu_player_turn_taker];
            turn_takers.insert(
                cli_input_player_index as usize,
                &mut cli_input_player_turn_taker,
            );

            trainer
                .train(
                    1,
                    games::checkers::create_initial_game_state,
                    &mut turn_takers,
                    -1,
                    true,
                )
                .expect("Training failed.");
        }
        Game::TicTacToe => {
            let game_name = "tic-tac-toe";
            let logs_serializer_version = 1;

            let lru_cache_max_capacity: usize = 100_000;
            let game_state_records_dal = SqliteGameStateRecordsDAL::new(game_name, sqlite_db_path)
                .expect("Failed to create SqliteGameStateRecordsDAL.");
            let game_state_records_dal_rc = Rc::new(RefCell::new(game_state_records_dal));
            let game_state_records_provider = LruCacheFrontedGameStateRecordsProvider::new(
                lru_cache_max_capacity,
                Rc::clone(&game_state_records_dal_rc),
            );
            let game_state_records_provider_ref_cell = RefCell::new(game_state_records_provider);

            let game_reports_persister = SqliteByteArrayLogGameReportsPersister::new(
                game_name,
                logs_serializer_version,
                1,
                sqlite_db_path,
            );
            let game_reports_persister_ref_cell = RefCell::new(game_reports_persister);

            let game_state_serializer = games::tic_tac_toe::ByteArrayGameStateSerializer {};
            let terminal_game_state_analyzer = games::tic_tac_toe::TerminalGameStateAnalyzer {};

            let mut base_game_runner = StandardTurnBasedGameRunner::new(
                &game_state_serializer,
                &terminal_game_state_analyzer,
            );
            let mut trainer = StandardTrainer::new(
                &mut base_game_runner,
                game_name,
                &game_reports_persister_ref_cell,
                &game_state_records_provider_ref_cell,
                false,
                vec![
                    &game_reports_persister_ref_cell,
                    &game_state_records_provider_ref_cell,
                ],
            );

            let available_next_game_states_finder =
                games::tic_tac_toe::AvailableNextGameStatesFinder {};

            let game_state_weights_calculator =
                RecordValuesWeightedSumGameStateWeightsCalculator::new(
                    &game_state_records_provider_ref_cell,
                    &game_state_serializer,
                    draws_weight,
                    losses_weight,
                    wins_weight,
                    visits_deficit_weight,
                );

            let cpu_player_index = (cli_input_player_index + 1) % 2;
            let mut cpu_player_turn_taker = WeightedGameStatesMonteCarloTurnTaker::new(
                &available_next_game_states_finder,
                &game_state_weights_calculator,
                cpu_player_index,
            );

            let cli_game_state_formatter = games::tic_tac_toe::CLIGameStateFormatter {};
            let mut user_input_game_state_creator =
                games::tic_tac_toe::UserInputGameStateCreator {};

            let mut cli_input_player_turn_taker = CLIInputPlayerTurnTaker::new(
                &cli_game_state_formatter,
                cli_input_player_index,
                &mut user_input_game_state_creator,
            );

            let mut turn_takers: Vec<&mut dyn TurnTaker<games::tic_tac_toe::GameStateType>> =
                vec![&mut cpu_player_turn_taker];
            turn_takers.insert(
                cli_input_player_index as usize,
                &mut cli_input_player_turn_taker,
            );

            trainer
                .train(
                    1,
                    games::tic_tac_toe::create_initial_game_state,
                    &mut turn_takers,
                    -1,
                    true,
                )
                .expect("Training failed.");
        }
    }

    return Ok(());
}

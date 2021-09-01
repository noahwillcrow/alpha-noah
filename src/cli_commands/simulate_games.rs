use crate::cli_commands::enums::Game;
use crate::game_runners::StandardTurnBasedGameRunner;
use crate::game_state_records_providers::LruCacheFrontedGameStateRecordsProvider;
use crate::games;
use crate::persistence::SqliteGameStateRecordsDAL;
use crate::training::SqliteByteArraySerializedGameStatesTrainer;
use crate::turn_takers::GameStateRecordWeightedMonteCarloTurnTaker;
use crate::weights_calculators::WeightedSumGameStateRecordWeightsCalculator;
use std::cell::RefCell;
use std::rc::Rc;

pub fn simulate_games(args: Vec<String>) -> Result<(), rusqlite::Error> {
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
            &["--vists-deficit-weight"],
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

    let game_state_record_weights_calculator = WeightedSumGameStateRecordWeightsCalculator {
        draws_weight: draws_weight,
        losses_weight: losses_weight,
        wins_weight: wins_weight,
        visits_deficit_weight: visits_deficit_weight,
    };
    let sqlite_db_path = "./GamesHistory.db";

    match game {
        Game::Checkers => {
            // let game_name = "checkers";
            // let logs_serializer_version = 1;

            // let lru_cache_max_capacity: usize = 10_000_000;
            // let mut sqlite_game_state_records_dal =
            //     SqliteGameStateRecordsDAL::new(game_name, sqlite_db_path)?;
            // let mut lru_cache_fronted_game_state_records_provider =
            //     LruCacheFrontedGameStateRecordsProvider::new(
            //         lru_cache_max_capacity,
            //         &mut sqlite_game_state_records_dal,
            //     );

            // let base_game_runner = StandardTurnBasedGameRunner::new(game_state_serializer, terminal_game_state_analyzer);
            // let trainer = SqliteByteArraySerializedGameStatesTrainer::new(
            //     base_game_runner,
            //     game_name,
            //     &mut lru_cache_fronted_game_state_records_provider,
            //     logs_serializer_version,
            //     sqlite_db_path,
            //     vec![&mut lru_cache_fronted_game_state_records_provider],
            // );
        }
        Game::TicTacToe => {
            let game_name = "tic-tac-toe";
            let logs_serializer_version = 1;

            let lru_cache_max_capacity: usize = 10_000_000;
            let game_state_records_dal = SqliteGameStateRecordsDAL::new(game_name, sqlite_db_path)?;
            let game_state_records_dal_rc = Rc::new(RefCell::new(game_state_records_dal));
            let game_state_records_provider = LruCacheFrontedGameStateRecordsProvider::new(
                lru_cache_max_capacity,
                Rc::clone(&game_state_records_dal_rc),
            );
            let game_state_records_provider_ref_cell = RefCell::new(game_state_records_provider);

            let game_state_serializer = games::tic_tac_toe::ByteArrayGameStateSerializer {};
            let terminal_game_state_analyzer = games::tic_tac_toe::TerminalGameStateAnalyzer {};

            let mut base_game_runner = StandardTurnBasedGameRunner::new(
                &game_state_serializer,
                &terminal_game_state_analyzer,
            );
            let mut trainer = SqliteByteArraySerializedGameStatesTrainer::new(
                &mut base_game_runner,
                game_name,
                logs_serializer_version,
                &game_state_records_provider_ref_cell,
                sqlite_db_path,
                &game_state_records_provider_ref_cell,
            );

            let available_next_game_states_finder =
                games::tic_tac_toe::AvailableNextGameStatesFinder {};

            let mut first_player_turn_taker = GameStateRecordWeightedMonteCarloTurnTaker::new(
                &available_next_game_states_finder,
                &game_state_records_provider_ref_cell,
                &game_state_record_weights_calculator,
                &game_state_serializer,
                0,
            );

            // let humans in!
            // let fmtr = games::tic_tac_toe::CLIGameStateFormatter {};
            // let mut ui = games::tic_tac_toe::UserInputGameStateCreator {};
            // let mut second_player_turn_taker =
            //     crate::turn_takers::CLIHumanPlayerTurnTaker::new(&fmtr, 1, &mut ui);
            let mut second_player_turn_taker = GameStateRecordWeightedMonteCarloTurnTaker::new(
                &available_next_game_states_finder,
                &game_state_records_provider_ref_cell,
                &game_state_record_weights_calculator,
                &game_state_serializer,
                1,
            );

            trainer.train(
                number_of_games,
                games::tic_tac_toe::create_initial_game_state,
                &mut vec![&mut first_player_turn_taker, &mut second_player_turn_taker],
                max_number_of_turns,
                is_reaching_max_number_of_turns_a_draw,
            )?;
        }
    }

    return Ok(());
}

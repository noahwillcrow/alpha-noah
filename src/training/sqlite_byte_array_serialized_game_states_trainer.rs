use crate::structs::GameReport;
use crate::training::update_game_state_records;
use crate::traits::{
    BasicGameState, GameRunner, GameStateRecordsProvider, PendingUpdatesManager, TurnTaker,
};
use rusqlite::Connection;
use std::cell::RefCell;
use std::time::Instant;

pub struct SqliteByteArraySerializedGameStatesTrainer<'a, GameState: BasicGameState> {
    base_game_runner: &'a mut dyn GameRunner<GameState, Vec<u8>>,
    game_name: String,
    logs_serializer_version: i32,
    game_state_records_provider_ref_cell: &'a RefCell<dyn GameStateRecordsProvider<Vec<u8>>>,
    sqlite_db_path: String,
    pending_updates_manager_ref_cell: &'a RefCell<dyn PendingUpdatesManager>,
}

impl<'a, GameState: BasicGameState> SqliteByteArraySerializedGameStatesTrainer<'a, GameState> {
    pub fn new(
        base_game_runner: &'a mut dyn GameRunner<GameState, Vec<u8>>,
        game_name: &str,
        logs_serializer_version: i32,
        game_state_records_provider_ref_cell: &'a RefCell<dyn GameStateRecordsProvider<Vec<u8>>>,
        sqlite_db_path: &str,
        pending_updates_manager_ref_cell: &'a RefCell<dyn PendingUpdatesManager>,
    ) -> SqliteByteArraySerializedGameStatesTrainer<'a, GameState> {
        return SqliteByteArraySerializedGameStatesTrainer {
            base_game_runner: base_game_runner,
            game_name: String::from(game_name),
            game_state_records_provider_ref_cell: game_state_records_provider_ref_cell,
            logs_serializer_version: logs_serializer_version,
            sqlite_db_path: String::from(sqlite_db_path),
            pending_updates_manager_ref_cell: pending_updates_manager_ref_cell,
        };
    }

    pub fn train(
        &mut self,
        number_of_games: u32,
        create_initial_game_state: fn() -> GameState,
        turn_takers: &mut Vec<&mut dyn TurnTaker<GameState>>,
        max_number_of_turns: i32,
        is_reaching_max_number_of_turns_a_draw: bool,
    ) -> Result<(), rusqlite::Error> {
        let mut game_reports: Vec<GameReport<Vec<u8>>> = vec![];

        println!(
            "Starting simulation of {} games of {}.",
            number_of_games, &self.game_name
        );

        let mut win_counts_by_player_index = vec![0; 2];
        let mut update_win_counts = |winning_player_index: i32| {
            if winning_player_index >= 0 {
                win_counts_by_player_index[winning_player_index as usize] += 1;
            }
        };

        let simulations_start_instant = Instant::now();
        for _ in 0..number_of_games {
            let run_game_result = self.base_game_runner.run_game(
                create_initial_game_state(),
                turn_takers,
                max_number_of_turns,
                is_reaching_max_number_of_turns_a_draw,
            );

            match run_game_result {
                Ok(game_report_option) => match game_report_option {
                    Some(game_report) => {
                        update_win_counts(game_report.winning_player_index);
                        update_game_state_records(
                            self.game_state_records_provider_ref_cell,
                            game_report.clone(),
                        );
                        game_reports.push(game_report);
                    }
                    None => (),
                },
                Err(_) => (),
            }
        }

        println!(
            "Simulations complete. Duration: {:?}.",
            simulations_start_instant.elapsed()
        );

        println!(
            "Number of games won by player index: {:#?}.",
            win_counts_by_player_index
        );

        println!("Waiting for all pending updates to be committed.");
        let pending_updates_start_instant = Instant::now();

        self.pending_updates_manager_ref_cell
            .borrow_mut()
            .try_commit_pending_updates_in_background(usize::MAX)
            .join()
            .expect("Failed to commit pending updates");

        println!(
            "Pending updates commited. Duration: {:?}.",
            pending_updates_start_instant.elapsed()
        );

        println!("Saving game logs.");
        let save_game_logs_start_instant = Instant::now();

        let mut sqlite_connection = Connection::open(&self.sqlite_db_path).unwrap();
        let sqlite_transaction: rusqlite::Transaction = sqlite_connection.transaction()?;

        loop {
            match game_reports.pop() {
                Some(game_report) => {
                    let mut log_entry: Vec<u8> = vec![];
                    for game_state_update in &game_report.game_state_updates {
                        log_entry.append(&mut game_state_update.new_serialized_game_state.clone());
                    }
                    sqlite_transaction
                        .execute(
                            "INSERT INTO GameLogs (GameName, Log, LogSerializerVersion, WinningPlayerIndex) VALUES (?1, ?2, ?3, ?4)",
                            rusqlite::params![self.game_name, log_entry, self.logs_serializer_version, game_report.winning_player_index],
                        )?;
                }
                None => {
                    break;
                }
            }
        }

        sqlite_transaction.commit()?;

        println!(
            "Saving game logs completed. Duration: {:?}",
            save_game_logs_start_instant.elapsed()
        );

        println!("Done training.");
        return Ok(());
    }
}

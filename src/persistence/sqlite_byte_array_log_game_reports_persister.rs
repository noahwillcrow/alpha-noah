use crate::structs::GameReport;
use crate::traits::{GameReportsPersister, PendingUpdatesManager};
use rusqlite::{Connection, Transaction};
use std::thread;

const MAX_ATTEMPTS_PER_GAME_REPORT: u8 = 3;

pub struct SqliteByteArrayLogGameReportsPersister {
    game_name: String,
    log_serializer_version: i32,
    max_batch_size: usize,
    pending_game_reports: Vec<GameReport<Vec<u8>>>,
    sqlite_db_path: String,
}

impl SqliteByteArrayLogGameReportsPersister {
    pub fn new(
        game_name: &str,
        log_serializer_version: i32,
        max_batch_size: usize,
        sqlite_db_path: &str,
    ) -> SqliteByteArrayLogGameReportsPersister {
        return SqliteByteArrayLogGameReportsPersister {
            game_name: String::from(game_name),
            log_serializer_version: log_serializer_version,
            max_batch_size: max_batch_size,
            pending_game_reports: vec![],
            sqlite_db_path: String::from(sqlite_db_path),
        };
    }
}

impl GameReportsPersister<Vec<u8>, ()> for SqliteByteArrayLogGameReportsPersister {
    fn persist_game_report(&mut self, game_report: GameReport<Vec<u8>>) -> Result<(), ()> {
        self.pending_game_reports.push(game_report);
        if self.pending_game_reports.len() >= self.max_batch_size {
            self.try_commit_pending_updates_in_background(self.max_batch_size as usize);
        }

        return Ok(());
    }
}

impl PendingUpdatesManager for SqliteByteArrayLogGameReportsPersister {
    fn try_commit_pending_updates_in_background(
        &mut self,
        max_number_to_commit: usize,
    ) -> std::thread::JoinHandle<()> {
        let mut game_reports_to_commit: Vec<GameReport<Vec<u8>>> = vec![];
        loop {
            match self.pending_game_reports.pop() {
                Some(game_report) => {
                    game_reports_to_commit.push(game_report);

                    if game_reports_to_commit.len() == max_number_to_commit {
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }

        let game_name = self.game_name.clone();
        let log_serializer_version = self.log_serializer_version;
        let sqlite_db_path = self.sqlite_db_path.clone();

        return thread::spawn(move || {
            let mut sqlite_connection: Connection;
            match Connection::open(&sqlite_db_path) {
                Ok(value) => {
                    sqlite_connection = value;
                }
                Err(err) => {
                    println!("Failed to open write connection for persisting updates. Updates will be dropped. Error: {}", err);
                    return;
                }
            }

            let sqlite_transaction: Transaction;
            match sqlite_connection.transaction() {
                Ok(value) => {
                    sqlite_transaction = value;
                }
                Err(err) => {
                    println!("Failed to start transaction for persisting updates. Updates will be dropped. Error: {}", err);
                    return;
                }
            }

            for game_report in game_reports_to_commit.iter() {
                let mut attempts_counter: u8 = 0;

                let mut log_entry: Vec<u8> = vec![];
                for game_state_update in &game_report.game_state_updates {
                    log_entry.append(&mut game_state_update.new_serialized_game_state.clone());
                }

                'attempts_loop: while attempts_counter < MAX_ATTEMPTS_PER_GAME_REPORT {
                    let execute_result = sqlite_transaction
                        .execute(
                            "INSERT INTO GameLogs (GameName, Log, LogSerializerVersion, WinningPlayerIndex) VALUES (?1, ?2, ?3, ?4)",
                            rusqlite::params![game_name, log_entry, log_serializer_version, game_report.winning_player_index],
                        );

                    match execute_result {
                        Ok(_) => break 'attempts_loop,
                        Err(_) => attempts_counter += 1,
                    };
                }
            }

            match sqlite_transaction.commit() {
                Ok(_) => (),
                Err(err) => {
                    println!("Failed to commit transaction for persisting updates. Updates will be dropped. Error: {}", err);
                    return;
                }
            }
        });
    }
}

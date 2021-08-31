use crate::structs::GameReport;
use crate::traits::{GameReportsPersister, PendingUpdatesManager};
use rusqlite::Connection;
use std::thread;

const MAX_ATTEMPTS_PER_GAME_REPORT: u8 = 3;

pub struct SqliteByteArrayLogGameReportsPersister<'a> {
    game_name: String,
    log_serializer_version: i32,
    pending_game_reports: &'a mut Vec<GameReport<Vec<u8>>>,
    sqlite_db_path: String,
}

impl<'a> SqliteByteArrayLogGameReportsPersister<'a> {
    pub fn new(
        game_name: &str,
        log_serializer_version: i32,
        sqlite_db_path: &str,
    ) -> SqliteByteArrayLogGameReportsPersister<'a> {
        return SqliteByteArrayLogGameReportsPersister {
            game_name: String::from(game_name),
            log_serializer_version: log_serializer_version,
            pending_game_reports: vec![],
            sqlite_db_path: String::from(sqlite_db_path),
        };
    }
}

impl<'a> PendingUpdatesManager for SqliteByteArrayLogGameReportsPersister<'a> {
    fn try_commit_pending_updates_in_background(
        &mut self,
        max_number_to_commit: usize,
    ) -> std::thread::JoinHandle<()> {
        let mut game_reports_to_commit: Vec<GameReport<Vec<u8>>> = vec![];
        'grab_tasks_loop: loop {
            match self.pending_game_reports.pop() {
                Some(game_report) => {
                    game_reports_to_commit.push(game_report);
                }
                None => {
                    break 'grab_tasks_loop;
                }
            }
        }

        let sqlite_db_path = self.sqlite_db_path.clone();
        let game_name = self.game_name.clone();

        return thread::spawn(move || {
            let sqlite_connection: Connection;

            match Connection::open(&sqlite_db_path) {
                Ok(opened_connection) => {
                    sqlite_connection = opened_connection;
                }
                Err(err) => {
                    println!("Failed to open write connection for persisting updates. Updates will be dropped. Error: {}", err);
                    return;
                }
            }

            for game_report in game_reports_to_commit.iter() {
                let mut attempts_counter: u8 = 0;

                let game_log_blob: Vec<u8> = vec![];
                for game_state_update in game_report.game

                'attempts_loop: while attempts_counter < MAX_ATTEMPTS_PER_GAME_REPORT {
                    let increment_result =
                        write_game_log_to_db(&sqlite_connection, &game_name, &);

                    match increment_result {
                        Ok(_) => break 'attempts_loop,
                        Err(_) => attempts_counter += 1,
                    };
                }
            }
        });
    }
}

fn write_game_log_to_db(
    sqlite_connection: &Connection,
    game_name: &str,
    game_log_blob: &Vec<u8>,
    log_serializer_version: i32,
    winning_player_index: i32,
) -> Result<(), rusqlite::Error> {
    sqlite_connection.execute(
        "INSERT INTO GameLogs (GameName, Log, LogSerializerVersion, WinningPlayerIndex) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![game_name, game_log_blob, log_serializer_version, winning_player_index],
    )?;

    return Ok(());
}

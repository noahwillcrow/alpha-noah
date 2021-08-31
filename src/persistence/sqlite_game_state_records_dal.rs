use crate::structs::{GameStateRecord, IncrementPersistedGameStateRecordValuesTask};
use crate::traits::{BasicSerializedGameState, GameStateRecordsDAL};
use rusqlite::Connection;
use rusqlite::Error::QueryReturnedNoRows;
use std::thread;

const MAX_ATTEMPTS_PER_UPDATE: u8 = 3;

pub struct SqliteGameStateRecordsDAL {
    game_name: String,
    read_only_connection: Connection,
    sqlite_db_path: String,
}

impl SqliteGameStateRecordsDAL {
    pub fn new(
        game_name: &str,
        sqlite_db_path: &str,
    ) -> Result<SqliteGameStateRecordsDAL, rusqlite::Error> {
        let read_only_connection = Connection::open(&sqlite_db_path)?;

        return Ok(SqliteGameStateRecordsDAL {
            game_name: String::from(game_name),
            read_only_connection: read_only_connection,
            sqlite_db_path: String::from(sqlite_db_path),
        });
    }
}

impl BasicSerializedGameState for Vec<u8> {}

impl GameStateRecordsDAL<Vec<u8>> for SqliteGameStateRecordsDAL {
    fn get_game_state_record(&mut self, state_hash: &Vec<u8>) -> Option<GameStateRecord> {
        match try_get_state_record_from_db(&self.read_only_connection, &self.game_name, &state_hash)
        {
            Ok(Some(state_record)) => return Some(state_record),
            Ok(None) => return None,
            Err(err) => {
                println!(
                    "Failed to read state record from db, returning None. Error: {}",
                    err
                );
                return None;
            }
        }
    }

    fn increment_game_state_records_values_in_background(
        &self,
        increment_tasks: Vec<IncrementPersistedGameStateRecordValuesTask<Vec<u8>>>,
    ) -> thread::JoinHandle<()> {
        let sqlite_db_path = self.sqlite_db_path.clone();
        let game_name = self.game_name.clone();

        return thread::spawn(move || {
            let write_connection: Connection;

            match Connection::open(&sqlite_db_path) {
                Ok(opened_connection) => {
                    write_connection = opened_connection;
                }
                Err(err) => {
                    println!("Failed to open write connection for persisting updates. Updates will be dropped. Error: {}", err);
                    return;
                }
            }

            for increment_task in increment_tasks.iter() {
                let mut attempts_counter: u8 = 0;

                'attempts_loop: while attempts_counter < MAX_ATTEMPTS_PER_UPDATE {
                    let increment_result = write_state_record_increment_to_db(
                        &write_connection,
                        &game_name,
                        &increment_task,
                    );

                    match increment_result {
                        Ok(_) => break 'attempts_loop,
                        Err(_) => attempts_counter += 1,
                    };
                }
            }
        });
    }
}

fn try_get_state_record_from_db(
    connection: &Connection,
    game_name: &str,
    state_hash: &Vec<u8>,
) -> rusqlite::Result<Option<GameStateRecord>> {
    let query_result = connection.query_row(
        "SELECT DrawsCount, LossesCount, WinsCount FROM GameStateRecords WHERE GameName = ?1 AND StateHash = ?2",
        rusqlite::params![game_name, &state_hash],
        |row| {
            return Ok(GameStateRecord {
                draws_count: row.get(0)?,
                losses_count: row.get(1)?,
                wins_count: row.get(2)?,
            });
        }
    );
    match query_result {
        Ok(state_record) => return Ok(Some(state_record)),
        Err(QueryReturnedNoRows) => return Ok(None),
        Err(err) => return Err(err),
    }
}

fn write_state_record_increment_to_db(
    connection: &Connection,
    game_name: &str,
    increment_task: &IncrementPersistedGameStateRecordValuesTask<Vec<u8>>,
) -> Result<(), rusqlite::Error> {
    match try_get_state_record_from_db(
        &connection,
        &game_name,
        &increment_task.serialized_game_state,
    )? {
        Some(old_state_record) => {
            let new_state_record = GameStateRecord::new(
                old_state_record.draws_count + increment_task.draws_count_addend,
                old_state_record.losses_count + increment_task.losses_count_addend,
                old_state_record.wins_count + increment_task.wins_count_addend,
            );

            connection.execute(
                "UPDATE GameStateRecords SET DrawsCount = ?1, LossesCount = ?2, WinsCount = ?3 WHERE GameName = ?4 AND StateHash = ?5",
                rusqlite::params![new_state_record.draws_count, new_state_record.losses_count, new_state_record.wins_count, game_name, &increment_task.serialized_game_state],
            )?;
        }
        None => {
            connection.execute(
                "INSERT INTO GameStateRecords (GameName, StateHash, DrawsCount, LossesCount, WinsCount) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    game_name,
                    &increment_task.serialized_game_state,
                    increment_task.draws_count_addend,
                    increment_task.losses_count_addend,
                    increment_task.wins_count_addend
                ],
            )?;
        }
    }

    return Ok(());
}

use crate::core::state_record::StateRecord;
use crate::core::state_record_provider::StateRecordProvider;
use rusqlite::Connection;
use rusqlite::Error::QueryReturnedNoRows;

pub struct SqliteStateRecordProvider<'a> {
    connection: &'a Connection,
    game_name: &'a str,
}

impl<'a> SqliteStateRecordProvider<'a> {
    pub fn new(connection: &'a Connection, game_name: &'a str) -> SqliteStateRecordProvider<'a> {
        return SqliteStateRecordProvider {
            connection: connection,
            game_name: game_name,
        };
    }
}

impl<'a> StateRecordProvider<Vec<u8>> for SqliteStateRecordProvider<'a> {
    fn get_state_record(&mut self, state_hash: &Vec<u8>) -> Option<StateRecord> {
        match try_get_state_record_from_db(self.connection, self.game_name, &state_hash) {
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

    fn update_state_record(&mut self, state_hash: Vec<u8>, did_draw: bool, did_win: bool) {
        match try_get_state_record_from_db(self.connection, self.game_name, &state_hash) {
            Ok(Some(old_state_record)) => {
                let new_state_record = StateRecord::new(
                    old_state_record.draws_count + if did_draw { 1 } else { 0 },
                    old_state_record.losses_count + if !did_draw && !did_win { 1 } else { 0 },
                    old_state_record.wins_count + if did_win { 1 } else { 0 },
                );

                match self.connection.execute(
                    "UPDATE GameStateRecords SET DrawsCount = ?1, LossesCount = ?2, WinsCount = ?3 WHERE GameName = ?4 AND StateHash = ?5",
                    rusqlite::params![new_state_record.draws_count, new_state_record.losses_count, new_state_record.wins_count, self.game_name, &state_hash],
                ) {
                    Ok(_) => return,
                    Err(err) => {
                        println!("Failed to update existing state record in db, failing update. Error: {}", err);
                        return;
                    }
                }
            }
            Ok(None) => {
                let new_state_record = StateRecord::new(
                    if did_draw { 1 } else { 0 },
                    if !did_draw && !did_win { 1 } else { 0 },
                    if did_win { 1 } else { 0 },
                );

                match self.connection.execute(
                    "INSERT INTO GameStateRecords (GameName, StateHash, DrawsCount, LossesCount, WinsCount) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![self.game_name, &state_hash, new_state_record.draws_count, new_state_record.losses_count, new_state_record.wins_count],
                ) {
                    Ok(_) => return,
                    Err(err) => {
                        println!("Failed to insert new state record in db, failing update. Error: {}", err);
                        return;
                    }
                }
            }
            Err(err) => {
                println!("Failed to read existing state record from db when attempting to update, failing update. Error: {}", err);
                return;
            }
        }
    }
}

fn try_get_state_record_from_db(
    connection: &Connection,
    game_name: &str,
    state_hash: &Vec<u8>,
) -> rusqlite::Result<Option<StateRecord>> {
    let query_result = connection.query_row(
        "SELECT DrawsCount, LossesCount, WinsCount FROM GameStateRecords WHERE GameName = ?1 AND StateHash = ?2",
        rusqlite::params![game_name, &state_hash],
        |row| {
            return Ok(StateRecord {
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

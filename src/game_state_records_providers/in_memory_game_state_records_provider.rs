use crate::structs::GameStateRecord;
use crate::traits::{BasicSerializedGameState, GameStateRecordsProvider};
use std::collections::HashMap;

pub struct InMemoryGameStateRecordsProvider<SerializedGameState: BasicSerializedGameState> {
    state_records: HashMap<SerializedGameState, GameStateRecord>,
}

impl<SerializedGameState: BasicSerializedGameState>
    InMemoryGameStateRecordsProvider<SerializedGameState>
{
    #[allow(dead_code)]
    pub fn new() -> InMemoryGameStateRecordsProvider<SerializedGameState> {
        return InMemoryGameStateRecordsProvider {
            state_records: HashMap::new(),
        };
    }
}

impl<SerializedGameState: BasicSerializedGameState> GameStateRecordsProvider<SerializedGameState>
    for InMemoryGameStateRecordsProvider<SerializedGameState>
{
    fn get_game_state_record(
        &mut self,
        state_hash: &SerializedGameState,
    ) -> Option<GameStateRecord> {
        match self.state_records.get(state_hash) {
            Some(state_record) => Some(*state_record),
            None => None,
        }
    }

    fn update_game_state_record(
        &mut self,
        state_hash: &SerializedGameState,
        did_draw: bool,
        did_win: bool,
    ) {
        let new_state_record: GameStateRecord;

        let state_records_key = state_hash.clone();

        match self.state_records.get(&state_records_key) {
            Some(&state_record) => {
                new_state_record = GameStateRecord::new(
                    state_record.draws_count + (if did_draw { 1 } else { 0 }),
                    state_record.losses_count + (if !did_draw && !did_win { 1 } else { 0 }),
                    state_record.wins_count + (if did_win { 1 } else { 0 }),
                )
            }
            None => {
                new_state_record = GameStateRecord::new(
                    if did_draw { 1 } else { 0 },
                    if !did_draw && !did_win { 1 } else { 0 },
                    if did_win { 1 } else { 0 },
                )
            }
        }

        self.state_records
            .insert(state_records_key, new_state_record);
    }
}

use crate::core::state_record::StateRecord;
use std::collections::HashMap;
use std::hash::Hash;

pub trait StateRecordProvider<HashedState: Eq + Hash> {
    fn get_state_record(&mut self, state_hash: &HashedState) -> Option<StateRecord>;

    fn update_state_record(&mut self, state_hash: HashedState, did_draw: bool, did_win: bool);
}

pub struct InMemoryStateByteVectorHashRecordProvider {
    state_records: HashMap<Vec<u8>, StateRecord>,
}

impl InMemoryStateByteVectorHashRecordProvider {
    #[allow(dead_code)]
    pub fn new() -> InMemoryStateByteVectorHashRecordProvider {
        return InMemoryStateByteVectorHashRecordProvider {
            state_records: HashMap::new(),
        };
    }
}

impl StateRecordProvider<Vec<u8>> for InMemoryStateByteVectorHashRecordProvider {
    fn get_state_record(&mut self, state_hash: &Vec<u8>) -> Option<StateRecord> {
        match self.state_records.get(state_hash) {
            Some(state_record) => Some(*state_record),
            None => None,
        }
    }

    fn update_state_record(&mut self, state_hash: Vec<u8>, did_draw: bool, did_win: bool) {
        let new_state_record: StateRecord;

        let state_records_key = state_hash.clone();

        match self.state_records.get(&state_records_key) {
            Some(&state_record) => {
                new_state_record = StateRecord::new(
                    state_record.draws_count + (if did_draw { 1 } else { 0 }),
                    state_record.losses_count + (if !did_draw && !did_win { 1 } else { 0 }),
                    state_record.wins_count + (if did_win { 1 } else { 0 }),
                )
            }
            None => {
                new_state_record = StateRecord::new(
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

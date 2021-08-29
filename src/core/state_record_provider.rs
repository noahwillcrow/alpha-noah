use crate::core::state_record::StateRecord;
use std::hash::Hash;

pub trait StateRecordProvider<HashedState: Eq + Hash + Clone> {
    fn get_state_record(&self, state_hash: HashedState) -> Option<StateRecord>;
    fn update_state_record(&mut self, state_hash: &HashedState, did_win: bool, did_draw: bool);
}

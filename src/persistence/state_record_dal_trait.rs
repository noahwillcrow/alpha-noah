use crate::core::state_record::StateRecord;
use std::hash::Hash;
use std::thread;

pub struct IncrementTask<HashedState: Eq + Hash + Send> {
    pub state_hash: HashedState,
    pub draws_count_addend: i32,
    pub losses_count_addend: i32,
    pub wins_count_addend: i32,
}

pub trait StateRecordDAL<HashedState: Eq + Hash + Send> {
    fn get_state_record(&mut self, state_hash: &HashedState) -> Option<StateRecord>;

    fn increment_state_records_values_in_background(
        &self,
        increment_tasks: Vec<IncrementTask<HashedState>>,
    ) -> thread::JoinHandle<()>;
}

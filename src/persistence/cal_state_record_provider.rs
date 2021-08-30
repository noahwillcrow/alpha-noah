use crate::core::state_record::StateRecord;
use crate::core::state_record_provider::StateRecordProvider;
use crate::persistence::state_record_dal_trait::{IncrementTask, StateRecordDAL};
use lru::LruCache;
use std::cmp;
use std::hash::Hash;
use std::thread;

const CAPACITY_CLEARANCE_DIVISOR: usize = 5;

pub struct CALStateRecordProvider<'a, HashedState: Eq + Hash + Clone + Send + Sync> {
    lru_cache: LruCache<HashedState, (StateRecord, StateRecord)>,
    max_capacity: usize,
    state_record_dal: &'a mut (dyn StateRecordDAL<HashedState> + Send),
}

impl<'a, HashedState: Eq + Hash + Clone + Send + Sync> CALStateRecordProvider<'a, HashedState> {
    pub fn new(
        max_capacity: usize,
        state_record_dal: &'a mut (dyn StateRecordDAL<HashedState> + Send),
    ) -> CALStateRecordProvider<HashedState> {
        return CALStateRecordProvider {
            lru_cache: LruCache::unbounded(),
            max_capacity: max_capacity,
            state_record_dal: state_record_dal,
        };
    }

    pub fn try_commit_lru_updates_to_dal_in_background(
        &mut self,
        number_to_commit: usize,
    ) -> thread::JoinHandle<()> {
        // while in the same thread, pull out the updates to commit to dal
        let mut increment_tasks: Vec<IncrementTask<HashedState>> = vec![];
        'while_loop: while increment_tasks.len() < number_to_commit {
            match self.lru_cache.pop_lru() {
                Some((state_hash, (_, pending_updates_state_record))) => {
                    increment_tasks.push(IncrementTask {
                        state_hash: state_hash,
                        draws_count_addend: pending_updates_state_record.draws_count,
                        losses_count_addend: pending_updates_state_record.losses_count,
                        wins_count_addend: pending_updates_state_record.wins_count,
                    });
                }
                None => {
                    break 'while_loop;
                }
            }
        }

        return self
            .state_record_dal
            .increment_state_records_values_in_background(increment_tasks);
    }
}

impl<'a, HashedState: Eq + Hash + Clone + Send + Send + Sync> StateRecordProvider<HashedState>
    for CALStateRecordProvider<'a, HashedState>
{
    fn get_state_record(&mut self, state_hash: &HashedState) -> Option<StateRecord> {
        match self.lru_cache.get(state_hash) {
            Some((original_state_record, pending_updates_state_record)) => Some(StateRecord::new(
                original_state_record.draws_count + pending_updates_state_record.draws_count,
                original_state_record.losses_count + pending_updates_state_record.losses_count,
                original_state_record.wins_count + pending_updates_state_record.wins_count,
            )),
            None => match self.state_record_dal.get_state_record(state_hash) {
                Some(state_record) => {
                    if self.lru_cache.len() >= self.max_capacity {
                        self.try_commit_lru_updates_to_dal_in_background(cmp::max(
                            self.max_capacity / CAPACITY_CLEARANCE_DIVISOR,
                            self.lru_cache.len() - (self.max_capacity - 1),
                        ));
                    }

                    self.lru_cache.put(
                        state_hash.clone(),
                        (state_record.clone(), StateRecord::new(0, 0, 0)),
                    );
                    return Some(state_record.clone());
                }
                None => None,
            },
        }
    }

    fn update_state_record(&mut self, state_hash: HashedState, did_draw: bool, did_win: bool) {
        let cached_record = self.lru_cache.get(&state_hash);

        let new_cache_value: Option<(StateRecord, StateRecord)>;

        match cached_record {
            Some((original_state_record, pending_updates_state_record)) => {
                new_cache_value = Some((
                    original_state_record.clone(),
                    StateRecord::new(
                        pending_updates_state_record.draws_count + if did_draw { 1 } else { 0 },
                        pending_updates_state_record.losses_count
                            + if !did_draw && !did_win { 1 } else { 0 },
                        pending_updates_state_record.wins_count + if did_win { 1 } else { 0 },
                    ),
                ));
            }
            None => match self.state_record_dal.get_state_record(&state_hash) {
                Some(state_record) => {
                    if self.lru_cache.len() >= self.max_capacity {
                        self.try_commit_lru_updates_to_dal_in_background(cmp::max(
                            self.max_capacity / CAPACITY_CLEARANCE_DIVISOR,
                            self.lru_cache.len() - (self.max_capacity - 1),
                        ));
                    }

                    new_cache_value = Some((
                        state_record.clone(),
                        StateRecord::new(
                            if did_draw { 1 } else { 0 },
                            if !did_draw && !did_win { 1 } else { 0 },
                            if did_win { 1 } else { 0 },
                        ),
                    ));
                }
                None => new_cache_value = None,
            },
        }

        match new_cache_value {
            Some(cacheable_tuple) => {
                self.lru_cache.put(state_hash.clone(), cacheable_tuple);
            }
            None => (),
        };
    }
}

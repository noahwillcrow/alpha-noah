use crate::core::state_record::StateRecord;
use crate::core::state_record_provider::StateRecordProvider;
use lru::LruCache;
use std::hash::Hash;

pub struct LruCacheStateRecordProvider<'a, HashedState: Eq + Hash> {
    base_state_record_provider: &'a mut dyn StateRecordProvider<HashedState>,
    lru_cache: LruCache<HashedState, StateRecord>,
}

impl<'a, HashedState: Eq + Hash> LruCacheStateRecordProvider<'a, HashedState> {
    pub fn new(
        base_state_record_provider: &'a mut dyn StateRecordProvider<HashedState>,
        max_capacity: usize,
    ) -> LruCacheStateRecordProvider<HashedState> {
        return LruCacheStateRecordProvider {
            base_state_record_provider: base_state_record_provider,
            lru_cache: LruCache::new(max_capacity),
        };
    }
}

impl<'a, HashedState: Eq + Hash> StateRecordProvider<HashedState>
    for LruCacheStateRecordProvider<'a, HashedState>
{
    fn get_state_record(&mut self, state_hash: &HashedState) -> Option<StateRecord> {
        match self.lru_cache.get(state_hash) {
            Some(state_record) => Some(*state_record),
            None => None,
        }
    }

    fn update_state_record(&mut self, state_hash: HashedState, did_draw: bool, did_win: bool) {
        self.lru_cache.pop(&state_hash);
        self.base_state_record_provider
            .update_state_record(state_hash, did_draw, did_win);
    }
}

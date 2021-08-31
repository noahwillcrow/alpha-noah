use crate::structs::{GameStateRecord, IncrementPersistedGameStateRecordValuesTask};
use crate::traits::{
    BasicSerializedGameState, GameStateRecordsDAL, GameStateRecordsProvider, PendingUpdatesManager,
};
use lru::LruCache;
use std::cell::RefCell;
use std::cmp;
use std::rc::Rc;
use std::thread;

const CAPACITY_CLEARANCE_DIVISOR: usize = 5;

pub struct LruCacheFrontedGameStateRecordsProvider<SerializedGameState: BasicSerializedGameState> {
    lru_cache: LruCache<SerializedGameState, (GameStateRecord, GameStateRecord)>,
    max_capacity: usize,
    game_state_records_dal_rc: Rc<RefCell<dyn GameStateRecordsDAL<SerializedGameState>>>,
}

impl<SerializedGameState: BasicSerializedGameState>
    LruCacheFrontedGameStateRecordsProvider<SerializedGameState>
{
    pub fn new<DalType: GameStateRecordsDAL<SerializedGameState> + 'static>(
        max_capacity: usize,
        game_state_records_dal_rc: Rc<RefCell<DalType>>,
    ) -> LruCacheFrontedGameStateRecordsProvider<SerializedGameState> {
        return LruCacheFrontedGameStateRecordsProvider {
            lru_cache: LruCache::unbounded(),
            max_capacity: max_capacity,
            game_state_records_dal_rc: game_state_records_dal_rc,
        };
    }
}

impl<SerializedGameState: BasicSerializedGameState + Send + Send + Sync> PendingUpdatesManager
    for LruCacheFrontedGameStateRecordsProvider<SerializedGameState>
{
    fn try_commit_pending_updates_in_background(
        &mut self,
        max_number_to_commit: usize,
    ) -> thread::JoinHandle<()> {
        // while in the same thread, pull out the updates to commit to dal
        let mut increment_tasks: Vec<
            IncrementPersistedGameStateRecordValuesTask<SerializedGameState>,
        > = vec![];
        'while_loop: while increment_tasks.len() < max_number_to_commit {
            match self.lru_cache.pop_lru() {
                Some((serialized_game_state, (_, pending_updates_game_state_record))) => {
                    increment_tasks.push(IncrementPersistedGameStateRecordValuesTask {
                        serialized_game_state: serialized_game_state,
                        draws_count_addend: pending_updates_game_state_record.draws_count,
                        losses_count_addend: pending_updates_game_state_record.losses_count,
                        wins_count_addend: pending_updates_game_state_record.wins_count,
                    });
                }
                None => {
                    break 'while_loop;
                }
            }
        }

        return self
            .game_state_records_dal_rc
            .borrow_mut()
            .increment_game_state_records_values_in_background(increment_tasks);
    }
}

impl<SerializedGameState: BasicSerializedGameState + Send + Send + Sync>
    GameStateRecordsProvider<SerializedGameState>
    for LruCacheFrontedGameStateRecordsProvider<SerializedGameState>
{
    fn get_game_state_record(
        &mut self,
        serialized_game_state: &SerializedGameState,
    ) -> Option<GameStateRecord> {
        let original_game_state_record_option: Option<GameStateRecord>;

        match self.lru_cache.get(serialized_game_state) {
            Some((original_game_state_record, pending_updates_game_state_record)) => {
                return Some(GameStateRecord::new(
                    original_game_state_record.draws_count
                        + pending_updates_game_state_record.draws_count,
                    original_game_state_record.losses_count
                        + pending_updates_game_state_record.losses_count,
                    original_game_state_record.wins_count
                        + pending_updates_game_state_record.wins_count,
                ))
            }
            None => match self
                .game_state_records_dal_rc
                .borrow_mut()
                .get_game_state_record(serialized_game_state)
            {
                Some(game_state_record) => {
                    original_game_state_record_option = Some(game_state_record.clone())
                }
                None => original_game_state_record_option = None,
            },
        }

        match original_game_state_record_option {
            Some(original_game_state_record) => {
                if self.lru_cache.len() >= self.max_capacity {
                    self.try_commit_pending_updates_in_background(cmp::max(
                        self.max_capacity / CAPACITY_CLEARANCE_DIVISOR,
                        self.lru_cache.len() - (self.max_capacity - 1),
                    ));
                }
                self.lru_cache.put(
                    serialized_game_state.clone(),
                    (
                        original_game_state_record.clone(),
                        GameStateRecord::new(0, 0, 0),
                    ),
                );
                return Some(original_game_state_record);
            }
            None => return None,
        }
    }

    fn update_game_state_record(
        &mut self,
        serialized_game_state: &SerializedGameState,
        did_draw: bool,
        did_win: bool,
    ) {
        let cached_record = self.lru_cache.get(serialized_game_state);

        let new_cache_value: Option<(GameStateRecord, GameStateRecord)>;

        match cached_record {
            Some((original_game_state_record, pending_updates_game_state_record)) => {
                new_cache_value = Some((
                    original_game_state_record.clone(),
                    GameStateRecord::new(
                        pending_updates_game_state_record.draws_count
                            + if did_draw { 1 } else { 0 },
                        pending_updates_game_state_record.losses_count
                            + if !did_draw && !did_win { 1 } else { 0 },
                        pending_updates_game_state_record.wins_count + if did_win { 1 } else { 0 },
                    ),
                ));
            }
            None => match self
                .game_state_records_dal_rc
                .borrow_mut()
                .get_game_state_record(&serialized_game_state)
            {
                Some(game_state_record) => {
                    new_cache_value = Some((
                        game_state_record.clone(),
                        GameStateRecord::new(
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
                if self.lru_cache.len() >= self.max_capacity {
                    self.try_commit_pending_updates_in_background(cmp::max(
                        self.max_capacity / CAPACITY_CLEARANCE_DIVISOR,
                        self.lru_cache.len() - (self.max_capacity - 1),
                    ));
                }

                self.lru_cache
                    .put(serialized_game_state.clone(), cacheable_tuple);
            }
            None => (),
        };
    }
}

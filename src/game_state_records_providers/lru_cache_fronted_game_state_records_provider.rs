use crate::structs::{
    GameReport, GameStateRecord, GameStateUpdate, IncrementPersistedGameStateRecordValuesTask,
};
use crate::traits::{
    BasicSerializedGameState, GameReportsProcessor, GameStateRecordsDAL, GameStateRecordsFetcher,
    PendingUpdatesManager,
};
use lru::LruCache;
use std::cell::{RefCell, RefMut};
use std::cmp;
use std::collections::HashSet;
use std::thread;
use std::time;

const CAPACITY_CLEARANCE_DIVISOR: usize = 5;

pub struct LruCacheFrontedGameStateRecordsProvider<
    'a,
    SerializedGameState: BasicSerializedGameState,
> {
    lru_cache_ref_cell: RefCell<LruCache<SerializedGameState, (GameStateRecord, GameStateRecord)>>,
    max_capacity: usize,
    game_state_records_dal: &'a dyn GameStateRecordsDAL<SerializedGameState>,
}

impl<'a, SerializedGameState: BasicSerializedGameState>
    LruCacheFrontedGameStateRecordsProvider<'a, SerializedGameState>
{
    pub fn new(
        max_capacity: usize,
        game_state_records_dal: &'a dyn GameStateRecordsDAL<SerializedGameState>,
    ) -> LruCacheFrontedGameStateRecordsProvider<'a, SerializedGameState> {
        return LruCacheFrontedGameStateRecordsProvider {
            lru_cache_ref_cell: RefCell::new(LruCache::unbounded()),
            max_capacity: max_capacity,
            game_state_records_dal: game_state_records_dal,
        };
    }

    fn safe_get_lru_cache_mut(
        &self,
    ) -> RefMut<LruCache<SerializedGameState, (GameStateRecord, GameStateRecord)>> {
        loop {
            match self.lru_cache_ref_cell.try_borrow_mut() {
                Ok(value) => return value,
                Err(_) => {
                    println!("Failed to get lru cache mutable reference!?");
                    thread::sleep(time::Duration::from_millis(10));
                }
            }
        }
    }
}

impl<'a, SerializedGameState: BasicSerializedGameState + Send + Send + Sync> PendingUpdatesManager
    for LruCacheFrontedGameStateRecordsProvider<'a, SerializedGameState>
{
    fn try_commit_pending_updates_in_background(
        &self,
        max_number_to_commit: usize,
    ) -> thread::JoinHandle<()> {
        // while in the same thread, pull out the updates to commit to dal
        let mut increment_tasks: Vec<
            IncrementPersistedGameStateRecordValuesTask<SerializedGameState>,
        > = vec![];
        let mut lru_cache = self.safe_get_lru_cache_mut();

        while let Some((serialized_game_state, (_, pending_updates_game_state_record))) =
            lru_cache.pop_lru()
        {
            increment_tasks.push(IncrementPersistedGameStateRecordValuesTask {
                serialized_game_state: serialized_game_state,
                draws_count_addend: pending_updates_game_state_record.draws_count,
                losses_count_addend: pending_updates_game_state_record.losses_count,
                wins_count_addend: pending_updates_game_state_record.wins_count,
            });

            if increment_tasks.len() == max_number_to_commit {
                break;
            }
        }

        return self
            .game_state_records_dal
            .increment_game_state_records_values_in_background(increment_tasks);
    }
}

impl<'a, SerializedGameState: BasicSerializedGameState + Send + Send + Sync>
    GameStateRecordsFetcher<SerializedGameState>
    for LruCacheFrontedGameStateRecordsProvider<'a, SerializedGameState>
{
    fn get_game_state_record(
        &self,
        serialized_game_state: &SerializedGameState,
    ) -> Option<GameStateRecord> {
        let original_game_state_record: GameStateRecord;

        let mut pending_updates_count_to_commit = 0;

        {
            let mut lru_cache = self.safe_get_lru_cache_mut();

            match lru_cache.get(serialized_game_state) {
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
                    .game_state_records_dal
                    .get_game_state_record(serialized_game_state)
                {
                    Some(dal_game_state_record) => {
                        original_game_state_record = dal_game_state_record.clone()
                    }
                    None => original_game_state_record = GameStateRecord::new_zeros(),
                },
            }

            // Must be a value that wasn't in the cache yet

            lru_cache.put(
                serialized_game_state.clone(),
                (
                    original_game_state_record.clone(),
                    GameStateRecord::new_zeros(),
                ),
            );

            if lru_cache.len() >= self.max_capacity {
                pending_updates_count_to_commit = cmp::max(
                    self.max_capacity / CAPACITY_CLEARANCE_DIVISOR,
                    lru_cache.len() - (self.max_capacity - 1),
                );
            }
        }

        // Must be a value that wasn't in the cache yet
        if pending_updates_count_to_commit > 0 {
            self.try_commit_pending_updates_in_background(pending_updates_count_to_commit);
        }

        return Some(original_game_state_record);
    }
}

impl<'a, SerializedGameState: BasicSerializedGameState + Send + Send + Sync>
    GameReportsProcessor<SerializedGameState, ()>
    for LruCacheFrontedGameStateRecordsProvider<'a, SerializedGameState>
{
    fn process_game_report(
        &self,
        game_report: &mut GameReport<SerializedGameState>,
    ) -> Result<(), ()> {
        let did_draw = game_report.winning_player_index == -1;

        let mut pending_updates_count_to_commit = 0;

        {
            let mut lru_cache = self.safe_get_lru_cache_mut();

            let mut already_updated_game_state_updates: HashSet<
                GameStateUpdate<SerializedGameState>,
            > = HashSet::new();
            for game_state_update in game_report.game_state_updates.iter() {
                if already_updated_game_state_updates.contains(&game_state_update) {
                    continue;
                }

                already_updated_game_state_updates.insert(game_state_update.clone());

                let did_win =
                    game_report.winning_player_index == game_state_update.responsible_player_index;
                let mut is_in_cache = false;
                let new_cache_value: (GameStateRecord, GameStateRecord);
                match lru_cache.get(&game_state_update.new_serialized_game_state) {
                    Some((original_game_state_record, pending_updates_game_state_record)) => {
                        is_in_cache = true;
                        new_cache_value = (
                            original_game_state_record.clone(),
                            GameStateRecord::new(
                                pending_updates_game_state_record.draws_count
                                    + if did_draw { 1 } else { 0 },
                                pending_updates_game_state_record.losses_count
                                    + if !did_draw && !did_win { 1 } else { 0 },
                                pending_updates_game_state_record.wins_count
                                    + if did_win { 1 } else { 0 },
                            ),
                        );
                    }
                    None => match self
                        .game_state_records_dal
                        .get_game_state_record(&game_state_update.new_serialized_game_state)
                    {
                        Some(game_state_record) => {
                            new_cache_value = (
                                game_state_record.clone(),
                                GameStateRecord::new(
                                    if did_draw { 1 } else { 0 },
                                    if !did_draw && !did_win { 1 } else { 0 },
                                    if did_win { 1 } else { 0 },
                                ),
                            );
                        }
                        None => {
                            new_cache_value =
                                (GameStateRecord::new_zeros(), GameStateRecord::new_zeros())
                        }
                    },
                }

                if !is_in_cache && lru_cache.len() >= self.max_capacity {
                    pending_updates_count_to_commit = cmp::max(
                        self.max_capacity / CAPACITY_CLEARANCE_DIVISOR,
                        lru_cache.len() - (self.max_capacity - 1),
                    );
                }

                lru_cache.put(
                    game_state_update.new_serialized_game_state.clone(),
                    new_cache_value,
                );
            }
        }

        if pending_updates_count_to_commit > 0 {
            self.try_commit_pending_updates_in_background(pending_updates_count_to_commit);
        }

        return Ok(());
    }
}

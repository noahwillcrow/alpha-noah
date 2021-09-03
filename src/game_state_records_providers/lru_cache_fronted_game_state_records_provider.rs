use crate::structs::{
    GameReport, GameStateRecord, GameStateUpdate, IncrementPersistedGameStateRecordValuesTask,
};
use crate::traits::{
    BasicSerializedGameState, GameReportsProcessor, GameStateRecordsDAL, GameStateRecordsFetcher,
    PendingUpdatesManager,
};
use lru::LruCache;
use std::cell::RefCell;
use std::cmp;
use std::collections::HashSet;
use std::thread;

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
        'while_loop: while increment_tasks.len() < max_number_to_commit {
            match self.lru_cache_ref_cell.borrow_mut().pop_lru() {
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

        match self
            .lru_cache_ref_cell
            .borrow_mut()
            .get(serialized_game_state)
        {
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
        if self.lru_cache_ref_cell.borrow().len() >= self.max_capacity {
            self.try_commit_pending_updates_in_background(cmp::max(
                self.max_capacity / CAPACITY_CLEARANCE_DIVISOR,
                self.lru_cache_ref_cell.borrow().len() - (self.max_capacity - 1),
            ));
        }

        self.lru_cache_ref_cell.borrow_mut().put(
            serialized_game_state.clone(),
            (
                original_game_state_record.clone(),
                GameStateRecord::new_zeros(),
            ),
        );

        return Some(original_game_state_record);
    }
}

impl<'a, SerializedGameState: BasicSerializedGameState + Send + Send + Sync>
    GameReportsProcessor<SerializedGameState, ()>
    for LruCacheFrontedGameStateRecordsProvider<'a, SerializedGameState>
{
    fn process_game_report(&self, game_report: GameReport<SerializedGameState>) -> Result<(), ()> {
        let did_draw = game_report.winning_player_index == -1;

        let mut already_updated_game_state_updates: HashSet<GameStateUpdate<SerializedGameState>> =
            HashSet::new();
        for game_state_update in game_report.game_state_updates.iter() {
            if already_updated_game_state_updates.contains(&game_state_update) {
                continue;
            }

            already_updated_game_state_updates.insert(game_state_update.clone());

            let did_win =
                game_report.winning_player_index == game_state_update.responsible_player_index;
            let mut is_in_cache = false;
            let new_cache_value: (GameStateRecord, GameStateRecord);
            match self
                .lru_cache_ref_cell
                .borrow_mut()
                .get(&game_state_update.new_serialized_game_state)
            {
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

            if !is_in_cache && self.lru_cache_ref_cell.borrow().len() >= self.max_capacity {
                self.try_commit_pending_updates_in_background(cmp::max(
                    self.max_capacity / CAPACITY_CLEARANCE_DIVISOR,
                    self.lru_cache_ref_cell.borrow().len() - (self.max_capacity - 1),
                ));
            }

            self.lru_cache_ref_cell.borrow_mut().put(
                game_state_update.new_serialized_game_state.clone(),
                new_cache_value,
            );
        }

        return Ok(());
    }
}

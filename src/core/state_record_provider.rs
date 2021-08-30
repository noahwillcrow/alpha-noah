use crate::core::state_record::StateRecord;
use std::collections::HashMap;
use std::hash::Hash;

pub trait StateRecordProvider<HashedState: Eq + Hash + Clone> {
    fn get_state_record(
        &self,
        current_player_index: i32,
        state_hash: HashedState,
    ) -> Option<StateRecord>;

    fn update_state_record(
        &mut self,
        state_hash: &HashedState,
        winning_player_index: i32,
        number_of_players: i32,
    );
}

pub struct InMemoryStateRecordProvider<HashedState: Eq + Hash + Clone> {
    state_records: HashMap<(i32, HashedState), StateRecord>,
}

impl<HashedState: Eq + Hash + Clone> InMemoryStateRecordProvider<HashedState> {
    pub fn new() -> InMemoryStateRecordProvider<HashedState> {
        return InMemoryStateRecordProvider {
            state_records: HashMap::new(),
        };
    }
}

impl<HashedState: Eq + Hash + Clone> StateRecordProvider<HashedState>
    for InMemoryStateRecordProvider<HashedState>
{
    fn get_state_record(
        &self,
        current_player_index: i32,
        state_hash: HashedState,
    ) -> Option<StateRecord> {
        match self.state_records.get(&(current_player_index, state_hash)) {
            Some(&state_record) => Some(state_record.clone()),
            None => None,
        }
    }

    fn update_state_record(
        &mut self,
        state_hash: &HashedState,
        winning_player_index: i32,
        number_of_players: i32,
    ) {
        // make some players never learn
        if winning_player_index == 0 {
            return;
        }

        let did_draw = winning_player_index == -1;

        for player_index in 0..number_of_players {
            let did_win = winning_player_index == player_index;
            let new_state_record: StateRecord;

            let state_records_index = (player_index, state_hash.clone());

            match self.state_records.get(&state_records_index) {
                Some(&state_record) => {
                    new_state_record = StateRecord::new(
                        state_record.draws_count + (if did_draw { 1 } else { 0 }),
                        state_record.losses_count,
                        state_record.wins_count + (if did_win { 1 } else { 0 }),
                    )
                }
                None => {
                    new_state_record = StateRecord::new(
                        if did_draw { 1 } else { 0 },
                        0,
                        if did_win { 1 } else { 0 },
                    )
                }
            }

            self.state_records
                .insert(state_records_index, new_state_record);
        }
    }
}

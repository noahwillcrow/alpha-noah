use crate::core::state_record::StateRecord;

pub fn count_visits(state_record: &StateRecord) -> i32 {
    return state_record.draws_count + state_record.losses_count + state_record.wins_count;
}

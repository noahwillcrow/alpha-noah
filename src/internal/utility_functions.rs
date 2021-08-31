use crate::structs::GameStateRecord;

pub fn count_visits(game_state_record: &GameStateRecord) -> i32 {
    return game_state_record.draws_count
        + game_state_record.losses_count
        + game_state_record.wins_count;
}

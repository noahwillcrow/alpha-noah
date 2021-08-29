#[derive(Copy, Clone)]
pub struct StateRecord {
    pub draws_count: i32,
    pub losses_count: i32,
    pub wins_count: i32,
}

impl StateRecord {
    pub fn new(draws_count: i32, losses_count: i32, wins_count: i32) -> StateRecord {
        return StateRecord {
            draws_count: draws_count,
            losses_count: losses_count,
            wins_count: wins_count,
        };
    }
}

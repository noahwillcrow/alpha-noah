pub enum DecideNextStateError {
    Unknown,
    NoAvailableStatesError,
}

pub enum RunGameError {
    #[allow(dead_code)]
    Unknown,
    UnableToDecideNextState(i32),
}

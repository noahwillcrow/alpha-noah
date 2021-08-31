pub enum DecideNextStateError {
    Unknown,
    NoAvailableStatesError,
}

pub enum RunGameError {
    Unknown,
    UnableToDecideNextState(i32),
}

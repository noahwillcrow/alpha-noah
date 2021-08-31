#[derive(Debug)]
pub enum DecideNextStateError {
    Unknown,
    NoAvailableStatesError,
}

#[derive(Debug)]
pub enum RunGameError {
    #[allow(dead_code)]
    Unknown,
    UnableToDecideNextState(i32),
}

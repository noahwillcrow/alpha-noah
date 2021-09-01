use crate::traits::BasicGameState;

pub type GameStateType = [[u8; 8]; 8];

impl BasicGameState for GameStateType {}

use crate::traits::BasicSerializedGameState;
use std::hash::Hash;

#[derive(Copy, Clone)]
pub struct GameStateRecord {
    pub draws_count: i32,
    pub losses_count: i32,
    pub wins_count: i32,
}

impl GameStateRecord {
    pub fn new(draws_count: i32, losses_count: i32, wins_count: i32) -> GameStateRecord {
        return GameStateRecord {
            draws_count: draws_count,
            losses_count: losses_count,
            wins_count: wins_count,
        };
    }

    pub fn new_zeros() -> GameStateRecord {
        return GameStateRecord {
            draws_count: 0,
            losses_count: 0,
            wins_count: 0,
        };
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct GameStateUpdate<SerializedGameState: BasicSerializedGameState> {
    pub new_serialized_game_state: SerializedGameState,
    pub responsible_player_index: i32,
}

#[derive(Clone)]
pub struct GameReport<SerializedGameState: BasicSerializedGameState> {
    pub game_state_updates: Vec<GameStateUpdate<SerializedGameState>>,
    pub number_of_players: i32,
    pub winning_player_index: i32,
}

#[derive(Clone)]
pub struct IncrementPersistedGameStateRecordValuesTask<
    SerializedGameState: BasicSerializedGameState,
> {
    pub serialized_game_state: SerializedGameState,
    pub draws_count_addend: i32,
    pub losses_count_addend: i32,
    pub wins_count_addend: i32,
}

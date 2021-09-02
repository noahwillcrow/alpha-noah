use crate::enums::{DecideNextStateError, RunGameError};
use crate::structs::{GameReport, GameStateRecord, IncrementPersistedGameStateRecordValuesTask};
use std::hash::Hash;

pub trait BasicGameState: Clone {}

pub trait BasicSerializedGameState: Clone + Eq + Hash + PartialEq {}

pub trait CLIGameStateFormatter<GameState: BasicGameState> {
    fn format_game_state_for_cli(&self, game_state: &GameState) -> String;
}

pub trait GameReportsPersister<SerializedGameState: BasicSerializedGameState, ErrorType> {
    fn persist_game_report(
        &mut self,
        game_report: GameReport<SerializedGameState>,
    ) -> Result<(), ErrorType>;
}

pub trait GameRulesAuthority<GameState: BasicGameState> {
    /// Analayzes the given game state to determine if it is terminal.
    /// If the given game state is terminal, then the function will return Some(i32),
    /// with the nested integer being the winning player index.
    /// Otherwise, it will return None.
    fn analyze_game_state_for_terminality(
        &self,
        game_state: &GameState,
        next_player_index: i32,
    ) -> Option<i32>;

    fn find_available_next_game_states(
        &self,
        current_player_index: i32,
        current_game_state: &GameState,
    ) -> Vec<GameState>;
}

pub trait GameRunner<GameState: BasicGameState, SerializedGameState: BasicSerializedGameState> {
    fn run_game(
        &mut self,
        initial_game_state: GameState,
        turn_takers: &mut Vec<&mut dyn TurnTaker<GameState>>,
        max_number_of_turns: i32,
        is_reaching_max_number_of_turns_a_draw: bool,
    ) -> Result<Option<GameReport<SerializedGameState>>, RunGameError>;
}

pub trait GameStateDeserializer<
    GameState: BasicGameState,
    SerializedGameState: BasicSerializedGameState,
>
{
    fn deserialize_game_state(
        &self,
        serialized_game_state: &SerializedGameState,
    ) -> (i32, GameState);
}

pub trait GameStateRecordsDAL<SerializedGameState: BasicSerializedGameState> {
    fn get_game_state_record(
        &mut self,
        serialized_game_state: &SerializedGameState,
    ) -> Option<GameStateRecord>;

    fn increment_game_state_records_values_in_background(
        &self,
        increment_tasks: Vec<IncrementPersistedGameStateRecordValuesTask<SerializedGameState>>,
    ) -> std::thread::JoinHandle<()>;
}

pub trait GameStateRecordsProvider<SerializedGameState: BasicSerializedGameState> {
    fn get_game_state_record(
        &mut self,
        serialized_game_state: &SerializedGameState,
    ) -> Option<GameStateRecord>;

    fn update_game_state_record(
        &mut self,
        serialized_game_state: &SerializedGameState,
        did_draw: bool,
        did_win: bool,
    );
}

pub trait GameStateSerializer<
    GameState: BasicGameState,
    SerializedGameState: BasicSerializedGameState,
>
{
    fn serialize_game_state(
        &self,
        responsible_player_index: i32,
        game_state: &GameState,
    ) -> SerializedGameState;
}

pub trait GameStateWeightsCalculator<GameState: BasicGameState> {
    fn weigh_game_states(
        &self,
        responsible_player_index: i32,
        game_states: &Vec<GameState>,
    ) -> Vec<f32>;
}

pub trait PendingUpdatesManager {
    fn try_commit_pending_updates_in_background(
        &mut self,
        max_number_to_commit: usize,
    ) -> std::thread::JoinHandle<()>;
}

pub trait TurnTaker<GameState: BasicGameState> {
    fn decide_next_game_state(
        &mut self,
        current_game_state: &GameState,
    ) -> Result<GameState, DecideNextStateError>;
}

pub trait UserInputGameStateCreator<GameState: BasicGameState, UserInputType> {
    fn create_new_game_state_from_user_input(
        &mut self,
        current_player_index: i32,
        current_game_state: &GameState,
        user_input: UserInputType,
    ) -> Result<GameState, String>;
}

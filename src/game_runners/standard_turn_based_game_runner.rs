use crate::enums::RunGameError;
use crate::structs::{GameReport, GameStateUpdate};
use crate::traits::{
    BasicGameState, BasicSerializedGameState, GameRunner, GameStateSerializer,
    TerminalGameStateAnalyzer, TurnTaker,
};

pub struct StandardTurnBasedGameRunner<
    'a,
    GameState: BasicGameState,
    SerializedGameState: BasicSerializedGameState,
> {
    game_state_serializer: &'a dyn GameStateSerializer<GameState, SerializedGameState>,
    terminal_game_state_analyzer: &'a dyn TerminalGameStateAnalyzer<GameState>,
}

impl<'a, GameState: BasicGameState, SerializedGameState: BasicSerializedGameState>
    StandardTurnBasedGameRunner<'a, GameState, SerializedGameState>
{
    pub fn new(
        game_state_serializer: &'a dyn GameStateSerializer<GameState, SerializedGameState>,
        terminal_game_state_analyzer: &'a dyn TerminalGameStateAnalyzer<GameState>,
    ) -> StandardTurnBasedGameRunner<'a, GameState, SerializedGameState> {
        return StandardTurnBasedGameRunner {
            game_state_serializer: game_state_serializer,
            terminal_game_state_analyzer: terminal_game_state_analyzer,
        };
    }
}

impl<'a, GameState: BasicGameState, SerializedGameState: BasicSerializedGameState>
    GameRunner<GameState, SerializedGameState>
    for StandardTurnBasedGameRunner<'a, GameState, SerializedGameState>
{
    fn run_game(
        &mut self,
        initial_game_state: GameState,
        turn_takers: &mut Vec<&mut dyn TurnTaker<GameState>>,
        max_number_of_turns: i32,
        is_reaching_max_number_of_turns_a_draw: bool,
    ) -> Result<Option<GameReport<SerializedGameState>>, RunGameError> {
        let mut game_state_updates: Vec<GameStateUpdate<SerializedGameState>> =
            vec![GameStateUpdate {
                new_serialized_game_state: self
                    .game_state_serializer
                    .serialize_game_state(-1, &initial_game_state),
                responsible_player_index: -1,
            }];

        let number_of_players = turn_takers.len() as i32;

        let mut current_player_index: i32 = -1;
        let mut current_game_state = initial_game_state;
        let mut turns_counter = 0;

        while max_number_of_turns == -1 || turns_counter < max_number_of_turns {
            let next_player_index = (current_player_index + 1) % number_of_players;

            // Check if the current state is terminal given the next player index before starting their turn
            match self
                .terminal_game_state_analyzer
                .analyze_game_state_for_terminality(&current_game_state, next_player_index)
            {
                None => (),
                Some(winning_player_index) => {
                    // A terminal index was reached - just return, we're done here.
                    return Ok(Some(GameReport {
                        game_state_updates: game_state_updates,
                        number_of_players: number_of_players,
                        winning_player_index: winning_player_index,
                    }));
                }
            }

            // The game is not yet over. Onto the next turn!

            turns_counter += 1;

            current_player_index = next_player_index;
            let current_turn_taker = &mut turn_takers[current_player_index as usize];

            match current_turn_taker.decide_next_game_state(&current_game_state) {
                Ok(new_game_state) => {
                    let new_serialized_game_state = self
                        .game_state_serializer
                        .serialize_game_state(current_player_index, &new_game_state);
                    game_state_updates.push(GameStateUpdate {
                        new_serialized_game_state: new_serialized_game_state,
                        responsible_player_index: current_player_index,
                    });

                    current_game_state = new_game_state;
                }
                Err(_) => return Err(RunGameError::UnableToDecideNextState(current_player_index)),
            }
        }

        if max_number_of_turns > -1
            && turns_counter == max_number_of_turns
            && is_reaching_max_number_of_turns_a_draw
        {
            return Ok(Some(GameReport {
                game_state_updates: game_state_updates,
                number_of_players: number_of_players,
                winning_player_index: -1,
            }));
        }

        return Ok(None);
    }
}

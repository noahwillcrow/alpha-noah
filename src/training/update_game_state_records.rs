use crate::structs::{GameReport, GameStateUpdate};
use crate::traits::{BasicSerializedGameState, GameStateRecordsProvider};
use std::cell::RefCell;
use std::collections::HashSet;

pub fn update_game_state_records<SerializedGameState: BasicSerializedGameState>(
    game_state_records_provider_ref_cell: &RefCell<
        dyn GameStateRecordsProvider<SerializedGameState>,
    >,
    game_report: GameReport<SerializedGameState>,
) {
    let did_draw = game_report.winning_player_index == -1;

    let mut already_updated_game_state_updates: HashSet<GameStateUpdate<SerializedGameState>> =
        HashSet::new();
    for game_state_update in game_report.game_state_updates.iter() {
        if already_updated_game_state_updates.contains(&game_state_update) {
            continue;
        }

        already_updated_game_state_updates.insert(game_state_update.clone());

        game_state_records_provider_ref_cell
            .borrow_mut()
            .update_game_state_record(
                &game_state_update.new_serialized_game_state,
                did_draw,
                game_state_update.responsible_player_index == game_report.winning_player_index,
            )
    }
}

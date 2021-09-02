use crate::training::update_game_state_records;
use crate::traits::{
    BasicGameState, BasicSerializedGameState, GameReportsPersister, GameRunner,
    GameStateRecordsProvider, PendingUpdatesManager, TurnTaker,
};
use std::cell::RefCell;
use std::time::Instant;

pub struct StandardTrainer<
    'a,
    GameState: BasicGameState,
    SerializedGameState: BasicSerializedGameState,
    GameReportsPersisterErrorType,
> {
    base_game_runner: &'a mut dyn GameRunner<GameState, SerializedGameState>,
    game_name: String,
    game_reports_persister_ref_cell:
        &'a RefCell<dyn GameReportsPersister<SerializedGameState, GameReportsPersisterErrorType>>,
    game_state_records_provider_ref_cell:
        &'a RefCell<dyn GameStateRecordsProvider<SerializedGameState>>,
    is_verbose: bool,
    pending_updates_manager_ref_cells: Vec<&'a RefCell<dyn PendingUpdatesManager>>,
}

impl<
        'a,
        GameState: BasicGameState,
        SerializedGameState: BasicSerializedGameState,
        GameReportsPersisterErrorType,
    > StandardTrainer<'a, GameState, SerializedGameState, GameReportsPersisterErrorType>
{
    pub fn new(
        base_game_runner: &'a mut dyn GameRunner<GameState, SerializedGameState>,
        game_name: &str,
        game_reports_persister_ref_cell: &'a RefCell<
            dyn GameReportsPersister<SerializedGameState, GameReportsPersisterErrorType>,
        >,
        game_state_records_provider_ref_cell: &'a RefCell<
            dyn GameStateRecordsProvider<SerializedGameState>,
        >,
        is_verbose: bool,
        pending_updates_manager_ref_cells: Vec<&'a RefCell<dyn PendingUpdatesManager>>,
    ) -> StandardTrainer<'a, GameState, SerializedGameState, GameReportsPersisterErrorType> {
        return StandardTrainer {
            base_game_runner: base_game_runner,
            game_name: String::from(game_name),
            game_reports_persister_ref_cell: game_reports_persister_ref_cell,
            game_state_records_provider_ref_cell: game_state_records_provider_ref_cell,
            is_verbose: is_verbose,
            pending_updates_manager_ref_cells: pending_updates_manager_ref_cells,
        };
    }

    pub fn train(
        &mut self,
        number_of_games: u32,
        create_initial_game_state: fn() -> GameState,
        turn_takers: &mut Vec<&mut dyn TurnTaker<GameState>>,
        max_number_of_turns: i32,
        is_reaching_max_number_of_turns_a_draw: bool,
    ) -> Result<(), GameReportsPersisterErrorType> {
        self.write_line_if_verbose(
            &format!(
                "Starting simulation of {} games of {}.",
                number_of_games, &self.game_name
            )[..],
        );

        let mut draws_count = 0;
        let mut inconclusive_games_count = 0;
        let mut wins_counts_by_player_index = vec![0; 2];
        let mut update_wins_counts = |winning_player_index: i32| {
            if winning_player_index >= 0 {
                wins_counts_by_player_index[winning_player_index as usize] += 1;
            } else if winning_player_index == -1 {
                draws_count += 1;
            }
        };

        let simulations_start_instant = Instant::now();
        for _ in 0..number_of_games {
            let run_game_result = self.base_game_runner.run_game(
                create_initial_game_state(),
                turn_takers,
                max_number_of_turns,
                is_reaching_max_number_of_turns_a_draw,
            );

            match run_game_result {
                Ok(game_report_option) => match game_report_option {
                    Some(game_report) => {
                        update_wins_counts(game_report.winning_player_index);
                        update_game_state_records(
                            self.game_state_records_provider_ref_cell,
                            game_report.clone(),
                        );
                        self.game_reports_persister_ref_cell
                            .borrow_mut()
                            .persist_game_report(game_report)?;
                    }
                    None => inconclusive_games_count += 1,
                },
                Err(_) => (),
            }
        }

        self.write_line_if_verbose(
            &format!(
                "Simulations complete. Duration: {:?}.",
                simulations_start_instant.elapsed()
            )[..],
        );

        self.write_line_if_verbose(
            &format!(
                "Number of games that were inconclusive: {}.",
                inconclusive_games_count
            )[..],
        );
        self.write_line_if_verbose(
            &format!("Number of games that ended in a draw: {}.", draws_count)[..],
        );
        self.write_line_if_verbose(
            &format!(
                "Number of games won by player index: {:#?}.",
                wins_counts_by_player_index
            )[..],
        );

        self.write_line_if_verbose("Waiting for all pending updates to be committed.");
        let pending_updates_start_instant = Instant::now();

        for pending_updates_manager_ref_cell in self.pending_updates_manager_ref_cells.iter() {
            pending_updates_manager_ref_cell
                .borrow_mut()
                .try_commit_pending_updates_in_background(usize::MAX)
                .join()
                .expect("Failed to commit pending updates");
        }

        self.write_line_if_verbose(
            &format!(
                "Pending updates commited. Duration: {:?}.",
                pending_updates_start_instant.elapsed()
            )[..],
        );

        self.write_line_if_verbose("Done training.");
        return Ok(());
    }

    fn write_line_if_verbose(&self, text: &str) {
        if self.is_verbose {
            println!("{}", text);
        }
    }
}

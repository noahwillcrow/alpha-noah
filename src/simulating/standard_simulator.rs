use crate::traits::{
    BasicGameState, BasicSerializedGameState, GameReportsProcessor, GameRunner,
    PendingUpdatesManager, TurnTaker,
};
use chrono::Local;
use std::io::{stdout, Write};
use std::time::Instant;

pub struct StandardSimulator<
    'a,
    GameState: BasicGameState,
    SerializedGameState: BasicSerializedGameState,
    GameReportsPersisterErrorType,
> {
    base_game_runner: &'a mut dyn GameRunner<GameState, SerializedGameState>,
    game_name: String,
    game_reports_processor:
        &'a dyn GameReportsProcessor<SerializedGameState, GameReportsPersisterErrorType>,
    is_verbose: bool,
    pending_updates_managers: &'a Vec<&'a dyn PendingUpdatesManager>,
}

impl<
        'a,
        GameState: BasicGameState,
        SerializedGameState: BasicSerializedGameState,
        GameReportsPersisterErrorType,
    > StandardSimulator<'a, GameState, SerializedGameState, GameReportsPersisterErrorType>
{
    pub fn new(
        base_game_runner: &'a mut dyn GameRunner<GameState, SerializedGameState>,
        game_name: &str,
        game_reports_processor: &'a dyn GameReportsProcessor<
            SerializedGameState,
            GameReportsPersisterErrorType,
        >,
        is_verbose: bool,
        pending_updates_managers: &'a Vec<&'a dyn PendingUpdatesManager>,
    ) -> StandardSimulator<'a, GameState, SerializedGameState, GameReportsPersisterErrorType> {
        return StandardSimulator {
            base_game_runner: base_game_runner,
            game_name: String::from(game_name),
            game_reports_processor: game_reports_processor,
            is_verbose: is_verbose,
            pending_updates_managers: pending_updates_managers,
        };
    }

    pub fn run_simulations<'b>(
        &mut self,
        number_of_games: u32,
        create_initial_game_state: fn() -> GameState,
        create_turn_takers: &mut dyn FnMut() -> Vec<&'b dyn TurnTaker<GameState>>,
        max_number_of_turns: i32,
        is_reaching_max_number_of_turns_a_draw: bool,
    ) -> Result<(), GameReportsPersisterErrorType> {
        self.write_line_if_verbose(
            &format!(
                "Starting simulation of {} games of {}. Initial start date and time is {}.",
                number_of_games,
                &self.game_name,
                Local::now().format("%Y-%m-%d - %H:%M:%S"),
            )[..],
        );

        let mut draws_count = 0;
        let mut inconclusive_games_count = 0;
        let mut wins_counts_by_player_index = vec![0; 2];
        let mut update_result_counts = |winning_player_index: i32| {
            if winning_player_index >= 0 {
                wins_counts_by_player_index[winning_player_index as usize] += 1;
            } else if winning_player_index == -1 {
                draws_count += 1;
            }
        };

        let mut stdout = stdout();

        let mut displayed_progress_percentage: i32 = -1;
        let simulations_start_instant = Instant::now();
        for i in 0..number_of_games {
            let turn_takers = create_turn_takers();
            let run_game_result = self.base_game_runner.run_game(
                create_initial_game_state(),
                &turn_takers,
                max_number_of_turns,
                is_reaching_max_number_of_turns_a_draw,
            );

            match run_game_result {
                Ok(game_report_option) => match game_report_option {
                    Some(game_report) => {
                        update_result_counts(game_report.winning_player_index);
                        self.game_reports_processor
                            .process_game_report(&mut game_report.clone())?;
                    }
                    None => inconclusive_games_count += 1,
                },
                Err(err) => println!("uh? {:?}", err),
            }

            let new_displayed_progress_percentage = (((i + 1) * 100) / number_of_games) as i32;
            if new_displayed_progress_percentage > displayed_progress_percentage {
                displayed_progress_percentage = new_displayed_progress_percentage;
                if self.is_verbose {
                    print!(
                        "\rCompleted {}% of {} simulations",
                        new_displayed_progress_percentage, number_of_games
                    );
                    stdout.flush().unwrap();
                }
            }
        }

        self.write_line_if_verbose("");

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

        for pending_updates_manager in self.pending_updates_managers.iter() {
            pending_updates_manager
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

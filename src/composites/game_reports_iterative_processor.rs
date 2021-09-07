use crate::structs::GameReport;
use crate::traits::{BasicSerializedGameState, GameReportsProcessor};

pub struct GameReportsIterativeProcessor<
    'a,
    SerializedGameState: BasicSerializedGameState,
    ErrorType,
> {
    game_report_processors: Vec<&'a dyn GameReportsProcessor<SerializedGameState, ErrorType>>,
}

impl<'a, SerializedGameState: BasicSerializedGameState, ErrorType>
    GameReportsIterativeProcessor<'a, SerializedGameState, ErrorType>
{
    pub fn new(
        game_report_processors: Vec<&'a dyn GameReportsProcessor<SerializedGameState, ErrorType>>,
    ) -> GameReportsIterativeProcessor<'a, SerializedGameState, ErrorType> {
        return GameReportsIterativeProcessor {
            game_report_processors: game_report_processors,
        };
    }
}

impl<'a, SerializedGameState: BasicSerializedGameState, ErrorType>
    GameReportsProcessor<SerializedGameState, ErrorType>
    for GameReportsIterativeProcessor<'a, SerializedGameState, ErrorType>
{
    fn process_game_report(
        &self,
        game_report: &mut GameReport<SerializedGameState>,
    ) -> Result<(), ErrorType> {
        for game_report_processor in &self.game_report_processors {
            game_report_processor.process_game_report(game_report)?;
        }

        return Ok(());
    }
}

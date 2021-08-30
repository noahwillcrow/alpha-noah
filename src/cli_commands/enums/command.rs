use std::str::FromStr;

#[derive(Debug)]
pub enum Command {
    InteractiveGame,
    SimulateGames,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "interactive-game" => Ok(Command::InteractiveGame),
            "simulate-games" => Ok(Command::SimulateGames),
            _ => Err(()),
        };
    }
}

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

#[derive(Debug)]
pub enum Game {
    Checkers,
    TicTacToe,
}

impl FromStr for Game {
    type Err = ();
    fn from_str(src: &str) -> Result<Game, ()> {
        return match src {
            "checkers" => Ok(Game::Checkers),
            "tic-tac-toe" => Ok(Game::TicTacToe),
            _ => Err(()),
        };
    }
}

use std::str::FromStr;

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

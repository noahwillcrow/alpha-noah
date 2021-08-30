use crate::cli_commands::enums::command::Command;
use argparse;

mod cli_commands;
mod core;
mod games;
mod persistence;
mod weight_calculators;

fn main() -> Result<(), rusqlite::Error> {
    let mut command = Command::SimulateGames;
    let mut args = vec![];

    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Runs alpha-noah");

        ap.refer(&mut command).required().add_argument(
            "command",
            argparse::Store,
            r#"Command to run (either "interactive-game" or "simulate-games")"#,
        );

        ap.refer(&mut args)
            .add_argument("arguments", argparse::List, r#"Arguments for command"#);

        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    args.insert(0, format!("command {:?}", command));
    match command {
        Command::InteractiveGame => cli_commands::interactive_game::interactive_game(args),
        Command::SimulateGames => cli_commands::simulate_games::simulate_games(args),
    }
}

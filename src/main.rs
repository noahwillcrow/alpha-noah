use argparse;

mod cli;
mod composites;
mod constants;
mod enums;
mod game_runners;
mod game_state_records_providers;
mod games;
mod internal;
mod persistence;
mod structs;
mod training;
mod traits;
mod turn_takers;
mod weights_calculators;

fn main() -> Result<(), ()> {
    let mut command = cli::enums::Command::SimulateGames;
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
        cli::enums::Command::InteractiveGame => return cli::interactive_game(args),
        cli::enums::Command::SimulateGames => return cli::simulate_games(args),
    }
}

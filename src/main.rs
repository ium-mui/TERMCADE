use std::io;

use clap::Parser;
use cli_game::app::run_tui;
use cli_game::cli::{Cli, resolve_route};
use cli_game::domain::GameCatalog;
use cli_game::history::JsonHistoryStore;

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let catalog = GameCatalog::default();
    let route = match resolve_route(&cli.path, &catalog) {
        Ok(route) => route,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let history = JsonHistoryStore::default();
    run_tui(catalog, route, Box::new(history))
}

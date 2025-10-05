mod cli;
mod enums;
mod manager;
mod persist;
mod util;

use color_eyre::Result;

fn main() {
    color_eyre::install().unwrap();
    
    if let Err(err) = cli::try_cli() {
        eprintln!("Error: {:?}", err);
        std::process::exit(1);
    }
}



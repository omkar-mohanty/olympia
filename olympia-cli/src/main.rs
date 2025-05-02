use std::path::PathBuf;

use olympia_core::load;
use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    path: PathBuf
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();
    let cli = Cli::parse();
    let _ = load(cli.path.as_path());
}

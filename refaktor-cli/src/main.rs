#![allow(unused)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

use clap::Parser;

#[derive(Parser)]
#[command(name = "refaktor")]
#[command(about = "Smart search & replace for code and files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Plan { old: String, new: String },
}

fn main() {
    let cli = Cli::parse();

    println!("Refaktor CLI v{}", env!("CARGO_PKG_VERSION"));
}

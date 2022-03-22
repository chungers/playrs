
use clap::{Parser, Subcommand};

mod serde;
mod template;
mod watch;


#[derive(Parser)]
#[clap(author = "chunger",
       version,
       about = "This is a simple learning exercise in Rust.",
       long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Prints out verbosely
    #[clap(long, short)]
    verbose: bool,
}


#[derive(Subcommand)]
enum Commands {
    /// Serde examples
    Serde(serde::Args),

    /// Template examples
    Template(template::Args),

    /// Notify (watch files) examples
    Watch(watch::Args),
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Serde(args) => {
            if cli.verbose {
                println!("Serde...");
            }
            serde::serde(args);
        },
        Commands::Template(args) => {
            if cli.verbose {
                println!("Templating...");
            }
            template::template(args);
        },
        Commands::Watch(args) => {
            if cli.verbose {
                println!("Watching...");
            }
            watch::watch(args);
        },
    }
}

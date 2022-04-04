
use clap::{Parser, Subcommand, Args};

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

    /// Stash - nested sub
    Stash(Stash),

    /// Notify (watch files) examples
    Watch(watch::command::Command),
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Stash {
    #[clap(subcommand)]
    command: Option<StashCommands>,

    #[clap(flatten)]
    push: StashPush,
}

#[derive(Debug, Subcommand)]
enum StashCommands {
    Push(StashPush),
    Pop { stash: Option<String> },
    Apply { stash: Option<String> },
}

#[derive(Debug, Args)]
struct StashPush {
    #[clap(short, long)]
    message: Option<String>,
}







fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serde(args) => {
            if cli.verbose {
                println!("Serde...");
            }
            serde::serde(&args);
        },
        Commands::Template(args) => {
            if cli.verbose {
                println!("Templating...");
            }
            template::template(&args);
        },
        Commands::Watch(args) => {
            if cli.verbose {
                println!("Watching...");
            }
            watch::watch(&args);
        },
        Commands::Stash(stash) => {
            let stash_cmd = stash.command.unwrap_or(StashCommands::Push(stash.push));
            match stash_cmd {
                StashCommands::Push(push) => {
                    println!("Pushing {:?}", push);
                }
                StashCommands::Pop { stash } => {
                    println!("Popping {:?}", stash);
                }
                StashCommands::Apply { stash } => {
                    println!("Applying {:?}", stash);
                }
            }
        },
    }
}

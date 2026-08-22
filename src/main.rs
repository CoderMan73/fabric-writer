mod commands;
mod state;

use clap::{Parser, Subcommand};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init {
            name,
            version,
            options,
            dangerous,
            dir,
            java_path,
        } => commands::init::run(name, version, options, dangerous, dir, java_path),
        Commands::Add { subcommand } => match subcommand {
            AddSubcommand::Item(args) => commands::add::run_item(args),
        },
    }
}

#[derive(Parser)]
#[command(name = "fw")]
#[command(about = "Create basic fabric mods efficiently and easily")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        name: String,
        #[arg(long)]
        version: String,
        #[arg(short = 'o', long = "option", action = clap::ArgAction::Append)]
        options: Vec<String>,
        #[arg(long)]
        dangerous: bool,
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        java_path: String,
    },
    Add {
        #[command(subcommand)]
        subcommand: AddSubcommand,
    },
}

#[derive(Subcommand)]
enum AddSubcommand {
    Item(commands::add::ItemAddArgs),
}

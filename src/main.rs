use clap::{Parser, Subcommand};
use fabric_writer::commands::{block, init, item, recipe, run, status};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => init::run(args),
        Commands::Add { subcommand } => match subcommand {
            AddSubcommand::Item(args) => item::add(args),
            AddSubcommand::Block(args) => block::add(args),
            AddSubcommand::Recipe(args) => recipe::add(args),
        },
        Commands::Remove { subcommand } => match subcommand {
            RemoveSubcommand::Item(args) => item::remove(args),
            RemoveSubcommand::Block(args) => block::remove(args),
            RemoveSubcommand::Recipe(args) => recipe::remove(args),
        },
        Commands::Run { subcommand } => match subcommand {
            RunSubcommand::Datagen => run::datagen(),
            RunSubcommand::Client => run::client(),
        },
        Commands::Status(args) => status::run(args),
        // TODO: finish Commands::Doctor => doctor::run(),
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
    Init(init::InitArgs),
    Add {
        #[command(subcommand)]
        subcommand: AddSubcommand,
    },
    Remove {
        #[command(subcommand)]
        subcommand: RemoveSubcommand,
    },
    Run {
        #[command(subcommand)]
        subcommand: RunSubcommand,
    },
    Status(status::StatusArgs),
    // TODO: finish Doctor
}

#[derive(Subcommand)]
enum AddSubcommand {
    Item(item::ItemAddArgs),
    Block(block::BlockAddArgs),
    Recipe(recipe::RecipeAddArgs),
}

#[derive(Subcommand)]
enum RemoveSubcommand {
    Item(item::ItemRemoveArgs),
    Block(block::BlockRemoveArgs),
    Recipe(recipe::RecipeRemoveArgs),
}

#[derive(Subcommand)]
enum RunSubcommand {
    Datagen,
    Client,
}

#![warn(rustdoc::all, missing_docs)]

//! CLI binary for the fabric-writer crate.
//!
//! Most logic lives in the [`fabric_writer`] library crate; this binary
//! provides the clap CLI front-end and command dispatch.

use clap::{Parser, Subcommand};
use fabric_writer::commands::{
    block::{self, BlockAddArgs, BlockRemoveArgs},
    creative_tab::{self, CreativeTabAddArgs, CreativeTabRemoveArgs},
    init::{self, InitArgs},
    item::{self, ItemAddArgs, ItemRemoveArgs},
    recipe::{self, RecipeAddArgs, RecipeRemoveArgs},
    regen::{self, RegenArgs},
    run,
    save_load::{self, SaveLoadArgs},
    status::{self, StatusArgs},
};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if cli.markdown_help {
        clap_markdown::print_help_markdown::<Cli>();
        return Ok(());
    }
    let command = cli
        .command
        .ok_or_else(|| anyhow::anyhow!("A subcommand is required"))?;
    match command {
        Commands::Init(args) => init::run(args),
        Commands::Add { subcommand } => match subcommand {
            AddSubcommand::Item(args) => item::add(args),
            AddSubcommand::Block(args) => block::add(args),
            AddSubcommand::Recipe(args) => recipe::add(args),
            AddSubcommand::CreativeTab(args) => creative_tab::add(args),
        },
        Commands::Remove { subcommand } => match subcommand {
            RemoveSubcommand::Item(args) => item::remove(args),
            RemoveSubcommand::Block(args) => block::remove(args),
            RemoveSubcommand::Recipe(args) => recipe::remove(args),
            RemoveSubcommand::CreativeTab(args) => creative_tab::remove(args),
        },
        Commands::Run { subcommand } => match subcommand {
            RunSubcommand::Datagen => run::datagen(),
            RunSubcommand::Client => run::client(),
            RunSubcommand::Server => run::server(),
        },
        Commands::Status(args) => status::run(args),
        Commands::Regen(args) => regen::run(args),
        Commands::SaveLoad(args) => save_load::run(args),
    }
}

#[derive(Parser)]
#[command(name = "fw")]
#[command(about = "Create basic fabric mods efficiently and easily")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Generate Markdown command reference and print to stdout
    #[arg(long, hide = true)]
    markdown_help: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a fabric-writer project
    Init(InitArgs),

    /// Subcommand for adding things to a project [alias: a]
    #[command(alias = "a")]
    Add {
        #[command(subcommand)]
        subcommand: AddSubcommand,
    },

    /// Subcommand for removing things from a project [alias: r]
    #[command(alias = "r")]
    Remove {
        #[command(subcommand)]
        subcommand: RemoveSubcommand,
    },

    /// Subcommand for running project
    Run {
        #[command(subcommand)]
        subcommand: RunSubcommand,
    },

    /// Prints project status [alias: s]
    #[command(alias = "s")]
    Status(StatusArgs),
    /// Regenerate all Java files from current state [alias: g]
    #[command(alias = "g")]
    Regen(RegenArgs),
    /// Save or load project state [alias: sl]
    #[command(alias = "sl")]
    SaveLoad(SaveLoadArgs),
}

#[derive(Subcommand)]
enum AddSubcommand {
    /// Add an item [alias: i]
    #[command(alias = "i")]
    Item(ItemAddArgs),

    /// Add a block [alias: b]
    #[command(alias = "b")]
    Block(BlockAddArgs),

    /// Add a recipe [alias: r]
    #[command(alias = "r")]
    Recipe(RecipeAddArgs),

    /// Add a creative tab [alias: t]
    #[command(alias = "t")]
    CreativeTab(CreativeTabAddArgs),
}

#[derive(Subcommand)]
enum RemoveSubcommand {
    /// Remove a item [alias: i]
    #[command(alias = "i")]
    Item(ItemRemoveArgs),

    /// Remove a block [alias: b]
    #[command(alias = "b")]
    Block(BlockRemoveArgs),

    /// Remove a recipe [alias: r]
    #[command(alias = "r")]
    Recipe(RecipeRemoveArgs),

    /// Remove a creative tab [alias: t]
    #[command(alias = "t")]
    CreativeTab(CreativeTabRemoveArgs),
}

#[derive(Subcommand)]
enum RunSubcommand {
    /// Run datagen [alias: d]
    #[command(alias = "d")]
    Datagen,

    /// Run client [alias: c]
    #[command(alias = "c")]
    Client,

    /// Run server [alias: s]
    #[command(alias = "s")]
    Server,
}

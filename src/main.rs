use clap::{Parser, Subcommand};

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
        #[arg(short, long)]
        mod_id: String,
    },
    ItemAdd {
        id: String,
        #[arg(long)]
        material: String,
        #[arg(long)]
        damage: i32,
        #[arg(long)]
        durability: i32,
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { mod_id } => println!("Init for {mod_id}"),
        Commands::ItemAdd { id, material, damage, durability } => {
            println!("Add item {id}: {material}, {damage}, {durability}")
        }
    }
}

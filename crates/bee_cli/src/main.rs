// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bee-rust", about = "bee-rust framework CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new bee-rust project
    New { name: String },
    /// Generate a controller or model
    Generate {
        #[command(subcommand)]
        kind: GenerateKind,
    },
    /// Run the development server
    Run {
        #[arg(long)]
        watch: bool,
    },
    /// Run database migrations
    Migrate {
        #[command(subcommand)]
        direction: MigrateDirection,
    },
    /// Package for deployment
    Pack {
        #[arg(long, default_value = "linux/x86_64")]
        target: String,
    },
}

#[derive(Subcommand)]
enum GenerateKind {
    Controller { name: String },
    Model { name: String, #[arg(long)] fields: Option<String> },
}

#[derive(Subcommand)]
enum MigrateDirection {
    Up,
    Down,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name } => println!("Creating new project: {}", name),
        Commands::Generate { kind } => match kind {
            GenerateKind::Controller { name } => println!("Generating controller: {}", name),
            GenerateKind::Model { name, fields } => println!("Generating model: {} with fields: {:?}", name, fields),
        },
        Commands::Run { watch } => println!("Running server (watch: {})", watch),
        Commands::Migrate { direction } => match direction {
            MigrateDirection::Up => println!("Running migrations up"),
            MigrateDirection::Down => println!("Running migrations down"),
        },
        Commands::Pack { target } => println!("Packaging for target: {}", target),
    }
}

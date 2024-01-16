mod client;
use clap::{Parser, Subcommand};
use client::Client;

#[derive(Debug, Parser)]
#[command(name = "dockery")]
#[command(about = "Docker registry v2 cli tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Debug, Subcommand)]
enum Commands {
    #[command()]
    Images,
    #[command()]
    Rmi { tag: String },
}

#[tokio::main]
async fn main() {
    let client = Client::new().await;
    let args = Cli::parse();
    match args.command {
        Commands::Images => match client.images().await {
            Ok(()) => {}
            Err(err) => eprintln!("Error: {}", err),
        },
        Commands::Rmi { tag } => {
            let tag_vec: Vec<&str> = tag.split(":").collect();
            match client.rmi(tag_vec[0], tag_vec[1]).await {
                Ok(()) => {}
                Err(err) => eprintln!("Error: {}", err),
            }
        }
    }
}

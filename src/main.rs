use anyhow::Result;
use clap::{Arg, Command};
use std::process;

mod cli;
mod config;
mod database;
mod error;
mod ui;

use config::Config;
use ui::connection_manager::ConnectionManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize SQLx drivers for the "any" module
    sqlx::any::install_default_drivers();
    
    let matches = Command::new("qgo")
        .version("0.1.0")
        .author("Ishan Ravindu")
        .about("A command-line SQL client written in Rust")
        .arg(
            Arg::new("connection")
                .short('c')
                .long("connection")
                .value_name("NAME")
                .help("Connect to a specific saved connection")
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .help("Display version information")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    if matches.get_flag("version") {
        println!("qgo version {}", env!("CARGO_PKG_VERSION"));
        println!("A command-line SQL client written in Rust");
        return Ok(());
    }

    let config = match Config::load().await {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error loading configuration: {}", err);
            process::exit(1);
        }
    };

    let mut connection_manager = ConnectionManager::new(config);

    if let Some(connection_name) = matches.get_one::<String>("connection") {
        match connection_manager.connect_by_name(connection_name).await {
            Ok(_) => {
                println!("Connected to database '{}'", connection_name);
                cli::run_interactive_session(&mut connection_manager).await?;
            }
            Err(err) => {
                eprintln!("Error connecting to '{}': {}", connection_name, err);
                process::exit(1);
            }
        }
    } else {
        loop {
            match connection_manager.select_or_manage_connection().await {
                Ok(true) => {
                    cli::run_interactive_session(&mut connection_manager).await?;
                    
                    if !ui::prompts::confirm("Do you want to connect to another database?") {
                        println!("Goodbye!");
                        break;
                    }
                }
                Ok(false) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            }
        }
    }

    Ok(())
}

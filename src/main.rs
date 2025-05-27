use anyhow::Context;
use clap::Parser;
use docker_db_container_login::{
    Config, DatabaseConnector, Result,
    cli::{Commands, ConnectArgs},
};
use docker_db_container_login::{get_connection_interactively, get_connection_with_auto_detect};
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = docker_db_container_login::Cli::parse();

    let mut config = Config::load().context("Failed to load config")?;
    match cli.command {
        Commands::Connect(args) => connect_command(args, &config).await?,
        Commands::Add(args) => {
            if args.auto_detect {
                let (alias, connection) = get_connection_with_auto_detect()
                    .await
                    .context("Failed in auto-detect mode input")?;

                config
                    .add_connection(alias.clone(), connection)
                    .context("Failed to add connection config")?;

                println!("Connection config '{}' added", alias);
            } else if args.interactive {
                let (alias, connection) = get_connection_interactively()
                    .await
                    .context("Failed in interactive mode input")?;

                config
                    .add_connection(alias.clone(), connection)
                    .context("Failed to add connection config")?;

                println!("Connection config '{}' added", alias);
            } else {
                let alias = match &args.alias {
                    Some(alias) => alias.clone(),
                    None => {
                        return Err(anyhow::anyhow!("Alias name not specified"));
                    }
                };

                let connection = args
                    .to_connection()
                    .map_err(|e| anyhow::anyhow!("Failed to convert connection info: {}", e))?;

                config
                    .add_connection(alias.clone(), connection)
                    .context("Failed to add connection config")?;

                println!("Connection config '{}' added", alias);
            }
        }
        Commands::Remove(args) => {
            config
                .remove_connection(&args.alias)
                .context("Failed to remove connection config")?;

            println!("Connection config '{}' removed", args.alias);
        }
        Commands::List => {
            let connections = config.list_connections();

            if connections.is_empty() {
                println!("No saved connections");
                return Ok(());
            }

            println!("Connection list:");
            for (alias, conn) in connections {
                let status = if DatabaseConnector::check_container(&conn.container).await? {
                    "Running"
                } else {
                    "Stopped"
                };

                println!(
                    "  {}: {} ({}@{}, DB: {}) [{}]",
                    alias,
                    conn.db_type,
                    conn.user,
                    conn.container,
                    conn.database.as_deref().unwrap_or("-"),
                    status
                );
            }
        }
    }

    Ok(())
}

async fn connect_command(args: ConnectArgs, config: &Config) -> Result<()> {
    let connection = if let Some(alias) = args.alias {
        config.get_connection(&alias)?.clone()
    } else if let Some(connection) = args.to_connection() {
        connection
    } else {
        eprintln!(
            "Error: Please specify an alias or required parameters (container name, DB type, username)"
        );
        process::exit(1);
    };

    if !DatabaseConnector::check_container(&connection.container).await? {
        eprintln!("Error: Container '{}' is not running", connection.container);
        process::exit(1);
    }

    println!(
        "Connecting to {} container '{}'...",
        connection.db_type, connection.container
    );
    DatabaseConnector::connect(&connection).await?;

    Ok(())
}

use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use rpassword::prompt_password;
use std::time::Duration;

use crate::config::{Config, Connection, DatabaseType};
use crate::database::Database;
use crate::error::QgoError;

pub struct ConnectionManager {
    config: Config,
    current_database: Option<Database>,
}

impl ConnectionManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            current_database: None,
        }
    }

    pub async fn select_or_manage_connection(&mut self) -> Result<bool> {
        if self.config.connections.is_empty() {
            println!("{}", style("No database connections found.").yellow());
            self.add_new_connection().await?;
            return Ok(true);
        }

        let mut options = vec!["Add new connection".to_string()];
        options.extend(
            self.config
                .connections
                .iter()
                .map(|conn| conn.display_name()),
        );
        options.push("Manage connections".to_string());
        options.push("Settings".to_string());
        options.push("Exit".to_string());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an option")
            .items(&options)
            .default(1) // Default to first connection if available
            .interact()?;

        match selection {
            0 => {
                // Add new connection
                self.add_new_connection().await?;
                Ok(true)
            }
            n if n > 0 && n <= self.config.connections.len() => {
                // Connect to existing connection
                let connection = self.config.connections[n - 1].clone();
                self.connect_to_database(connection).await?;
                Ok(true)
            }
            n if n == self.config.connections.len() + 1 => {
                // Manage connections
                self.manage_connections().await?;
                Ok(false) // Return to main menu
            }
            n if n == self.config.connections.len() + 2 => {
                // Settings
                self.manage_settings().await?;
                Ok(false) // Return to main menu
            }
            _ => {
                // Exit
                Ok(false)
            }
        }
    }

    pub async fn connect_by_name(&mut self, name: &str) -> Result<()> {
        let connection = self
            .config
            .get_connection_by_name(name)
            .ok_or_else(|| QgoError::ConnectionNotFound(name.to_string()))?
            .clone();

        self.connect_to_database(connection).await
    }

    pub async fn connect_to_database(&mut self, mut connection: Connection) -> Result<()> {
        println!("{}", style(format!("Connecting to {}...", connection.display_name())).cyan());

        // If password is empty, prompt for it
        if connection.password.is_empty() {
            connection.password = prompt_password("Enter password: ")?;
        }

        let timeout = Duration::from_secs(self.config.settings.query_timeout_seconds);
        let database = Database::connect(connection, timeout).await?;

        println!("{}", style("Connected successfully!").green());
        self.current_database = Some(database);
        Ok(())
    }

    async fn add_new_connection(&mut self) -> Result<()> {
        println!("{}", style("Add New Database Connection").bold().blue());
        println!();

        let name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Connection name")
            .interact_text()?;

        let db_types = vec!["MySQL", "PostgreSQL", "SQLite"];
        let db_type_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Database type")
            .items(&db_types)
            .default(0)
            .interact()?;

        let db_type = match db_type_selection {
            0 => DatabaseType::MySQL,
            1 => DatabaseType::PostgreSQL,
            2 => DatabaseType::SQLite,
            _ => unreachable!(),
        };

        let (host, port, username, password, database) = match db_type {
            DatabaseType::SQLite => {
                let database: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Database file path")
                    .interact_text()?;
                
                ("localhost".to_string(), 0, "".to_string(), "".to_string(), database)
            }
            _ => {
                let host: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Host")
                    .default("localhost".to_string())
                    .interact_text()?;

                let port: u16 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Port")
                    .default(match db_type {
                        DatabaseType::MySQL => 3306,
                        DatabaseType::PostgreSQL => 5432,
                        _ => 0,
                    })
                    .interact_text()?;

                let username: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Username")
                    .interact_text()?;

                let database: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Database name")
                    .interact_text()?;

                let test_connection = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Test connection now?")
                    .default(true)
                    .interact()?;

                let password = if test_connection {
                    let password = prompt_password("Password: ")?;
                    
                    // Test the connection
                    let test_conn = Connection::new(
                        name.clone(),
                        db_type.clone(),
                        host.clone(),
                        port,
                        username.clone(),
                        password.clone(),
                        database.clone(),
                    );

                    print!("Testing connection... ");
                    let timeout = Duration::from_secs(self.config.settings.query_timeout_seconds);
                    
                    match Database::test_connection(&test_conn, timeout).await {
                        Ok(_) => {
                            println!("{}", style("✓ Connection successful!").green());
                        }
                        Err(e) => {
                            println!("{}", style(format!("✗ Connection failed: {}", e)).red());
                            
                            let continue_anyway = Confirm::with_theme(&ColorfulTheme::default())
                                .with_prompt("Save connection anyway?")
                                .default(false)
                                .interact()?;
                            
                            if !continue_anyway {
                                return Ok(());
                            }
                        }
                    }
                    
                    password
                } else {
                    "".to_string() // Will prompt when connecting
                };

                (host, port, username, password, database)
            }
        };

        let connection = Connection::new(name, db_type, host, port, username, password, database);
        self.config.add_connection(connection);
        self.config.save().await?;

        println!("{}", style("Connection saved successfully!").green());
        Ok(())
    }

    async fn manage_connections(&mut self) -> Result<()> {
        if self.config.connections.is_empty() {
            println!("{}", style("No connections to manage.").yellow());
            return Ok(());
        }

        loop {
            let mut options = vec!["Back to main menu".to_string()];
            options.extend(
                self.config
                    .connections
                    .iter()
                    .map(|conn| format!("Delete: {}", conn.display_name())),
            );

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Connection Management")
                .items(&options)
                .default(0)
                .interact()?;

            if selection == 0 {
                break; // Back to main menu
            }

            let conn_index = selection - 1;
            let connection = &self.config.connections[conn_index];
            
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Delete connection '{}'?", connection.name))
                .default(false)
                .interact()?;

            if confirm {
                let conn_id = connection.id;
                self.config.remove_connection(&conn_id)?;
                self.config.save().await?;
                println!("{}", style("Connection deleted successfully!").green());
                
                if self.config.connections.is_empty() {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn manage_settings(&mut self) -> Result<()> {
        loop {
            let timeout_option = format!("Query timeout: {} seconds", self.config.settings.query_timeout_seconds);
            let max_rows_option = format!("Max rows display: {:?}", self.config.settings.max_rows_display);
            let auto_completion_option = format!("Auto completion: {}", self.config.settings.auto_completion);
            let history_size_option = format!("History size: {}", self.config.settings.history_size);
            
            let options = vec![
                "Back to main menu",
                &timeout_option,
                &max_rows_option,
                &auto_completion_option,
                &history_size_option,
            ];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Settings")
                .items(&options)
                .default(0)
                .interact()?;

            match selection {
                0 => break, // Back to main menu
                1 => {
                    let timeout: u64 = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Query timeout (seconds)")
                        .default(self.config.settings.query_timeout_seconds)
                        .interact_text()?;
                    self.config.settings.query_timeout_seconds = timeout;
                }
                2 => {
                    let max_rows: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Max rows display (enter 'none' for no limit)")
                        .default(self.config.settings.max_rows_display.map_or_else(|| "none".to_string(), |n| n.to_string()))
                        .interact_text()?;
                    
                    self.config.settings.max_rows_display = if max_rows.to_lowercase() == "none" {
                        None
                    } else {
                        Some(max_rows.parse()?)
                    };
                }
                3 => {
                    self.config.settings.auto_completion = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enable auto completion")
                        .default(self.config.settings.auto_completion)
                        .interact()?;
                }
                4 => {
                    let history_size: usize = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("History size")
                        .default(self.config.settings.history_size)
                        .interact_text()?;
                    self.config.settings.history_size = history_size;
                }
                _ => {}
            }
        }

        self.config.save().await?;
        println!("{}", style("Settings saved successfully!").green());
        Ok(())
    }

    pub fn get_database(&mut self) -> Option<&mut Database> {
        self.current_database.as_mut()
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

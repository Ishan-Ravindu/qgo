use anyhow::Result;
use console::style;
use rustyline::{error::ReadlineError, history::FileHistory, Editor};

use crate::ui::{connection_manager::ConnectionManager, table_display};

pub struct QueryHistory {
    history: Vec<String>,
    current_index: Option<usize>,
}

impl QueryHistory {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_index: None,
        }
    }

    pub fn add(&mut self, query: String) {
        if !query.trim().is_empty() && self.history.last() != Some(&query) {
            self.history.push(query);
            self.current_index = None;
        }
    }

    #[allow(dead_code)]
    pub fn get_all(&self) -> &[String] {
        &self.history
    }

    #[allow(dead_code)]
    pub fn previous(&mut self) -> Option<&String> {
        if self.history.is_empty() {
            return None;
        }

        self.current_index = match self.current_index {
            None => Some(self.history.len() - 1),
            Some(0) => Some(0),
            Some(i) => Some(i - 1),
        };

        self.current_index.and_then(|i| self.history.get(i))
    }

    #[allow(dead_code)]
    pub fn next(&mut self) -> Option<&String> {
        if self.history.is_empty() {
            return None;
        }

        self.current_index = match self.current_index {
            None => None,
            Some(i) if i >= self.history.len() - 1 => None,
            Some(i) => Some(i + 1),
        };

        self.current_index.and_then(|i| self.history.get(i))
    }
}

pub async fn run_interactive_session(connection_manager: &mut ConnectionManager) -> Result<()> {
    let max_rows_display = {
        let config = connection_manager.get_config();
        config.settings.max_rows_display
    };
    
    // Get database after releasing the borrow on connection_manager
    let database = match connection_manager.get_database() {
        Some(db) => db,
        None => {
            println!("{}", style("No database connection available.").red());
            return Ok(());
        }
    };

    let connection_info = database.get_connection().clone();
    println!("{}", style(format!("Connected to {} database.", connection_info.db_type)).green());
    println!("{}", style("Type your SQL queries, 'help' for commands, or 'exit' to quit.").dim());

    let mut history = QueryHistory::new();
    
    // Setup readline editor
    let mut rl = Editor::<(), FileHistory>::new()?;
    let history_file = dirs::config_dir()
        .map(|dir| dir.join("qgo").join("history.txt"))
        .unwrap_or_else(|| std::path::PathBuf::from("qgo_history.txt"));

    if history_file.exists() {
        let _ = rl.load_history(&history_file);
    }

    let prompt = format!("{}@{}:({})> ", 
        connection_info.username, 
        connection_info.host, 
        connection_info.database
    );

    loop {
        match rl.readline(&prompt) {
            Ok(line) => {
                let input = line.trim();
                
                if input.is_empty() {
                    continue;
                }

                rl.add_history_entry(input.to_string())?;
                history.add(input.to_string());

                if let Err(e) = handle_input(input, database, max_rows_display).await {
                    println!("{}", style(format!("Error: {}", e)).red());
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C pressed. Type 'exit' to quit.");
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D pressed. Goodbye!");
                break;
            }
            Err(err) => {
                println!("Error reading input: {}", err);
                break;
            }
        }
    }

    // Save history
    if let Some(parent) = history_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = rl.save_history(&history_file);

    Ok(())
}

async fn handle_input(
    input: &str,
    database: &mut crate::database::Database,
    max_rows_display: Option<usize>,
) -> Result<()> {
    let trimmed = input.trim().to_lowercase();

    match trimmed.as_str() {
        "exit" | "quit" | "\\q" => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        "help" | "\\h" => {
            show_help();
            return Ok(());
        }
        "clear" | "\\c" => {
            table_display::clear_screen();
            return Ok(());
        }
        "version" | "\\v" => {
            println!("qgo version {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        "tables" | "\\dt" => {
            let tables = database.get_tables().await?;
            if tables.is_empty() {
                println!("No tables found.");
            } else {
                println!("Tables:");
                for table in tables {
                    println!("  {}", table);
                }
            }
            return Ok(());
        }
        _ => {}
    }

    // Handle DESCRIBE commands
    if trimmed.starts_with("describe ") || trimmed.starts_with("\\d ") {
        let table_name = if trimmed.starts_with("describe ") {
            &input[9..].trim()
        } else {
            &input[3..].trim()
        };
        
        let columns = database.get_columns(table_name).await?;
        if columns.is_empty() {
            println!("Table '{}' not found or has no columns.", table_name);
        } else {
            println!("Columns in table '{}':", table_name);
            for column in columns {
                println!("  {}", column);
            }
        }
        return Ok(());
    }

    // Handle EXPORT commands
    if trimmed.starts_with("export ") {
        let parts: Vec<&str> = input[7..].splitn(3, ' ').collect();
        if parts.len() == 3 {
            let format = parts[0].to_lowercase();
            let filename = parts[1];
            let query = parts[2];
            
            let result = database.execute_query(query).await?;
            
            match format.as_str() {
                "csv" => {
                    table_display::export_to_csv(&result, filename)?;
                }
                "json" => {
                    table_display::export_to_json(&result, filename)?;
                }
                _ => {
                    println!("Unsupported export format. Use 'csv' or 'json'.");
                }
            }
            return Ok(());
        } else {
            println!("Usage: export <format> <filename> <query>");
            println!("Example: export csv results.csv SELECT * FROM users");
            return Ok(());
        }
    }

    // Execute SQL query
    let result = database.execute_query(input).await?;
    table_display::display_table(&result, max_rows_display);
    
    Ok(())
}

fn show_help() {
    println!("{}", style("Qgo - SQL Client Commands").bold().blue());
    println!();
    println!("{}", style("SQL Commands:").bold());
    println!("  SELECT, SHOW, DESCRIBE, EXPLAIN  - Execute SQL queries");
    println!();
    println!("{}", style("Special Commands:").bold());
    println!("  help, \\h          - Show this help message");
    println!("  exit, quit, \\q    - Exit the program");
    println!("  clear, \\c         - Clear the screen");
    println!("  version, \\v       - Show version information");
    println!("  tables, \\dt       - List all tables");
    println!("  describe <table>, \\d <table> - Describe table structure");
    println!();
    println!("{}", style("Export Commands:").bold());
    println!("  export csv <file> <query>   - Export query results to CSV");
    println!("  export json <file> <query>  - Export query results to JSON");
    println!();
    println!("{}", style("Keyboard Shortcuts:").bold());
    println!("  Ctrl+C            - Cancel current input");
    println!("  Ctrl+D            - Exit program");
    println!("  Up/Down arrows    - Navigate command history");
}

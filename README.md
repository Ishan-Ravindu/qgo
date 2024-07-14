# Qgo

Qgo is a command-line SQL client written in Go. It supports MySQL and PostgreSQL databases and provides an interactive prompt with auto-completion for SQL commands, table names, and column names.

# Features

- Support for MySQL and PostgreSQL
- Interactive CLI with auto-completion
- Multiple database connection management
- Formatted table output for query results

# Installation

1. Clone the repository:
git clone https://github.com/Ishan-Ravindu/qgo.git
cd qgo

2. Build the project:
go build ./cmd/qgo

3. Run Qgo:
./qgo

# Usage

1. On first run, you'll be prompted to add a new database connection.
2. For subsequent runs, you can choose an existing connection or add a new one.
3. Once connected, type your SQL queries at the prompt.
4. Use tab for auto-completion of SQL keywords, table names, and column names.
5. Type 'exit' to quit the program.

# TODO


## Core Functionality
-  Implement a simple query history feature
-  Add support for more database types (e.g., SQLite, Oracle, SQL Server)

## User Interface
-  Improve auto-completion accuracy and performance
-  Implement a simple pager for large result sets

## Configuration and Connection Management
-  Add ability to edit existing database connections
-  Implement basic password masking for security
-  Add a 'test connection' feature when adding/editing connections

## Data Manipulation and Output
-  Add option to export query results to CSV
-  Add ability to limit the number of rows returned

## Performance
-  Add timeout handling for long-running queries

## Documentation
-  Write a basic user guide with examples
-  Create a simple 'help' command within the CLI

## Testing
-  Implement basic unit tests for core functions
-  Add integration tests for database connections

## Usability
-  Add a 'clear screen' command
-  Add a 'version' command to display the current version of Qgo

## Code Quality
-  Add logging for debugging purposes
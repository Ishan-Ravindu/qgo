package cli

import (
	"database/sql"
	"fmt"
	"os"
	"strings"

	"github.com/Ishan-Ravindu/qgo/internal/config"
	"github.com/Ishan-Ravindu/qgo/internal/database"

	"github.com/c-bata/go-prompt"
	"github.com/olekukonko/tablewriter"
)

func RunPrompt(db *sql.DB, currentConnection config.Connection) {
	tables, err := database.FetchTables(db, currentConnection.Type)
	if err != nil {
		fmt.Println("Error fetching tables:", err)
		return
	}

	columns := make(map[string][]string)
	for _, table := range tables {
		cols, err := database.FetchColumns(db, currentConnection.Type, table)
		if err != nil {
			fmt.Printf("Error fetching columns for table %s: %v\n", table, err)
			continue
		}
		columns[table] = cols
	}

	p := prompt.New(
		func(input string) {
			executor(db, input)
		},
		func(d prompt.Document) []prompt.Suggest {
			return completer(d, tables, columns)
		},
		prompt.OptionPrefix(fmt.Sprintf("%s@%s:(%s)-> ",
			currentConnection.User,
			currentConnection.Host,
			currentConnection.Database)),
		prompt.OptionTitle("Qgo CLI"),
	)
	p.Run()
}

func executor(db *sql.DB, input string) {
	input = strings.TrimSpace(input)
	if input == "exit" {
		fmt.Println("Goodbye!")
		os.Exit(0)
	}

	rows, err := db.Query(input)
	if err != nil {
		fmt.Println("Error executing query:", err)
		return
	}
	defer rows.Close()

	cols, err := rows.Columns()
	if err != nil {
		fmt.Println("Error getting columns:", err)
		return
	}

	table := tablewriter.NewWriter(os.Stdout)
	table.SetHeader(cols)

	rawResult := make([][]byte, len(cols))
	dest := make([]interface{}, len(cols))
	for i := range rawResult {
		dest[i] = &rawResult[i]
	}

	for rows.Next() {
		err = rows.Scan(dest...)
		if err != nil {
			fmt.Println("Error scanning row:", err)
			return
		}

		row := make([]string, len(cols))
		for i, raw := range rawResult {
			if raw == nil {
				row[i] = "NULL"
			} else {
				row[i] = string(raw)
			}
		}
		table.Append(row)
	}

	table.Render()
}

func completer(d prompt.Document, tables []string, columns map[string][]string) []prompt.Suggest {
	suggestions := []prompt.Suggest{
		{Text: "SELECT", Description: "Retrieve data from the database"},
		{Text: "INSERT", Description: "Insert new data into a table"},
		{Text: "UPDATE", Description: "Modify existing data in a table"},
		{Text: "DELETE", Description: "Remove data from a table"},
		{Text: "FROM", Description: "Specify the table to query"},
		{Text: "WHERE", Description: "Filter the results"},
		{Text: "ORDER BY", Description: "Sort the results"},
		{Text: "GROUP BY", Description: "Group the results"},
		{Text: "HAVING", Description: "Filter grouped results"},
		{Text: "JOIN", Description: "Combine rows from two or more tables"},
	}

	for _, table := range tables {
		suggestions = append(suggestions, prompt.Suggest{Text: table, Description: "Table"})
	}

	for table, cols := range columns {
		for _, col := range cols {
			suggestions = append(suggestions, prompt.Suggest{Text: col, Description: fmt.Sprintf("Column in %s", table)})
		}
	}

	return prompt.FilterHasPrefix(suggestions, d.GetWordBeforeCursor(), true)
}

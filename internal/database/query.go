package database

import (
	"database/sql"
	"fmt"
)

func FetchTables(db *sql.DB, dbType string) ([]string, error) {
	var query string
	switch dbType {
	case "mysql":
		query = "SHOW TABLES"
	case "postgresql":
		query = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'"
	default:
		return nil, fmt.Errorf("unsupported database type: %s", dbType)
	}

	rows, err := db.Query(query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var tables []string
	for rows.Next() {
		var table string
		err := rows.Scan(&table)
		if err != nil {
			return nil, err
		}
		tables = append(tables, table)
	}

	return tables, nil
}

func FetchColumns(db *sql.DB, dbType, table string) ([]string, error) {
	var query string
	switch dbType {
	case "mysql":
		query = fmt.Sprintf("SHOW COLUMNS FROM %s", table)
	case "postgresql":
		query = fmt.Sprintf("SELECT column_name FROM information_schema.columns WHERE table_name = '%s'", table)
	default:
		return nil, fmt.Errorf("unsupported database type: %s", dbType)
	}

	rows, err := db.Query(query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var columns []string
	for rows.Next() {
		var column string
		var dummy sql.NullString
		if dbType == "mysql" {
			err = rows.Scan(&column, &dummy, &dummy, &dummy, &dummy, &dummy)
		} else {
			err = rows.Scan(&column)
		}
		if err != nil {
			return nil, err
		}
		columns = append(columns, column)
	}

	return columns, nil
}
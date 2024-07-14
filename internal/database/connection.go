package database

import (
	"database/sql"
	"fmt"

	"github.com/Ishan-Ravindu/qgo/internal/config"

	_ "github.com/go-sql-driver/mysql"
	_ "github.com/lib/pq"
)

func Connect(conn config.Connection) (*sql.DB, error) {
	var db *sql.DB
	var err error

	switch conn.Type {
	case "mysql":
		dsn := fmt.Sprintf("%s:%s@tcp(%s:%s)/%s", conn.User, conn.Password, conn.Host, conn.Port, conn.Database)
		db, err = sql.Open("mysql", dsn)
	case "postgresql":
		dsn := fmt.Sprintf("host=%s port=%s user=%s password=%s dbname=%s sslmode=disable",
			conn.Host, conn.Port, conn.User, conn.Password, conn.Database)
		db, err = sql.Open("postgres", dsn)
	default:
		return nil, fmt.Errorf("unsupported database type: %s", conn.Type)
	}

	if err != nil {
		return nil, err
	}

	err = db.Ping()
	if err != nil {
		return nil, err
	}

	return db, nil
}
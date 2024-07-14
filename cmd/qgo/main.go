package main

import (
	"fmt"
	"os"

	"github.com/Ishan-Ravindu/qgo/internal/cli"
	"github.com/Ishan-Ravindu/qgo/internal/config"
	"github.com/Ishan-Ravindu/qgo/internal/database"
)

func main() {
	cfg, err := config.LoadConfig()
	if err != nil {
		fmt.Println("Error loading config:", err)
		os.Exit(1)
	}

	if len(cfg.Connections) == 0 {
		cfg, err = config.AddNewConnection(cfg)
		if err != nil {
			fmt.Println("Error adding new connection:", err)
			os.Exit(1)
		}
	} else {
		fmt.Println("Choose a connection or add a new one:")
		for i, conn := range cfg.Connections {
			fmt.Printf("%d. %s (%s)\n", i+1, conn.Name, conn.Type)
		}
		fmt.Printf("%d. Add new connection\n", len(cfg.Connections)+1)

		var choice int
		fmt.Scan(&choice)

		if choice == len(cfg.Connections)+1 {
			cfg, err = config.AddNewConnection(cfg)
			if err != nil {
				fmt.Println("Error adding new connection:", err)
				os.Exit(1)
			}
		} else if choice > 0 && choice <= len(cfg.Connections) {
			cfg.CurrentConnection = cfg.Connections[choice-1]
		} else {
			fmt.Println("Invalid choice")
			os.Exit(1)
		}
	}

	db, err := database.Connect(cfg.CurrentConnection)
	if err != nil {
		fmt.Println("Error connecting to database:", err)
		os.Exit(1)
	}
	defer db.Close()

	fmt.Printf("Connected to %s database. Type your SQL queries or 'exit' to quit.\n", cfg.CurrentConnection.Type)

	cli.RunPrompt(db, cfg.CurrentConnection.Type)
}
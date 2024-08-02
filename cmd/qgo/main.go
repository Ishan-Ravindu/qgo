package main

import (
	"fmt"
	"os"

	"github.com/Ishan-Ravindu/qgo/internal/cli"
	"github.com/Ishan-Ravindu/qgo/internal/config"
	"github.com/Ishan-Ravindu/qgo/internal/database"
	"github.com/Ishan-Ravindu/qgo/pkg/dropdown"
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
		options := make([]dropdown.Option, len(cfg.Connections)+1)
		for i, conn := range cfg.Connections {
			options[i] = dropdown.Option{
				Value: fmt.Sprintf("%d", i),
				Label: fmt.Sprintf("%s (%s)", conn.Name, conn.Type),
			}
		}
		options[len(cfg.Connections)] = dropdown.Option{
			Value: fmt.Sprintf("%d", len(cfg.Connections)),
			Label: "Add new connection",
		}

		selected, err := dropdown.Select("Choose a connection or add a new one:", options)
		if err != nil {
			fmt.Println("Error selecting connection:", err)
			os.Exit(1)
		}

		choice := -1
		fmt.Sscan(selected, &choice)

		if choice == len(cfg.Connections) {
			cfg, err = config.AddNewConnection(cfg)
			if err != nil {
				fmt.Println("Error adding new connection:", err)
				os.Exit(1)
			}
		} else if choice >= 0 && choice < len(cfg.Connections) {
			cfg.CurrentConnection = cfg.Connections[choice]
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

	fmt.Printf("Connected to %s database. Type your SQL queries or '/exit' to quit.\n", cfg.CurrentConnection.Type)
	cli.RunPrompt(db, cfg.CurrentConnection)
}

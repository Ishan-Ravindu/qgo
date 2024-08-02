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
	for {
		cfg, err := initializeConfig()
		if err != nil {
			handleError("Error initializing config", err)
		}

		db, err := database.Connect(cfg.CurrentConnection)
		if err != nil {
			handleError("Error connecting to database", err)
		}

		fmt.Printf("Connected to %s database. Type your SQL queries or '/exit' to quit.\n", cfg.CurrentConnection.Type)

		cli.RunPrompt(db, cfg.CurrentConnection)

		err = db.Close()
		if err != nil {
			fmt.Printf("Error closing database connection: %v\n", err)
		}

		fmt.Print("Do you want to connect to another database? (y/N): ")
		var answer string
		fmt.Scanln(&answer)
		if answer != "y" && answer != "Y" {
			fmt.Println("Exiting program.")
			break
		}
	}
}

func initializeConfig() (config.Config, error) {
	cfg, err := config.LoadConfig()
	if err != nil {
		return config.Config{}, fmt.Errorf("loading config: %w", err)
	}

	return selectOrAddConnection(cfg)
}

func addNewConnection(cfg config.Config) (config.Config, error) {
	newCfg, err := config.AddNewConnection(cfg)
	if err != nil {
		return config.Config{}, fmt.Errorf("adding new connection: %w", err)
	}
	return newCfg, nil
}

func selectOrAddConnection(cfg config.Config) (config.Config, error) {
	for {
		options := createConnectionOptions(cfg)
		selected, err := dropdown.Select("Choose a connection or action:", options)
		if err != nil {
			return config.Config{}, fmt.Errorf("selecting connection: %w", err)
		}

		choice := -1
		fmt.Sscan(selected, &choice)

		if choice == len(cfg.Connections) {
			newCfg, err := addNewConnection(cfg)
			if err != nil {
				fmt.Printf("Error adding new connection: %v\n", err)
				continue
			}
			return newCfg, nil
		}

		if len(cfg.Connections) > 0 && choice == len(cfg.Connections)+1 {
			newCfg, err := deleteConnection(cfg)
			if err != nil {
				fmt.Printf("Error deleting connection: %v\n", err)
			}
			cfg = newCfg
			continue
		}

		if choice >= 0 && choice < len(cfg.Connections) {
			cfg.CurrentConnection = cfg.Connections[choice]
			return cfg, nil
		}

		fmt.Println("Invalid choice, please try again.")
	}
}

func createConnectionOptions(cfg config.Config) []dropdown.Option {
	optionsCount := len(cfg.Connections) + 1
	if len(cfg.Connections) > 0 {
		optionsCount++
	}

	options := make([]dropdown.Option, optionsCount)

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

	if len(cfg.Connections) > 0 {
		options[len(cfg.Connections)+1] = dropdown.Option{
			Value: fmt.Sprintf("%d", len(cfg.Connections)+1),
			Label: "Delete a connection",
		}
	}

	return options
}

func deleteConnection(cfg config.Config) (config.Config, error) {
	if len(cfg.Connections) == 0 {
		return cfg, fmt.Errorf("no connections to delete")
	}

	options := make([]dropdown.Option, len(cfg.Connections))
	for i, conn := range cfg.Connections {
		options[i] = dropdown.Option{
			Value: fmt.Sprintf("%d", i),
			Label: fmt.Sprintf("%s (%s)", conn.Name, conn.Type),
		}
	}

	selected, err := dropdown.Select("Choose a connection to delete:", options)
	if err != nil {
		return cfg, fmt.Errorf("selecting connection to delete: %w", err)
	}

	choice := -1
	fmt.Sscan(selected, &choice)

	if choice < 0 || choice >= len(cfg.Connections) {
		return cfg, fmt.Errorf("invalid choice")
	}

	deletedConn := cfg.Connections[choice]
	cfg.Connections = append(cfg.Connections[:choice], cfg.Connections[choice+1:]...)

	if cfg.CurrentConnection.Name == deletedConn.Name {
		cfg.CurrentConnection = config.Connection{}
	}

	err = config.SaveConfig(cfg)
	if err != nil {
		return cfg, fmt.Errorf("saving config after deletion: %w", err)
	}

	fmt.Printf("Connection '%s' deleted successfully.\n", deletedConn.Name)

	return cfg, nil
}

func handleError(message string, err error) {
	fmt.Printf("%s: %v\n", message, err)
	os.Exit(1)
}

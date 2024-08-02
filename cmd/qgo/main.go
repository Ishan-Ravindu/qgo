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
	cfg, err := initializeConfig()
	if err != nil {
		handleError("Error initializing config", err)
	}

	db, err := database.Connect(cfg.CurrentConnection)
	if err != nil {
		handleError("Error connecting to database", err)
	}
	defer db.Close()

	fmt.Printf("Connected to %s database. Type your SQL queries or '/exit' to quit.\n", cfg.CurrentConnection.Type)
	cli.RunPrompt(db, cfg.CurrentConnection)
}

func initializeConfig() (config.Config, error) {
	cfg, err := config.LoadConfig()
	if err != nil {
		return config.Config{}, fmt.Errorf("loading config: %w", err)
	}

	if len(cfg.Connections) == 0 {
		return addNewConnection(cfg)
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
	options := createConnectionOptions(cfg)
	selected, err := dropdown.Select("Choose a connection or add a new one:", options)
	if err != nil {
		return config.Config{}, fmt.Errorf("selecting connection: %w", err)
	}

	choice := -1
	fmt.Sscan(selected, &choice)

	if choice == len(cfg.Connections) {
		return addNewConnection(cfg)
	}

	if choice >= 0 && choice < len(cfg.Connections) {
		cfg.CurrentConnection = cfg.Connections[choice]
		return cfg, nil
	}

	return config.Config{}, fmt.Errorf("invalid choice")
}

func createConnectionOptions(cfg config.Config) []dropdown.Option {
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
	return options
}

func handleError(message string, err error) {
	fmt.Printf("%s: %v\n", message, err)
	os.Exit(1)
}

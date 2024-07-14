package config

import (
	"encoding/json"
	"fmt"
	"os"
	
	"github.com/Ishan-Ravindu/qgo/pkg/utils"
)

type Connection struct {
	Name     string
	Type     string
	Host     string
	Port     string
	User     string
	Password string
	Database string
}

type Config struct {
	Connections       []Connection
	CurrentConnection Connection
}

func LoadConfig() (Config, error) {
	configPath := utils.GetConfigPath()
	file, err := os.Open(configPath)
	if os.IsNotExist(err) {
		return Config{}, nil
	} else if err != nil {
		return Config{}, err
	}
	defer file.Close()

	var cfg Config
	err = json.NewDecoder(file).Decode(&cfg)
	return cfg, err
}

func SaveConfig(cfg Config) error {
	configPath := utils.GetConfigPath()
	file, err := os.Create(configPath)
	if err != nil {
		return err
	}
	defer file.Close()

	return json.NewEncoder(file).Encode(cfg)
}

func AddNewConnection(cfg Config) (Config, error) {
	var conn Connection
	fmt.Print("Enter connection name: ")
	fmt.Scan(&conn.Name)
	fmt.Print("Enter database type (mysql/postgresql): ")
	fmt.Scan(&conn.Type)
	fmt.Print("Enter host: ")
	fmt.Scan(&conn.Host)
	fmt.Print("Enter port: ")
	fmt.Scan(&conn.Port)
	fmt.Print("Enter username: ")
	fmt.Scan(&conn.User)
	fmt.Print("Enter password: ")
	fmt.Scan(&conn.Password)
	fmt.Print("Enter database name: ")
	fmt.Scan(&conn.Database)

	cfg.Connections = append(cfg.Connections, conn)
	cfg.CurrentConnection = conn

	err := SaveConfig(cfg)
	if err != nil {
		return cfg, err
	}

	return cfg, nil
}
package utils

import (
	"os"
	"path/filepath"
)

func GetConfigPath() string {
	homeDir, err := os.UserHomeDir()
	if err != nil {
		panic(err)
	}
	return filepath.Join(homeDir, ".qgo_config.json")
}
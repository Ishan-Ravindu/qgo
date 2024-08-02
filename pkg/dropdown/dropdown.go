package dropdown

import (
	"fmt"

	"github.com/eiannone/keyboard"
)

type Option struct {
	Value string
	Label string
}

func Select(prompt string, options []Option) (string, error) {
	if len(options) == 0 {
		return "", fmt.Errorf("no options provided")
	}

	selectedIndex := 0

	err := keyboard.Open()
	if err != nil {
		return "", err
	}
	defer keyboard.Close()

	for {
		// Clear the screen
		fmt.Print("\033[2J")
		fmt.Print("\033[H")

		// Print the prompt
		fmt.Println(prompt)
		fmt.Println()

		// Print options
		for i, option := range options {
			prefix := "  "
			if i == selectedIndex {
				prefix = "> "
			}
			fmt.Printf("%s%s\n", prefix, option.Label)
		}

		// Get key press
		char, key, err := keyboard.GetKey()
		if err != nil {
			return "", err
		}

		switch key {
		case keyboard.KeyArrowUp:
			selectedIndex = (selectedIndex - 1 + len(options)) % len(options)
		case keyboard.KeyArrowDown:
			selectedIndex = (selectedIndex + 1) % len(options)
		case keyboard.KeyEnter:
			return options[selectedIndex].Value, nil
		case keyboard.KeyEsc:
			return "", fmt.Errorf("selection cancelled")
		}

		if char == 'q' || char == 'Q' {
			return "", fmt.Errorf("selection cancelled")
		}
	}
}

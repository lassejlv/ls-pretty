package main

import (
	"fmt"
	"os"
	"strings"
)

func main() {
	// ===========================================
	// Go Autocomplete Test File for ls-pretty
	// ===========================================

	// Try the following autocomplete features:
	// 1. Type "fmt." and autocomplete will show up automatically
	// 2. Type "strings." to see string function completions
	// 3. Use Ctrl+Space to manually trigger autocomplete
	// 4. Type 'k' and 'j' keys - they should work normally in edit mode

	name := "World"

	// Test fmt package autocomplete - type "fmt." after this line:
	fmt.Println("Hello, " + name + "!")

	// Test automatic triggering - add "fmt." here and watch autocomplete appear:

	// More examples for autocomplete testing
	message := fmt.Sprintf("Welcome to %s", "ls-pretty")
	fmt.Printf("Message: %s\n", message)

	// Test strings package autocomplete - type "strings." after this line:
	text := "Go Language Server Protocol"
	lower := strings.ToLower(text)
	upper := strings.ToUpper(text)
	contains := strings.Contains(text, "Server")

	fmt.Println("Original:", text)
	fmt.Println("Lower:", lower)
	fmt.Println("Upper:", upper)
	fmt.Println("Contains 'Server':", contains)

	// Test keyword autocomplete - type "f" and see "func" suggestion:

	// Test conditional autocomplete - type "i" and see "if" suggestion:

	// File operations
	if len(os.Args) > 1 {
		filename := os.Args[1]
		fmt.Printf("Processing file: %s\n", filename)
	}

	// Function call for testing autocomplete
	result := calculateSum(10, 20)
	fmt.Printf("Sum: %d\n", result)

	// Test typing 'k' and 'j' keys in edit mode:
	keyboard := "test kjkjkj keys work"
	fmt.Println(keyboard)
}

func calculateSum(a, b int) int {
	return a + b
}

func processData(data []string) {
	// Test for loop autocomplete - type "fo" and see "for" suggestion:

	for i, item := range data {
		fmt.Printf("Item %d: %s\n", i, item)
	}
}

// Instructions for testing:
// 1. Open this file in ls-pretty
// 2. Press Enter to view, then Ctrl+E to edit
// 3. Look for 🟢 LSP status in header
// 4. Try typing after the empty comment lines above
// 5. Autocomplete should appear automatically as you type
// 6. Test that 'k' and 'j' keys work normally for typing

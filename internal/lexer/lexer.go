// Package lexer provides lexical analysis for command-line arguments.
//
// It tokenizes raw CLI arguments into structured tokens (commands, tags, attributes, IDs, text)
// based on pattern matching rules.
package lexer

import (
	"regexp"
)

var (
	// attributeRegex matches key:value patterns (e.g., "project:home", "due:tomorrow").
	attributeRegex = regexp.MustCompile(`^[a-zA-Z0-9_]+:`)
	// tagRegex matches tags starting with '+' or '-' (e.g., "+work", "-blocked").
	tagRegex = regexp.MustCompile(`^\+|^-`)
	// idRegex matches numeric IDs (e.g., "1", "42").
	idRegex = regexp.MustCompile(`^[0-9]+$`)
)

// knownCommands is the set of valid command strings.
var knownCommands = map[string]bool{
	"add":    true,
	"list":   true,
	"done":   true,
	"modify": true,
	"delete": true,
	"undo":   true,
}

// Parse tokenizes the raw command‑line arguments into a slice of Tokens.
func Parse(args []string) []Token {
	tokens := make([]Token, 0, len(args))

	for _, arg := range args {
		token := classify(arg)
		tokens = append(tokens, token)
	}

	return tokens
}

// classify determines the TokenType of a single argument string.
func classify(arg string) Token {
	// 1. Check if it's a known command
	if knownCommands[arg] {
		return Token{Type: TokenCommand, Raw: arg}
	}

	// 2. Check attribute pattern (must contain colon at the end of the first word)
	if attributeRegex.MatchString(arg) {
		return Token{Type: TokenAttribute, Raw: arg}
	}

	// 3. Check tag pattern
	if tagRegex.MatchString(arg) {
		return Token{Type: TokenTag, Raw: arg}
	}

	// 4. Check numeric ID
	if idRegex.MatchString(arg) {
		return Token{Type: TokenID, Raw: arg}
	}

	// 5. Everything else is plain text
	return Token{Type: TokenText, Raw: arg}
}
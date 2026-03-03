// Package router routes parsed tokens to the appropriate command.
//
// It identifies the command token in the input, separates filter tokens (before the command)
// from modification tokens (after the command), and instantiates the corresponding command handler.
package router

import (
	"digital-receipt-task/internal/commands"
	"digital-receipt-task/internal/lexer"
	"fmt"
)

// RouteResult holds the command and separated token groups.
type RouteResult struct {
	Command          commands.Command
	FilterTokens     []lexer.Token
	ModificationTokens []lexer.Token
}

// Route examines the token slice and returns the command to execute
// along with separated filter tokens (before command) and modification tokens (after command).
// If no command token is present, the default command is "list".
func Route(tokens []lexer.Token) (commands.Command, []lexer.Token, error) {
	result, _, err := RouteWithSeparatedTokens(tokens)
	if err != nil {
		return nil, nil, err
	}
	// For backward compatibility, combine filter and modification tokens
	allTokens := append(result.FilterTokens, result.ModificationTokens...)
	return result.Command, allTokens, nil
}

// RouteWithSeparatedTokens returns the command with filter and modification tokens separated.
func RouteWithSeparatedTokens(tokens []lexer.Token) (*RouteResult, []lexer.Token, error) {
	// Find the first command token
	var cmdIndex = -1
	var cmdName string
	for i, tok := range tokens {
		if tok.Type == lexer.TokenCommand {
			cmdIndex = i
			cmdName = tok.Raw
			break
		}
	}

	var filterTokens []lexer.Token
	var modTokens []lexer.Token

	if cmdIndex == -1 {
		// No command token → default to "list", all tokens are filter tokens
		cmdName = "list"
		filterTokens = tokens
		modTokens = []lexer.Token{}
	} else {
		// Tokens before command are filter tokens
		filterTokens = tokens[:cmdIndex]
		// Tokens after command are modification tokens
		modTokens = tokens[cmdIndex+1:]
	}

	factory, ok := commandFactories[cmdName]
	if !ok {
		return nil, nil, fmt.Errorf("unknown command: %q", cmdName)
	}

	result := &RouteResult{
		Command:          factory(),
		FilterTokens:     filterTokens,
		ModificationTokens: modTokens,
	}
	return result, filterTokens, nil
}

// commandFactories maps command names to factory functions that create command instances.
var commandFactories = map[string]func() commands.Command{
	"add":    func() commands.Command { return &commands.AddCommand{} },
	"list":   func() commands.Command { return &commands.ListCommand{} },
	"done":   func() commands.Command { return &commands.DoneCommand{} },
	"modify": func() commands.Command { return &commands.ModifyCommand{} },
	"delete": func() commands.Command { return &commands.DeleteCommand{} },
	"undo":   func() commands.Command { return &commands.UndoCommand{} },
}
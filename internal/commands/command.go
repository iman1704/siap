// Package commands implements the command pattern for task operations.
package commands

import (
	"digital-receipt-task/internal/context"
	"digital-receipt-task/internal/domain"
	"digital-receipt-task/internal/lexer"
	"digital-receipt-task/internal/storage"
)

// CommandContext holds the separated token groups for command execution.
type CommandContext struct {
	// FilterTokens are tokens before the command (used for filtering targets)
	FilterTokens []lexer.Token
	// ModificationTokens are tokens after the command (used for modifications)
	ModificationTokens []lexer.Token
}

// Command is the interface that all concrete commands must implement.
type Command interface {
	// Execute performs the command's action using the provided context and tokens.
	// It returns an error if the command cannot be completed.
	Execute(ctx *context.Context, cmdCtx *CommandContext) error
}

// loadTasks loads all tasks from the data file specified in the context.
func loadTasks(ctx *context.Context) ([]*domain.Task, error) {
	return storage.LoadTasks(ctx.DataFile)
}

// saveTasks writes the given tasks to the data file.
func saveTasks(ctx *context.Context, tasks []*domain.Task) error {
	return storage.SaveTasks(ctx.DataFile, tasks)
}
package commands

import (
	"digital-receipt-task/internal/context"
	"digital-receipt-task/internal/domain"
	"digital-receipt-task/internal/lexer"
	"digital-receipt-task/internal/storage"
	"fmt"
	"strings"
	"time"

	"github.com/tj/go-naturaldate"
)

// AddCommand creates a new task.
type AddCommand struct{}

// Execute creates a new task from the provided tokens and saves it.
func (c *AddCommand) Execute(ctx *context.Context, cmdCtx *CommandContext) error {
	task, err := buildTaskFromTokens(cmdCtx.ModificationTokens)
	if err != nil {
		return fmt.Errorf("failed to build task: %w", err)
	}

	// Append create operation
	op := storage.Operation{
		Type:      storage.OpCreate,
		TaskID:    task.UUID,
		Task:      task,
		Timestamp: time.Now(),
	}
	if err := storage.AppendOperation(ctx.DataFile, op); err != nil {
		return fmt.Errorf("failed to append operation: %w", err)
	}

	// Reload tasks to get the new ephemeral ID
	tasks, err := loadTasks(ctx)
	if err != nil {
		return fmt.Errorf("failed to load tasks: %w", err)
	}

	// Find the task we just added to get its ID
	for _, t := range tasks {
		if t.UUID == task.UUID {
			fmt.Fprintf(ctx.Out, "Added task %d: %s\n", t.ID, task.Description)
			break
		}
	}
	return nil
}

// buildTaskFromTokens constructs a domain.Task from a slice of tokens.
func buildTaskFromTokens(tokens []lexer.Token) (*domain.Task, error) {
	var descriptionParts []string
	task := domain.NewTask("")

	for _, tok := range tokens {
		switch tok.Type {
		case lexer.TokenText:
			descriptionParts = append(descriptionParts, tok.Raw)

		case lexer.TokenTag:
			// +tag adds, -tag removes
			if strings.HasPrefix(tok.Raw, "+") {
				tag := strings.TrimPrefix(tok.Raw, "+")
				task.AddTag(tag)
			} else if strings.HasPrefix(tok.Raw, "-") {
				tag := strings.TrimPrefix(tok.Raw, "-")
				task.RemoveTag(tag)
			}

		case lexer.TokenAttribute:
			key, val, found := strings.Cut(tok.Raw, ":")
			if !found {
				// Should not happen per lexer regex
				continue
			}
			switch key {
			case "project":
				task.Project = val
			case "due":
				if due, err := parseDueDate(val); err == nil {
					task.DueDate = &due
				} else {
					// Store raw value as custom attribute
					task.SetAttribute(key, val)
				}
			default:
				task.SetAttribute(key, val)
			}

		case lexer.TokenID, lexer.TokenCommand:
			// IDs and commands are not expected in modifications for add.
			// Ignore them silently.
			continue
		}
	}

	if len(descriptionParts) == 0 {
		return nil, fmt.Errorf("task description cannot be empty")
	}
	task.Description = strings.Join(descriptionParts, " ")
	return task, nil
}

// parseDueDate attempts to parse a due date string using natural language.
// Supports formats like "tomorrow", "next week", "friday", "2025-12-31", etc.
func parseDueDate(s string) (time.Time, error) {
	// Try natural language parsing first
	if due, err := naturaldate.Parse(s, time.Now()); err == nil {
		return due, nil
	}

	// Fallback to ISO 8601 format (YYYY-MM-DD)
	return time.Parse("2006-01-02", s)
}
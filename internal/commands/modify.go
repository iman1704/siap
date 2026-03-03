package commands

import (
	"digital-receipt-task/internal/context"
	"digital-receipt-task/internal/domain"
	"digital-receipt-task/internal/filter"
	"digital-receipt-task/internal/lexer"
	"digital-receipt-task/internal/storage"
	"fmt"
	"strings"
	"time"
)

// ModifyCommand modifies existing tasks.
type ModifyCommand struct{}

// Execute finds tasks by ID filter and applies modifications.
func (c *ModifyCommand) Execute(ctx *context.Context, cmdCtx *CommandContext) error {
	tasks, err := loadTasks(ctx)
	if err != nil {
		return err
	}

	// Extract filter criteria from filter tokens
	filterCriteria, _ := filter.ExtractFilterTokens(cmdCtx.FilterTokens)

	// Find target tasks by ID
	targetTasks := filter.ApplyByID(tasks, filterCriteria)
	if len(targetTasks) == 0 {
		return fmt.Errorf("no tasks found matching the filter criteria")
	}

	// Apply modifications from modification tokens (tokens after the command)
	for _, task := range targetTasks {
		if err := applyModifications(task, cmdCtx.ModificationTokens); err != nil {
			return err
		}
		// Append update operation
		op := storage.Operation{
			Type:      storage.OpUpdate,
			TaskID:    task.UUID,
			Task:      task,
			Timestamp: time.Now(),
		}
		if err := storage.AppendOperation(ctx.DataFile, op); err != nil {
			return fmt.Errorf("failed to append operation: %w", err)
		}
	}

	// Print confirmation
	for _, task := range targetTasks {
		fmt.Fprintf(ctx.Out, "Modified task %d: %s\n", task.ID, task.Description)
	}

	return nil
}

// applyModifications applies tag and attribute modifications to a task.
func applyModifications(task *domain.Task, tokens []lexer.Token) error {
	for _, token := range tokens {
		switch token.Type {
		case lexer.TokenTag:
			// Handle +tag (add) and -tag (remove)
			if strings.HasPrefix(token.Raw, "+") {
				tagName := strings.TrimPrefix(token.Raw, "+")
				task.AddTag(tagName)
			} else if strings.HasPrefix(token.Raw, "-") {
				tagName := strings.TrimPrefix(token.Raw, "-")
				task.RemoveTag(tagName)
			}
		case lexer.TokenAttribute:
			// Handle attribute modifications (e.g., project:home)
			parts := strings.SplitN(token.Raw, ":", 2)
			if len(parts) == 2 {
				key := parts[0]
				value := parts[1]
				if key == "project" {
					task.Project = value
				} else {
					task.SetAttribute(key, value)
				}
			}
		}
	}
	return nil
}
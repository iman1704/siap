package commands

import (
	"digital-receipt-task/internal/context"
	"digital-receipt-task/internal/filter"
	"digital-receipt-task/internal/storage"
	"fmt"
	"time"
)

// DeleteCommand marks tasks as deleted.
type DeleteCommand struct{}

// Execute finds tasks by ID filter and marks them as deleted.
func (c *DeleteCommand) Execute(ctx *context.Context, cmdCtx *CommandContext) error {
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

	// Mark tasks as deleted and append delete operations
	for _, task := range targetTasks {
		task.Status = "deleted"
		op := storage.Operation{
			Type:      storage.OpDelete,
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
		fmt.Fprintf(ctx.Out, "Deleted task %d: %s\n", task.ID, task.Description)
	}

	return nil
}
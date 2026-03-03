package commands

import (
	"digital-receipt-task/internal/context"
	"digital-receipt-task/internal/storage"
	"fmt"
)

// UndoCommand undoes the last operation.
type UndoCommand struct{}

// Execute removes the last operation from the event log.
func (c *UndoCommand) Execute(ctx *context.Context, cmdCtx *CommandContext) error {
	lastOp, err := storage.UndoLastOperation(ctx.DataFile)
	if err != nil {
		return fmt.Errorf("failed to undo last operation: %w", err)
	}

	if lastOp == nil {
		fmt.Fprintf(ctx.Out, "Nothing to undo\n")
		return nil
	}

	switch lastOp.Type {
	case storage.OpCreate:
		fmt.Fprintf(ctx.Out, "Undone: Created task\n")
	case storage.OpUpdate:
		fmt.Fprintf(ctx.Out, "Undone: Updated task\n")
	case storage.OpDelete:
		fmt.Fprintf(ctx.Out, "Undone: Deleted task\n")
	}

	return nil
}

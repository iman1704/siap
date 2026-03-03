package commands

import (
	"digital-receipt-task/internal/context"
	"digital-receipt-task/internal/domain"
	"digital-receipt-task/internal/filter"
	"digital-receipt-task/internal/view"
	"os"
	"sort"
)

// ListCommand lists pending tasks.
type ListCommand struct{}

// Execute loads all tasks, applies filters, sorts by urgency, and renders them in a table.
func (c *ListCommand) Execute(ctx *context.Context, cmdCtx *CommandContext) error {
	tasks, err := loadTasks(ctx)
	if err != nil {
		return err
	}

	// Filter to only pending tasks
	pendingTasks := filterPending(tasks)

	// Extract filter criteria from filter tokens and apply filters
	filterCriteria, _ := filter.ExtractFilterTokens(cmdCtx.FilterTokens)
	filteredTasks := filter.Apply(pendingTasks, filterCriteria)

	// Sort by urgency (descending)
	sortByUrgency(filteredTasks)

	// Render the tasks
	engine := view.NewEngine()
	f, ok := ctx.Out.(*os.File)
	if !ok {
		f = os.Stdout
	}
	return engine.Render(f, filteredTasks)
}

// filterPending returns only tasks with status "pending".
func filterPending(tasks []*domain.Task) []*domain.Task {
	result := make([]*domain.Task, 0)
	for _, task := range tasks {
		if task.Status == "pending" {
			result = append(result, task)
		}
	}
	return result
}

// sortByUrgency sorts tasks by urgency score in descending order.
func sortByUrgency(tasks []*domain.Task) {
	sort.Slice(tasks, func(i, j int) bool {
		return tasks[i].CalculateUrgency() > tasks[j].CalculateUrgency()
	})
}
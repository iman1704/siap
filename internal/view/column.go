package view

import "digital-receipt-task/internal/domain"

// Column defines the interface for rendering task data in a table column.
type Column interface {
	// Measure returns the maximum width required to display the task's data in this column.
	Measure(task *domain.Task) int

	// Render returns the string representation of the task's data,
	// padded or truncated to the specified width.
	Render(task *domain.Task, width int) string

	// Header returns the column header text.
	Header() string
}

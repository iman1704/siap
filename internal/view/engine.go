// Package view provides the rendering engine for displaying tasks in the terminal.
//
// It renders tasks in a table format with configurable columns, automatic width calculation,
// and terminal-aware truncation.
package view

import (
	"digital-receipt-task/internal/domain"
	"fmt"
	"os"
	"strings"

	"github.com/mattn/go-runewidth"
	"golang.org/x/term"
)

// Engine handles the rendering of tasks into a terminal table.
type Engine struct {
	columns []Column
	width   int
}

// NewEngine creates a new rendering engine with the default columns.
func NewEngine() *Engine {
	return &Engine{
		columns: []Column{
			&IdColumn{},
			&DescColumn{},
			&ProjectColumn{},
			&TagsColumn{},
		},
		width: getTerminalWidth(),
	}
}

// NewEngineWithColumns creates a new rendering engine with custom columns.
func NewEngineWithColumns(columns []Column) *Engine {
	return &Engine{
		columns: columns,
		width:   getTerminalWidth(),
	}
}

// Render renders the given tasks to the provided writer.
func (e *Engine) Render(out *os.File, tasks []*domain.Task) error {
	if len(tasks) == 0 {
		return nil
	}

	// Calculate column widths
	colWidths := e.calculateColumnWidths(tasks)

	// Render header
	header := e.renderHeader(colWidths)
	fmt.Fprintln(out, header)

	// Render separator
	separator := e.renderSeparator(colWidths)
	fmt.Fprintln(out, separator)

	// Render rows
	for _, task := range tasks {
		row := e.renderRow(task, colWidths)
		fmt.Fprintln(out, row)
	}

	return nil
}

// calculateColumnWidths determines the optimal width for each column.
func (e *Engine) calculateColumnWidths(tasks []*domain.Task) []int {
	widths := make([]int, len(e.columns))

	// First pass: measure header widths
	for i, col := range e.columns {
		widths[i] = runewidth.StringWidth(col.Header())
	}

	// Second pass: measure task data widths
	for _, task := range tasks {
		for i, col := range e.columns {
			measured := col.Measure(task)
			if measured > widths[i] {
				widths[i] = measured
			}
		}
	}

	// Ensure minimum widths for each column
	minWidths := []int{2, 10, 7, 4} // ID, Desc, Project, Tags
	for i := range widths {
		if widths[i] < minWidths[i] {
			widths[i] = minWidths[i]
		}
	}

	// Add padding between columns (2 spaces)
	totalWidth := 0
	for _, w := range widths {
		totalWidth += w + 2
	}
	totalWidth -= 2 // Remove last padding

	// If total exceeds terminal width, shrink the Description column
	if totalWidth > e.width {
		widths = e.shrinkDescriptionColumn(widths, totalWidth)
	}

	return widths
}

// shrinkDescriptionColumn reduces the description column width to fit the terminal.
func (e *Engine) shrinkDescriptionColumn(widths []int, totalWidth int) []int {
	descIdx := 1 // Description is always at index 1

	// Minimum width for description (including ellipsis)
	const minDescWidth = 4

	// Calculate how much we need to shrink
	excess := totalWidth - e.width

	// Calculate maximum we can shrink description
	availableShrink := widths[descIdx] - minDescWidth

	if availableShrink <= 0 {
		// Can't shrink description, try other columns (except ID)
		for i := range widths {
			if i == descIdx || i == 0 { // Skip ID and Description
				continue
			}
			// Can shrink this column
			shrinkFromCol := widths[i] - 4 // Keep minimum 4 chars
			if shrinkFromCol > 0 {
				shrink := excess
				if shrink > shrinkFromCol {
					shrink = shrinkFromCol
				}
				widths[i] -= shrink
				excess -= shrink
				if excess <= 0 {
					break
				}
			}
		}
		return widths
	}

	shrink := excess
	if shrink > availableShrink {
		shrink = availableShrink
	}

	widths[descIdx] -= shrink
	return widths
}

// renderHeader renders the table header row.
func (e *Engine) renderHeader(colWidths []int) string {
	parts := make([]string, len(e.columns))
	for i, col := range e.columns {
		header := col.Header()
		parts[i] = padRight(header, colWidths[i])
	}
	return strings.Join(parts, "  ")
}

// renderSeparator renders the separator line between header and rows.
func (e *Engine) renderSeparator(colWidths []int) string {
	parts := make([]string, len(e.columns))
	for i, width := range colWidths {
		parts[i] = strings.Repeat("─", width)
	}
	return strings.Join(parts, "──")
}

// renderRow renders a single task row.
func (e *Engine) renderRow(task *domain.Task, colWidths []int) string {
	parts := make([]string, len(e.columns))
	for i, col := range e.columns {
		parts[i] = col.Render(task, colWidths[i])
	}
	return strings.Join(parts, "  ")
}

// getTerminalWidth returns the current terminal width.
func getTerminalWidth() int {
	width, _, err := term.GetSize(int(os.Stdout.Fd()))
	if err != nil {
		return 80 // Default width if we can't determine terminal size
	}
	return width
}

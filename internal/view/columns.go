package view

import (
	"digital-receipt-task/internal/domain"
	"fmt"
	"strings"

	"github.com/mattn/go-runewidth"
)

// IdColumn renders the ephemeral task ID.
type IdColumn struct{}

func (c *IdColumn) Header() string {
	return "ID"
}

func (c *IdColumn) Measure(task *domain.Task) int {
	if task.ID == 0 {
		return 1
	}
	return len(fmt.Sprintf("%d", task.ID))
}

func (c *IdColumn) Render(task *domain.Task, width int) string {
	if task.ID == 0 {
		return padLeft("", width)
	}
	return padLeft(fmt.Sprintf("%d", task.ID), width)
}

// DescColumn renders the task description.
type DescColumn struct{}

func (c *DescColumn) Header() string {
	return "Description"
}

func (c *DescColumn) Measure(task *domain.Task) int {
	return runewidth.StringWidth(task.Description)
}

func (c *DescColumn) Render(task *domain.Task, width int) string {
	desc := task.Description
	descWidth := runewidth.StringWidth(desc)

	if descWidth <= width {
		return desc
	}

	// Truncate with ellipsis
	if width <= 3 {
		return desc[:width]
	}
	// Truncate and add ellipsis
	truncated := truncateString(desc, width-3)
	return truncated + "..."
}

// ProjectColumn renders the task project.
type ProjectColumn struct{}

func (c *ProjectColumn) Header() string {
	return "Project"
}

func (c *ProjectColumn) Measure(task *domain.Task) int {
	if task.Project == "" {
		return 1 // Minimum width for empty cells
	}
	return runewidth.StringWidth(task.Project)
}

func (c *ProjectColumn) Render(task *domain.Task, width int) string {
	if task.Project == "" {
		return padRight("", width)
	}
	proj := task.Project
	projWidth := runewidth.StringWidth(proj)

	if projWidth <= width {
		return padRight(proj, width)
	}

	// Truncate if too long
	if width <= 3 {
		return proj[:width]
	}
	truncated := truncateString(proj, width-3)
	return truncated + "..."
}

// TagsColumn renders the task tags as a comma-separated list.
type TagsColumn struct{}

func (c *TagsColumn) Header() string {
	return "Tags"
}

func (c *TagsColumn) Measure(task *domain.Task) int {
	tags := buildTagsString(task)
	if tags == "" {
		return 1 // Minimum width for empty cells
	}
	return runewidth.StringWidth(tags)
}

func (c *TagsColumn) Render(task *domain.Task, width int) string {
	tags := buildTagsString(task)
	tagsWidth := runewidth.StringWidth(tags)

	if tagsWidth <= width {
		return tags
	}

	// Truncate if too long
	if width <= 3 {
		return tags[:width]
	}
	truncated := truncateString(tags, width-3)
	return truncated + "..."
}

// buildTagsString creates a comma-separated string of tags with '+' prefix.
func buildTagsString(task *domain.Task) string {
	if len(task.Tags) == 0 {
		return ""
	}

	tags := make([]string, 0, len(task.Tags))
	for tag := range task.Tags {
		tags = append(tags, "+"+tag)
	}
	return strings.Join(tags, ",")
}

// truncateString truncates a string to the specified display width,
// accounting for multi-byte characters.
func truncateString(s string, maxWidth int) string {
	if maxWidth <= 0 {
		return ""
	}

	width := 0
	result := ""
	for _, r := range s {
		rw := runewidth.RuneWidth(r)
		if width+rw > maxWidth {
			break
		}
		width += rw
		result += string(r)
	}
	return result
}

// padLeft pads a string on the left with spaces to reach the specified width.
func padLeft(s string, width int) string {
	sw := runewidth.StringWidth(s)
	if sw >= width {
		return s
	}
	return strings.Repeat(" ", width-sw) + s
}

// padRight pads a string on the right with spaces to reach the specified width.
func padRight(s string, width int) string {
	sw := runewidth.StringWidth(s)
	if sw >= width {
		return s
	}
	return s + strings.Repeat(" ", width-sw)
}

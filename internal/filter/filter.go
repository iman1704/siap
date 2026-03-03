// Package filter provides filtering logic for tasks based on parsed tokens.
//
// It supports filtering by ID, tags, projects, and custom attributes,
// with implicit AND logic when multiple criteria are specified.
package filter

import (
	"digital-receipt-task/internal/domain"
	"digital-receipt-task/internal/lexer"
	"fmt"
	"strings"
)

// Filter holds the filter criteria extracted from tokens.
type Filter struct {
	// IDs is a set of numeric IDs to match (ephemeral IDs).
	IDs map[int]bool
	// Tags is a set of tags to match (without leading '+').
	Tags map[string]bool
	// Projects is a set of project names to match.
	Projects map[string]bool
	// Attributes is a map of attribute key-value pairs to match.
	Attributes map[string]string
}

// NewFilter creates a new empty Filter.
func NewFilter() *Filter {
	return &Filter{
		IDs:        make(map[int]bool),
		Tags:       make(map[string]bool),
		Projects:   make(map[string]bool),
		Attributes: make(map[string]string),
	}
}

// ExtractFilterTokens scans the token slice and extracts filter criteria.
// It returns a Filter struct and the remaining non-filter tokens.
func ExtractFilterTokens(tokens []lexer.Token) (*Filter, []lexer.Token) {
	filter := NewFilter()
	remaining := make([]lexer.Token, 0)

	for _, token := range tokens {
		switch token.Type {
		case lexer.TokenID:
			// Parse numeric ID
			var id int
			if _, err := fmt.Sscanf(token.Raw, "%d", &id); err == nil {
				filter.IDs[id] = true
			}
		case lexer.TokenTag:
			// Extract tag name (remove leading '+' or '-')
			tagName := strings.TrimPrefix(token.Raw, "+")
			tagName = strings.TrimPrefix(tagName, "-")
			filter.Tags[tagName] = true
		case lexer.TokenAttribute:
			// Parse attribute (e.g., "project:home")
			parts := strings.SplitN(token.Raw, ":", 2)
			if len(parts) == 2 {
				key := parts[0]
				value := parts[1]
				// Special handling for "project" attribute
				if key == "project" {
					filter.Projects[value] = true
				} else {
					filter.Attributes[key] = value
				}
			}
		default:
			// Keep non-filter tokens
			remaining = append(remaining, token)
		}
	}

	return filter, remaining
}

// Apply filters the task slice based on the filter criteria.
// It uses implicit AND logic: tasks must match ALL specified criteria.
func Apply(tasks []*domain.Task, filter *Filter) []*domain.Task {
	// If no filter criteria, return all tasks
	if len(filter.IDs) == 0 && len(filter.Tags) == 0 && len(filter.Projects) == 0 && len(filter.Attributes) == 0 {
		return tasks
	}

	result := make([]*domain.Task, 0)

	for _, task := range tasks {
		if matchesFilter(task, filter) {
			result = append(result, task)
		}
	}

	return result
}

// matchesFilter checks if a task matches all filter criteria (implicit AND).
func matchesFilter(task *domain.Task, filter *Filter) bool {
	// Check ID filter
	if len(filter.IDs) > 0 && !filter.IDs[task.ID] {
		return false
	}

	// Check tags filter (task must have ALL specified tags)
	for tag := range filter.Tags {
		if !task.HasTag(tag) {
			return false
		}
	}

	// Check projects filter
	if len(filter.Projects) > 0 && !filter.Projects[task.Project] {
		return false
	}

	// Check attributes filter (task must have ALL specified attributes with matching values)
	for key, value := range filter.Attributes {
		if task.GetAttribute(key) != value {
			return false
		}
	}

	return true
}

// ApplyByID filters tasks to only those matching the specified IDs.
// This is a helper for mutation commands that target specific task IDs.
func ApplyByID(tasks []*domain.Task, filter *Filter) []*domain.Task {
	// If no ID filter specified, return empty (mutation requires target)
	if len(filter.IDs) == 0 {
		return []*domain.Task{}
	}

	result := make([]*domain.Task, 0)
	for _, task := range tasks {
		if filter.IDs[task.ID] {
			result = append(result, task)
		}
	}

	return result
}

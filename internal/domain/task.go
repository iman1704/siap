// Package domain provides the core domain models for the task management system.
//
// The Task type is the central entity, representing a single task with its properties,
// status, tags, and custom attributes.
package domain

import (
	"time"

	"github.com/google/uuid"
)

// Task represents a single task in the system.
type Task struct {
	// UUID is the immutable unique identifier for the task.
	UUID uuid.UUID `json:"uuid"`

	// ID is an ephemeral integer assigned only to pending tasks at runtime.
	// It is not persisted to storage.
	ID int `json:"-"`

	// Description is the textual content of the task.
	Description string `json:"description"`

	// Status indicates the current state of the task.
	// Possible values: "pending", "completed", "deleted".
	Status string `json:"status"`

	// Tags is a set of labels attached to the task.
	// The map key is the tag name (without leading '+'), value is always true.
	Tags map[string]bool `json:"tags"`

	// Project is an optional grouping attribute.
	Project string `json:"project,omitempty"`

	// EntryDate is the timestamp when the task was created.
	EntryDate time.Time `json:"entry_date"`

	// DueDate is an optional deadline for the task.
	DueDate *time.Time `json:"due_date,omitempty"`

	// Attributes holds arbitrary key‑value pairs for extensibility.
	Attributes map[string]string `json:"attributes"`
}

// NewTask creates a new pending Task with the given description.
// The UUID is generated, EntryDate set to now, and maps are initialized.
func NewTask(description string) *Task {
	now := time.Now()
	return &Task{
		UUID:        uuid.New(),
		ID:          0, // will be assigned later by the storage layer
		Description: description,
		Status:      "pending",
		Tags:        make(map[string]bool),
		Project:     "",
		EntryDate:   now,
		DueDate:     nil,
		Attributes:  make(map[string]string),
	}
}

// AddTag adds a tag to the task (without the leading '+').
func (t *Task) AddTag(tag string) {
	t.Tags[tag] = true
}

// RemoveTag removes a tag from the task.
func (t *Task) RemoveTag(tag string) {
	delete(t.Tags, tag)
}

// HasTag returns true if the task has the given tag.
func (t *Task) HasTag(tag string) bool {
	return t.Tags[tag]
}

// SetAttribute sets a custom key‑value pair.
func (t *Task) SetAttribute(key, value string) {
	t.Attributes[key] = value
}

// GetAttribute retrieves a custom attribute, returning empty string if missing.
func (t *Task) GetAttribute(key string) string {
	return t.Attributes[key]
}

// CalculateUrgency computes an urgency score for the task.
// Higher scores indicate more urgent tasks.
// Scoring rules:
//   - Due today: +10
//   - Due within 3 days: +5
//   - Due within 7 days: +2
//   - Has project: +1
//   - Has "urgent" tag: +5
//   - Has any tag: +1 per tag (max +3)
//   - Entry date older than 7 days: +1 per week (max +5)
func (t *Task) CalculateUrgency() int {
	score := 0
	now := time.Now()

	// Due date scoring
	if t.DueDate != nil {
		daysUntilDue := t.DueDate.Sub(now).Hours() / 24
		if daysUntilDue < 0 {
			// Overdue: high urgency
			score += 15
		} else if daysUntilDue < 1 {
			// Due today
			score += 10
		} else if daysUntilDue < 3 {
			// Due within 3 days
			score += 5
		} else if daysUntilDue < 7 {
			// Due within 7 days
			score += 2
		}
	}

	// Project scoring
	if t.Project != "" {
		score += 1
	}

	// Tag scoring
	if t.HasTag("urgent") {
		score += 5
	}
	tagCount := 0
	for range t.Tags {
		tagCount++
		if tagCount >= 3 {
			break
		}
	}
	score += tagCount

	// Age scoring (older tasks get more urgent)
	daysOld := now.Sub(t.EntryDate).Hours() / 24
	if daysOld > 7 {
		weeksOld := int(daysOld / 7)
		ageScore := weeksOld
		if ageScore > 5 {
			ageScore = 5
		}
		score += ageScore
	}

	return score
}

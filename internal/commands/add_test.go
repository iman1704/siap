package commands

import (
	"digital-receipt-task/internal/context"
	"digital-receipt-task/internal/lexer"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestAddCommand_Execute(t *testing.T) {
	// Create a temporary directory for test data
	tmpDir := t.TempDir()
	dataFile := filepath.Join(tmpDir, "tasks.jsonl")

	ctx := context.NewContext().WithDataFile(dataFile)

	tests := []struct {
		name        string
		args        []string
		wantDesc    string
		wantTags    []string
		wantProject string
		wantDueDate *time.Time
	}{
		{
			name:        "simple description",
			args:        []string{"add", "Buy", "milk"},
			wantDesc:    "Buy milk",
			wantTags:    nil,
			wantProject: "",
		},
		{
			name:        "with tag",
			args:        []string{"add", "Buy", "milk", "+groceries"},
			wantDesc:    "Buy milk",
			wantTags:    []string{"groceries"},
			wantProject: "",
		},
		{
			name:        "with project",
			args:        []string{"add", "Fix", "bug", "project:backend"},
			wantDesc:    "Fix bug",
			wantTags:    nil,
			wantProject: "backend",
		},
		{
			name:        "with due date",
			args:        []string{"add", "Submit", "report", "due:2025-12-31"},
			wantDesc:    "Submit report",
			wantDueDate: mustParseTime("2025-12-31"),
		},
		{
			name:        "multiple tags and attributes",
			args:        []string{"add", "Write", "docs", "+work", "+urgent", "project:api", "due:2025-06-15"},
			wantDesc:    "Write docs",
			wantTags:    []string{"work", "urgent"},
			wantProject: "api",
			wantDueDate: mustParseTime("2025-06-15"),
		},
		{
			name:        "minus tag ignored for add",
			args:        []string{"add", "Task", "-blocked"},
			wantDesc:    "Task",
			wantTags:    nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Remove the data file before each test to start fresh
			_ = os.Remove(dataFile)

			tokens := lexer.Parse(tt.args[1:]) // skip the "add" command token
			cmd := &AddCommand{}
			cmdCtx := &CommandContext{
				FilterTokens:     []lexer.Token{},
				ModificationTokens: tokens,
			}
			err := cmd.Execute(ctx, cmdCtx)
			require.NoError(t, err)

			// Load tasks back and verify
			tasks, err := loadTasks(ctx)
			require.NoError(t, err)
			require.Len(t, tasks, 1, "expected exactly one task after add")

			task := tasks[0]
			assert.Equal(t, tt.wantDesc, task.Description)
			assert.Equal(t, "pending", task.Status)
			assert.NotEqual(t, 0, task.ID, "task should have ephemeral ID")
			assert.NotEmpty(t, task.UUID)

			// Check tags
			for _, tag := range tt.wantTags {
				assert.True(t, task.HasTag(tag), "missing tag %q", tag)
			}
			assert.Equal(t, len(tt.wantTags), len(task.Tags))

			// Check project
			if tt.wantProject != "" {
				assert.Equal(t, tt.wantProject, task.Project)
			}

			// Check due date
			if tt.wantDueDate != nil {
				require.NotNil(t, task.DueDate)
				assert.Equal(t, tt.wantDueDate.UTC().Truncate(24*time.Hour), task.DueDate.UTC().Truncate(24*time.Hour))
			}
		})
	}
}

func mustParseTime(s string) *time.Time {
	t, err := time.Parse("2006-01-02", s)
	if err != nil {
		panic(err)
	}
	return &t
}
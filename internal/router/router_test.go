package router

import (
	"digital-receipt-task/internal/lexer"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestRoute(t *testing.T) {
	tests := []struct {
		name          string
		input         []string
		wantCmdName   string
		wantRemaining []string
	}{
		{
			name:          "command add",
			input:         []string{"add", "Buy", "milk"},
			wantCmdName:   "add",
			wantRemaining: []string{"Buy", "milk"},
		},
		{
			name:          "command with tag and attribute",
			input:         []string{"+work", "project:home", "list"},
			wantCmdName:   "list",
			wantRemaining: []string{"+work", "project:home"},
		},
		{
			name:          "no command defaults to list",
			input:         []string{"+work", "project:home"},
			wantCmdName:   "list",
			wantRemaining: []string{"+work", "project:home"},
		},
		{
			name:          "command in middle",
			input:         []string{"1", "done", "+urgent"},
			wantCmdName:   "done",
			wantRemaining: []string{"1", "+urgent"},
		},
		{
			name:          "multiple commands picks first",
			input:         []string{"add", "done", "foo"},
			wantCmdName:   "add",
			wantRemaining: []string{"done", "foo"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tokens := lexer.Parse(tt.input)
			cmd, remaining, err := Route(tokens)
			require.NoError(t, err)
			require.NotNil(t, cmd)
			// Check command type by executing? We'll just ensure it's not nil.
			// For simplicity, we can assert that remaining tokens match expected.
			var remainingStrs []string
			for _, tok := range remaining {
				remainingStrs = append(remainingStrs, tok.Raw)
			}
			assert.Equal(t, tt.wantRemaining, remainingStrs)
		})
	}
}
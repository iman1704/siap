package lexer

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParse(t *testing.T) {
	tests := []struct {
		name     string
		input    []string
		expected []Token
	}{
		{
			name:  "example from spec",
			input: []string{"+work", "1", "done"},
			expected: []Token{
				{Type: TokenTag, Raw: "+work"},
				{Type: TokenID, Raw: "1"},
				{Type: TokenCommand, Raw: "done"},
			},
		},
		{
			name:  "attribute token",
			input: []string{"project:home", "add"},
			expected: []Token{
				{Type: TokenAttribute, Raw: "project:home"},
				{Type: TokenCommand, Raw: "add"},
			},
		},
		{
			name:  "tag with minus",
			input: []string{"-blocked", "42"},
			expected: []Token{
				{Type: TokenTag, Raw: "-blocked"},
				{Type: TokenID, Raw: "42"},
			},
		},
		{
			name:  "text token",
			input: []string{"Buy", "milk", "+groceries"},
			expected: []Token{
				{Type: TokenText, Raw: "Buy"},
				{Type: TokenText, Raw: "milk"},
				{Type: TokenTag, Raw: "+groceries"},
			},
		},
		{
			name:  "mixed tokens",
			input: []string{"1", "modify", "due:tomorrow", "+urgent"},
			expected: []Token{
				{Type: TokenID, Raw: "1"},
				{Type: TokenCommand, Raw: "modify"},
				{Type: TokenAttribute, Raw: "due:tomorrow"},
				{Type: TokenTag, Raw: "+urgent"},
			},
		},
		{
			name:  "empty input",
			input: []string{},
			expected: []Token{},
		},
		{
			name:  "numeric ID with leading zeros",
			input: []string{"001"},
			expected: []Token{
				{Type: TokenID, Raw: "001"},
			},
		},
		{
			name:  "attribute with underscore",
			input: []string{"due_date:2025-01-01"},
			expected: []Token{
				{Type: TokenAttribute, Raw: "due_date:2025-01-01"},
			},
		},
		{
			name:  "command case sensitive",
			input: []string{"ADD", "Add", "add"},
			expected: []Token{
				{Type: TokenText, Raw: "ADD"},
				{Type: TokenText, Raw: "Add"},
				{Type: TokenCommand, Raw: "add"},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := Parse(tt.input)
			require.Len(t, result, len(tt.expected), "token count mismatch")
			for i, exp := range tt.expected {
				assert.Equal(t, exp.Type, result[i].Type, "token type mismatch at index %d", i)
				assert.Equal(t, exp.Raw, result[i].Raw, "token raw value mismatch at index %d", i)
			}
		})
	}
}

func TestTokenTypeString(t *testing.T) {
	tests := []struct {
		tokenType TokenType
		expected  string
	}{
		{TokenAttribute, "Attribute"},
		{TokenTag, "Tag"},
		{TokenID, "ID"},
		{TokenCommand, "Command"},
		{TokenText, "Text"},
		{TokenType(99), "Unknown"},
	}

	for _, tt := range tests {
		t.Run(tt.expected, func(t *testing.T) {
			assert.Equal(t, tt.expected, tt.tokenType.String())
		})
	}
}
package lexer

// TokenType represents the category of a lexical token.
type TokenType int

const (
	// TokenAttribute matches key:value patterns (e.g., "project:home").
	TokenAttribute TokenType = iota
	// TokenTag matches tags starting with '+' or '-' (e.g., "+work", "-blocked").
	TokenTag
	// TokenID matches numeric IDs (e.g., "1", "42").
	TokenID
	// TokenCommand matches known command strings (e.g., "add", "list").
	TokenCommand
	// TokenText matches any other unclassified string.
	TokenText
)

// Token holds a single lexical unit parsed from the command line.
type Token struct {
	Type  TokenType
	Raw   string
}

// String returns a human-readable representation of the token type.
func (tt TokenType) String() string {
	switch tt {
	case TokenAttribute:
		return "Attribute"
	case TokenTag:
		return "Tag"
	case TokenID:
		return "ID"
	case TokenCommand:
		return "Command"
	case TokenText:
		return "Text"
	default:
		return "Unknown"
	}
}
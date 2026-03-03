// Package context provides the global application context for the mt CLI.
//
// The Context type holds configuration, I/O writers, and the path to the task database.
package context

import (
	"io"
	"os"
	"path/filepath"
	"strings"
)

// expandHomeDir expands a leading "~" to the current user's home directory.
func expandHomeDir(path string) string {
	if strings.HasPrefix(path, "~/") {
		if home, err := os.UserHomeDir(); err == nil {
			return filepath.Join(home, path[2:])
		}
	}
	return path
}

// Context holds the global application state: configuration, output writers,
// and a connection to the task database.
type Context struct {
	// DataFile is the absolute path to the JSONL task database.
	DataFile string
	// Out is the writer for standard output (usually os.Stdout).
	Out io.Writer
	// Err is the writer for standard error (usually os.Stderr).
	Err io.Writer
}

// NewContext creates a default Context with sensible defaults.
func NewContext() *Context {
	return &Context{
		DataFile: expandHomeDir("~/.mt/tasks.jsonl"),
		Out:      os.Stdout,
		Err:      os.Stderr,
	}
}

// WithDataFile returns a copy of the Context with DataFile set to the given path.
func (c *Context) WithDataFile(path string) *Context {
	cp := *c
	cp.DataFile = expandHomeDir(path)
	return &cp
}

// WithOutput returns a copy of the Context with Out and Err set to the given writers.
func (c *Context) WithOutput(out, err io.Writer) *Context {
	cp := *c
	cp.Out = out
	cp.Err = err
	return &cp
}

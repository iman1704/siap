// Package main provides the entry point for the siap command-line task manager.
//
// The siap application is a fast, pipeline-driven task manager that follows the grammar:
//
//	siap <filter> <command> <modifications>
//
// Examples:
//
//	siap add Buy milk +groceries due:tomorrow
//	siap +work list
//	siap 1 done
package main

import (
	"fmt"
	"os"

	"digital-receipt-task/internal/commands"
	"digital-receipt-task/internal/context"
	"digital-receipt-task/internal/lexer"
	"digital-receipt-task/internal/router"
	"github.com/urfave/cli/v2"
)

// main is the entry point for the siap CLI application.
// It initializes the CLI app, parses command-line arguments, and routes them to the appropriate command.
func main() {
	app := &cli.App{
		Name:      "siap",
		Usage:     "Siap: a cli todo list written in Go",
		Version:   "1.0.0",
		ArgsUsage: "[filter] [command] [modifications]",
		Description: `Siap: a cli todo list written in Go.

Every command follows the grammar:
  siap <filter> <command> <modifications>

Examples:
  siap add Buy milk +groceries due:tomorrow
  siap +work list
  siap 1 done`,
		// Capture raw arguments and bypass standard subcommand routing
		Action: func(c *cli.Context) error {
			args := c.Args().Slice()
			ctx := context.NewContext()
			if dataFile := c.String("data-file"); dataFile != "" {
				ctx = ctx.WithDataFile(dataFile)
			}
			tokens := lexer.Parse(args)
			result, _, err := router.RouteWithSeparatedTokens(tokens)
			if err != nil {
				return err
			}
			cmdCtx := &commands.CommandContext{
				FilterTokens:     result.FilterTokens,
				ModificationTokens: result.ModificationTokens,
			}
			return result.Command.Execute(ctx, cmdCtx)
		},
		// Global flags (e.g., --config, --data-file)
		Flags: []cli.Flag{
			&cli.StringFlag{
				Name:    "data-file",
				Aliases: []string{"f"},
				Usage:   "path to the JSONL task database",
				Value:   "~/.siap/tasks.jsonl",
				EnvVars: []string{"SIAP_DATA_FILE"},
			},
		},
	}

	if err := app.Run(os.Args); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}
}

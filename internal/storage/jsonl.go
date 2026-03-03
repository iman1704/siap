// Package storage provides persistent storage for tasks using event sourcing.
//
// Tasks are stored in a JSONL (JSON Lines) file where each line represents an operation
// (CREATE, UPDATE, DELETE). The current state is reconstructed by replaying operations.
package storage

import (
	"bufio"
	"encoding/json"
	"os"
	"time"

	"digital-receipt-task/internal/domain"

	"github.com/google/uuid"
)

// OperationType represents the type of operation in event sourcing.
type OperationType string

const (
	// OpCreate represents a task creation operation.
	OpCreate OperationType = "CREATE"
	// OpUpdate represents a task update operation.
	OpUpdate OperationType = "UPDATE"
	// OpDelete represents a task deletion operation.
	OpDelete OperationType = "DELETE"
)

// Operation represents a single event in the event sourcing log.
type Operation struct {
	// Type is the operation type (CREATE, UPDATE, DELETE).
	Type OperationType `json:"op"`
	// TaskID is the UUID of the affected task.
	TaskID uuid.UUID `json:"task_id"`
	// Task is the task data (for CREATE and UPDATE operations).
	Task *domain.Task `json:"task,omitempty"`
	// Timestamp is when the operation occurred.
	Timestamp time.Time `json:"timestamp"`
}

// LoadTasks reads the JSONL event log and rebuilds the current state.
// If the file does not exist, an empty slice is returned (no error).
// Tasks with status "pending" receive sequential ephemeral IDs (1,2,3…).
func LoadTasks(filePath string) ([]*domain.Task, error) {
	file, err := os.Open(filePath)
	if err != nil {
		if os.IsNotExist(err) {
			// No file yet -> empty task list
			return []*domain.Task{}, nil
		}
		return nil, err
	}
	defer file.Close()

	// Rebuild state from operations
	tasks := make(map[uuid.UUID]*domain.Task)

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := scanner.Bytes()
		if len(line) == 0 {
			continue
		}

		// Try to parse as operation first (event sourcing format)
		var op Operation
		if err := json.Unmarshal(line, &op); err == nil && op.Type != "" {
			// This is an operation
			switch op.Type {
			case OpCreate:
				if op.Task != nil {
					tasks[op.TaskID] = op.Task
				}
			case OpUpdate:
				if op.Task != nil {
					tasks[op.TaskID] = op.Task
				}
			case OpDelete:
				delete(tasks, op.TaskID)
			}
			continue
		}

		// Backward compatibility: try to parse as legacy task format
		var task domain.Task
		if err := json.Unmarshal(line, &task); err == nil {
			sanitizeTask(&task)
			tasks[task.UUID] = &task
		}
	}

	if err := scanner.Err(); err != nil {
		return nil, err
	}

	// Convert map to slice
	result := make([]*domain.Task, 0, len(tasks))
	for _, task := range tasks {
		sanitizeTask(task)
		result = append(result, task)
	}

	AssignEphemeralIDs(result)
	return result, nil
}

// sanitizeTask ensures that maps are initialized after JSON unmarshaling.
func sanitizeTask(t *domain.Task) {
	if t.Tags == nil {
		t.Tags = make(map[string]bool)
	}
	if t.Attributes == nil {
		t.Attributes = make(map[string]string)
	}
}

// AssignEphemeralIDs gives sequential integer IDs (1,2,3…) to pending tasks.
// Non‑pending tasks keep ID == 0.
func AssignEphemeralIDs(tasks []*domain.Task) {
	nextID := 1
	for _, task := range tasks {
		if task.Status == "pending" {
			task.ID = nextID
			nextID++
		}
	}
}

// SaveTasks writes the entire task slice to a JSONL file using event sourcing.
// It appends CREATE or UPDATE operations for each task.
func SaveTasks(filePath string, tasks []*domain.Task) error {
	file, err := os.OpenFile(filePath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}
	defer file.Close()

	writer := bufio.NewWriter(file)
	for _, task := range tasks {
		op := Operation{
			Type:      OpCreate, // For simplicity, treat all as CREATE
			TaskID:    task.UUID,
			Task:      task,
			Timestamp: time.Now(),
		}
		if err := writeOperation(writer, op); err != nil {
			return err
		}
	}
	return writer.Flush()
}

// AppendOperation appends a single operation to the event log.
func AppendOperation(filePath string, op Operation) error {
	file, err := os.OpenFile(filePath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}
	defer file.Close()

	writer := bufio.NewWriter(file)
	if err := writeOperation(writer, op); err != nil {
		return err
	}
	return writer.Flush()
}

// writeOperation writes a single operation to the writer.
func writeOperation(writer *bufio.Writer, op Operation) error {
	data, err := json.Marshal(op)
	if err != nil {
		return err
	}
	if _, err := writer.Write(data); err != nil {
		return err
	}
	if err := writer.WriteByte('\n'); err != nil {
		return err
	}
	return nil
}

// UndoLastOperation removes the last operation from the event log and returns it.
func UndoLastOperation(filePath string) (*Operation, error) {
	// Read all operations
	ops, err := readAllOperations(filePath)
	if err != nil {
		return nil, err
	}

	if len(ops) == 0 {
		return nil, nil // Nothing to undo
	}

	// Get the last operation
	lastOp := ops[len(ops)-1]

	// Rewrite the file without the last operation
	if err := rewriteOperations(filePath, ops[:len(ops)-1]); err != nil {
		return nil, err
	}

	return &lastOp, nil
}

// readAllOperations reads all operations from the file.
func readAllOperations(filePath string) ([]Operation, error) {
	file, err := os.Open(filePath)
	if err != nil {
		if os.IsNotExist(err) {
			return []Operation{}, nil
		}
		return nil, err
	}
	defer file.Close()

	var ops []Operation
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := scanner.Bytes()
		if len(line) == 0 {
			continue
		}
		var op Operation
		if err := json.Unmarshal(line, &op); err == nil {
			ops = append(ops, op)
		}
	}
	return ops, scanner.Err()
}

// rewriteOperations rewrites the file with the given operations.
func rewriteOperations(filePath string, ops []Operation) error {
	file, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer file.Close()

	writer := bufio.NewWriter(file)
	for _, op := range ops {
		if err := writeOperation(writer, op); err != nil {
			return err
		}
	}
	return writer.Flush()
}

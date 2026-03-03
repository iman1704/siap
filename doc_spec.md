# Go Documentation Specification

## 1. Purpose
This document outlines the required standards and conventions for documenting Go (Golang) code within our projects. Following these specifications ensures that our codebase remains highly readable, maintainable, and perfectly compatible with standard Go tooling (e.g., `go doc`, `pkg.go.dev`, and IDE hovers).

## 2. General Principles
* **Exported Identifiers MUST be Documented:** Every exported (capitalized) package, variable, constant, type, function, and method must have a documentation comment.
* **Unexported Identifiers SHOULD be Documented:** While not strictly enforced by standard Go tooling, non-trivial unexported (lowercase) identifiers should be documented to aid internal maintenance.
* **Plain Text Only:** Do not use HTML, Javadoc-style tags (e.g., `@param`, `@return`), or complex Markdown. Go documentation relies on specific plain-text formatting.
* **Use Line Comments:** Always use `//` for documentation comments. Block comments `/* ... */` are reserved for disabling large blocks of code.

## 3. Syntax & Formatting Standards

### 3.1 The Golden Rule (Naming)
Every documentation comment MUST be a complete sentence that begins with the exact name of the identifier it describes.

* **Bad:** `// This function returns the current user.`
* **Good:** `// GetCurrentUser returns the currently authenticated user.`

### 3.2 Proximity
Documentation comments MUST be placed immediately above the declaration. There MUST NOT be any blank lines between the comment and the code.

* **Bad:**
  ```go
  // User represents a system user.
  
  type User struct {}
  ```
* **Good:**
  ```go
  // User represents a system user.
  type User struct {}
  ```

### 3.3 Paragraphs
Separate multiple paragraphs with a single empty comment line `//`.

```go
// CreateUser creates a new user in the database.
//
// If the user already exists, it returns an ErrUserExists error.
func CreateUser() error { ... }
```

## 4. Component-Specific Rules

### 4.1 Packages
* Every package MUST have a package comment.
* The comment MUST be placed immediately above the `package` clause.
* It MUST begin with the words `// Package <name>`.
* **Large Packages:** If the package documentation is extensive, it MUST be placed in a dedicated file named `doc.go`.

```go
// Package auth provides authentication and authorization middleware.
//
// It supports JWT, OAuth2, and basic HTTP authentication.
package auth
```

### 4.2 Functions and Methods
* Document what the function computes or returns.
* Explicitly mention error conditions (e.g., "returns ErrNotFound if...").
* Do not list parameters separately. Integrate them naturally into the description.

```go
// ParseConfig reads the configuration from the provided file path.
// It returns a Config struct on success, or an error if the file
// cannot be read or parsed correctly.
func ParseConfig(path string) (*Config, error) { ... }
```

### 4.3 Types and Fields
* Document the type itself.
* Document every exported field within a `struct` or method within an `interface`.

```go
// ServerConfig holds HTTP server initialization variables.
type ServerConfig struct {
    // Port is the TCP port the server listens on.
    Port int
    
    // ReadTimeout is the maximum duration for reading the entire request.
    ReadTimeout time.Duration
}
```

### 4.4 Variables and Constants
* Document individual exported constants/variables.
* For grouped declarations, you MAY document the entire block, the individual items, or both.

```go
// Standard HTTP status codes.
const (
    // StatusOK indicates the request succeeded.
    StatusOK = 200

    // StatusBadRequest indicates a client error.
    StatusBadRequest = 400
)
```

## 5. Advanced Formatting (Go 1.19+)

Go 1.19 introduced formalized rendering rules. The following formatting standards MUST be used when complex documentation is required:

### 5.1 Code Blocks
To create a preformatted code block (for examples), indent the text by a single tab or multiple spaces.

```go
// GenerateID creates a new unique identifier.
//
// Example:
//
//  id := GenerateID()
//  fmt.Println(id)
//
func GenerateID() string { ... }
```

### 5.2 Lists
Use `-` or `*` to create unordered lists.

```go
// Validate checks the request for the following:
//  - Valid email format
//  - Password length > 8 characters
//  - Non-empty username
func Validate(req Request) error { ... }
```

### 5.3 Links
To link to external URLs or internal Go identifiers, enclose the text in brackets `[ ]`.

* **External Links:** Define the URL reference at the bottom of the comment block.
* **Internal Links:** Put the exact Go identifier in brackets (e.g., `[bytes.Buffer]`).

```go
// NewReader creates a custom reader. 
// It behaves similarly to [bufio.NewReader].
//
// For more on buffer behavior, see the [Go IO documentation].
//
// [Go IO documentation]: https://pkg.go.dev/io
func NewReader() { ... }
```

## 6. Deprecation Policy
When an exported identifier is no longer recommended for use, it MUST be marked as deprecated.

* The deprecation notice MUST be a new paragraph.
* It MUST start exactly with `// Deprecated: `
* It MUST instruct the developer on what to use instead.

```go
// HashPassword hashes a string using MD5.
//
// Deprecated: Use HashPasswordBcrypt instead. MD5 is no longer secure.
func HashPassword(p string) string { ... }
```

## 7. Enforcement Tooling
To ensure this specification is adhered to, the following tooling will be utilized in our CI/CD pipeline:

1. **`go vet`**: Standard Go vetting tool.
2. **`golangci-lint`**: Configured to run:
   * `revive` (with `exported` rule enabled to catch missing doc comments).
   * `staticcheck` (to catch formatting issues and deprecation warnings).
3. **`go doc -all`**: Run locally to preview documentation rendering before committing. 

*Code reviews MUST block PRs that introduce exported identifiers without accompanying documentation compliant with this spec.*

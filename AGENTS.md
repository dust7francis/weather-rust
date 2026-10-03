# AGENTS.md

## 1. Purpose

This file defines development guidelines for AI coding agents working in this repository.

Agents should:

* Understand the existing architecture before making changes.
* Prefer small, focused changes.
* Reuse existing code and dependencies where practical.
* Avoid unnecessary refactoring.
* Keep changes easy to review and test.

## 2. Project Structure

Before modifying code:

1. Inspect the repository structure.
2. Read `README.md` and relevant documentation.
3. Identify the application entry points.
4. Identify the test structure.
5. Check existing configuration and dependency files.

Do not assume the project architecture.

## 3. General Coding Principles

* Prefer simple, readable code over clever implementations.
* Follow the existing coding style.
* Keep functions and modules focused on one responsibility.
* Avoid unnecessary abstractions.
* Avoid duplicated logic.
* Preserve backwards compatibility unless a breaking change is explicitly requested.
* Do not introduce a new dependency when the existing standard library or project dependency is sufficient.

## 4. Changes

Before making a change:

* Understand why the change is required.
* Identify affected components.
* Consider error handling and edge cases.
* Check whether tests need to be added or updated.

After making a change:

* Review the resulting diff.
* Remove debugging code and unused imports.
* Run the relevant tests.
* Run formatting and linting where available.

## 5. Python

For Python code:

* Use Python 3.x according to the project's configured version.
* Prefer type hints for public functions and important internal interfaces.
* Use `venv`, `uv`, Poetry, or the project's existing environment management rather than introducing another tool.
* Follow the project's existing formatter/linter configuration.
* Prefer `asyncio` for I/O-bound asynchronous operations.
* Handle exceptions explicitly at application boundaries.
* Do not silently swallow exceptions.

Example:

```python
async def get_weather(location: str) -> Weather:
    ...
```

## 6. Rust

For Rust code:

* Follow standard Rust idioms.
* Prefer ownership and borrowing over unnecessary cloning.
* Use `Result` and `Option` appropriately.
* Avoid `unwrap()` and `expect()` in production paths unless failure is genuinely impossible and documented.
* Run:

```bash
cargo fmt
cargo check
cargo test
```

* Run `cargo clippy` when available.

Prefer meaningful error types and contextual error messages.

## 7. MCP Servers

For Model Context Protocol (MCP) servers:

* Keep MCP tool interfaces small and predictable.
* Clearly define tool names, descriptions, inputs, and outputs.
* Validate tool inputs.
* Return useful, structured errors.
* Do not expose secrets through tool responses.
* Keep external API credentials out of source code.
* Prefer environment variables or the project's existing secret-management mechanism.
* Avoid exposing unnecessary filesystem, network, or shell access.

For a local MCP server, document:

```text
Tool name
Purpose
Input parameters
Output format
Errors
Required environment variables
Example invocation
```

## 8. APIs and External Services

When calling an external API:

* Set reasonable timeouts.
* Handle HTTP errors explicitly.
* Validate responses before using them.
* Avoid logging credentials, tokens, or sensitive data.
* Respect API rate limits.
* Do not hard-code environment-specific URLs.

Use configuration for:

```text
API_BASE_URL
API_KEY
TIMEOUT
ENVIRONMENT
```

where appropriate.

## 9. Security

Never commit:

* API keys
* passwords
* OAuth tokens
* private keys
* certificates containing private material
* production credentials
* `.env` files containing secrets

Check `.gitignore` before creating local configuration files.

If credentials are accidentally exposed, treat them as compromised and recommend rotation.

## 10. Tests

New functionality should normally include tests.

At minimum:

* Test the normal case.
* Test important error cases.
* Test boundary conditions where relevant.

Do not remove or weaken existing tests simply to make a change pass.

Run the smallest relevant test suite first, followed by the full suite when practical.

## 11. Documentation

Update documentation when a change affects:

* Installation
* Configuration
* Environment variables
* API/MCP tools
* CLI commands
* Architecture
* User-visible behaviour

Prefer updating existing documentation rather than creating duplicate documentation.

## 12. Git

Do not modify Git history unless explicitly requested.

Do not:

* force-push
* reset unrelated user changes
* delete branches
* rewrite commits

without explicit approval.

Before committing, inspect:

```bash
git status
git diff
```

Do not include unrelated changes in a commit.

## 13. Existing User Changes

Treat existing uncommitted changes as intentional.

Before modifying a file that already has user changes:

* Inspect the existing diff.
* Preserve unrelated changes.
* Do not overwrite or revert them.

## 14. Dependencies

Before adding a dependency:

1. Check whether the functionality already exists in the project.
2. Check whether the standard library provides a suitable solution.
3. Consider maintenance, security, licence, and dependency size.
4. Use the project's existing package manager.

Avoid adding dependencies for trivial functionality.

## 15. Configuration

Keep environment-specific configuration outside source code.

Prefer:

```text
.env.example
config.example.yaml
configuration documentation
```

over committing actual credentials.

Document required configuration clearly.

## 16. Generated Files

Do not manually edit generated files unless the project explicitly requires it.

Identify the source/template/schema that generates the file and modify that instead.

## 17. Command Execution

Before running potentially destructive commands:

* Understand what the command will modify.
* Prefer read-only inspection first.
* Avoid commands that delete data unless explicitly requested.

Examples requiring caution:

```bash
rm -rf
git reset --hard
git clean
terraform destroy
kubectl delete
```

## 18. Agent Workflow

For a typical task:

### Step 1 — Understand

Inspect:

```text
README.md
AGENTS.md
package/dependency files
source structure
tests
configuration
```

### Step 2 — Plan

Identify:

* files that need changing
* dependencies affected
* tests required
* potential compatibility issues

### Step 3 — Implement

Make the smallest reasonable change.

### Step 4 — Verify

Run appropriate:

```text
formatter
linter
type checker
unit tests
integration tests
build
```

### Step 5 — Review

Inspect:

```bash
git diff
git status
```

Check for:

* accidental changes
* debugging code
* secrets
* unused dependencies
* missing tests
* documentation changes

### Step 6 — Report

Summarize:

1. What changed
2. Why it changed
3. Tests/checks performed
4. Any remaining issues or limitations

## 19. When Requirements Are Ambiguous

If a reasonable interpretation can be made without significant risk, proceed using the simplest interpretation.

Ask for clarification when:

* the change could cause data loss
* requirements conflict
* a breaking API change may be involved
* security implications are unclear
* multiple substantially different architectures are possible

Do not invent requirements.

## 20. Priority of Instructions

When instructions conflict, use this order:

1. System/platform instructions
2. User's explicit request
3. This `AGENTS.md`
4. Existing project conventions
5. Agent's assumptions

When uncertain, preserve existing behaviour rather than making speculative changes.

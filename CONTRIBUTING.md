# Contributing

Thanks for your interest in contributing!

## Development Setup

```bash
cd mcp
cargo build         # Build
cargo test          # Run tests
cargo clippy        # Lint
cargo fmt           # Format
```

## Commit Messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/) with [Release Please](https://github.com/googleapis/release-please) for automated releases.

```
feat: add new feature        → minor version bump
fix: fix a bug               → patch version bump
feat!: breaking change       → major version bump (post-1.0)
chore: maintenance tasks     → no release
docs: documentation only     → no release
```

## Pull Requests

1. Fork the repo and create a branch from `main`
2. Write tests for new functionality
3. Ensure `cargo test`, `cargo clippy`, and `cargo fmt --check` all pass
4. Use conventional commit messages
5. Open a PR against `main`

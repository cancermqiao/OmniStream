# Contributing

## Development Workflow

1. Create a branch from the latest main.
2. Keep changes scoped and atomic.
3. Run quality gates locally before opening PR.

## Local Quality Gates

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
```

Optional shortcut (requires `just`):

```bash
just ci
```

## Commit Conventions

- Use clear, imperative commit messages.
- Keep one logical change per commit when possible.
- Explain behavior changes in the commit body if needed.

## Pull Request Checklist

- [ ] Code is formatted with rustfmt.
- [ ] Tests pass locally.
- [ ] Clippy passes with `-D warnings`.
- [ ] README/docs updated when behavior changes.

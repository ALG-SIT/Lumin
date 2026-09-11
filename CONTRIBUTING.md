# Contributing to Lumin

This guide is the canonical contributor workflow for Lumin. It covers GitHub Flow, branch naming, commit conventions, pull requests, quality gates, and issue tracking.

## Prerequisites

- **Bun** 1.x (package manager and runtime)
- **Rust** 1.77 or later
- Tauri system dependencies for your platform (see `README.md`)

## Getting started

```bash
bun install
cargo check --manifest-path src-tauri/Cargo.toml
bun run dev        # Vite frontend only
bun run tauri dev  # Full Tauri desktop app
```

## Development workflow (GitHub Flow, mandatory)

1. **Never push directly to `main`.** All changes must go through a pull request.
2. Create a short-lived branch from the latest `main`:
   ```bash
   git checkout main && git pull origin main
   git checkout -b feat/<scope>
   ```
3. Make atomic, reviewable commits on the branch.
4. Push the branch and open a PR.
5. Make sure every quality gate passes before requesting review.
6. Merge only via GitHub (squash or merge commit on the PR page). Do not run `git push origin main` or merge locally into `main`.
7. Delete the branch after merge.

### Branch naming

Use one prefix and a short, descriptive scope:

| Prefix | Use for |
|--------|---------|
| `feat/<scope>` | New features or user-visible behavior |
| `fix/<scope>` | Bug fixes |
| `chore/<scope>` | Tooling, dependencies, maintenance |
| `docs/<scope>` | Documentation only |
| `test/<scope>` | Tests, test infrastructure |
| `spike/<scope>` | Experiments or time-boxed research |

Examples: `feat/hint-generation`, `fix/peer-reconnection`, `docs/contributing`, `chore/update-tauri`.

### Atomic commits

Each commit should contain one logical change. A good commit is easy to review, easy to revert, and easy to understand in `git blame`.

- Do not mix unrelated fixes in the same commit.
- Do not commit generated build artifacts.
- If a commit fails or hooks reject it, fix the problem and create a new commit. Do not amend the failed commit.

### Conventional commit prefixes

Use one of these prefixes in commit messages and PR titles:

- `feat:` — new feature or behavior
- `fix:` — bug fix
- `chore:` — maintenance, tooling, or dependencies
- `docs:` — documentation changes
- `test:` — tests or test helpers

A message looks like this:

```text
feat(analysis): add concept tag to AnalysisEvent
```

## Pull requests

Every branch must have a PR. Use the template in `.github/pull_request_template.md`, which asks for:

- **WHAT** — what changed
- **WHY** — why the change is needed
- **HOW** — the approach taken
- **VERIFICATION** — how you confirmed it works
- **RISK** — what could go wrong or what reviewers should watch for

PR titles must use a conventional prefix. Example: `feat: add session history persistence`.

## Merge policy

- Merge only through GitHub. Choose **Squash and merge** or **Create a merge commit** depending on whether the branch history is worth preserving.
- Do not merge directly into `main` from your local machine.
- Do not push directly to `main`.
- Update your branch with `git fetch origin && git merge origin/main` or `git rebase origin/main` when `main` has moved ahead.

### Branch protection

This project does not allow direct pushes to `main`. Enforcement is handled in the repository settings, but the rule is simple: all changes go through a reviewed PR.

## Quality gates

Run these commands after every task and make sure they pass before you open a PR:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
bun run build
```

Rules:

- `cargo check` must finish without errors.
- `cargo clippy -- -D warnings` must finish without warnings. Fix with `cargo clippy --fix --allow-dirty` or by hand.
- `cargo fmt -- --check` must exit 0. Run `cargo fmt --manifest-path src-tauri/Cargo.toml` before committing.
- `bun run build` must pass. It runs TypeScript compilation and Vite production build.
- If your change only touches files outside `src-tauri/`, the `cargo` steps may be skipped, but `bun run build` is still required.

CI enforces the same checks. A PR with failing checks will not be merged.

## Issue and milestone tracking

- Use `.github/ISSUE_TEMPLATE/task.md` for new work.
- Use `.github/ISSUE_TEMPLATE/bug.md` for bug reports.
- Group related issues and PRs under a GitHub milestone when they belong to the same release or iteration.
- Use labels such as `area:frontend`, `area:rust`, `area:docs`, `priority:high`, and `good first issue` where they help.

## Documentation

Update `README.md` or `CONTRIBUTING.md` when the workflow, quality gates, or project setup changes.

## References

- `README.md` — setup, architecture, and user-facing documentation
- `.github/pull_request_template.md` — PR structure
- `.github/ISSUE_TEMPLATE/task.md` — task issue structure

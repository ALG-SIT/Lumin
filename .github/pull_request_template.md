# WHAT

What changed in this PR?

# WHY

Why is this change needed?

# HOW

What approach did you take?

# VERIFICATION

How did you confirm it works? Include commands, tests, or manual checks.

# RISK

What could go wrong? What should reviewers watch for?

# Checklist

- [ ] `cargo check --manifest-path src-tauri/Cargo.toml` passes
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` passes
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passes
- [ ] `bun run build` passes
- [ ] Branch is up to date with `main`
- [ ] PR title uses a conventional prefix (`feat:`, `fix:`, `chore:`, `docs:`, `test:`)

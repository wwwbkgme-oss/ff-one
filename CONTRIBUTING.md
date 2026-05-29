# Contributing to ForgeFabrik

Thank you for your interest in contributing. This document explains how the project is structured,
what rules govern each layer, and how to get a change from idea to merged PR.

---

## Table of Contents

1. [Layer Rules](#layer-rules)
2. [Development Setup](#development-setup)
3. [Coding Standards](#coding-standards)
4. [Adding a New Crate](#adding-a-new-crate)
5. [Tests](#tests)
6. [Branching and Commits](#branching-and-commits)
7. [Pull Request Checklist](#pull-request-checklist)

---

## Layer Rules

The BKG three-layer model is the project's single most important constraint.
Every file you touch must respect the dependency direction:

```
foundation  ←  domain  ←  runtime
```

Arrows point in the direction of allowed imports.

| Layer | Allowed | Forbidden |
|---|---|---|
| `foundation/` | Types, traits, error enums, events, serialisation | HTTP, databases, I/O, business logic |
| `domain/` | Business logic, reducers, deterministic simulations | REST servers, CLI, non-deterministic side-effects |
| `runtime/` | HTTP, I/O, processes, plugins, logging, scheduling | Defining domain state outside sanctioned pipelines |
| `plugins/` | `cdylib` implementations of domain concepts | Direct access to `runtime` internals |

**If you are unsure which layer a new type or function belongs to, apply this decision tree:**

```
Is it a fundamental building block (type / trait / error)?  →  foundation/
Is it business logic (deterministic, no I/O)?               →  domain/
Does it touch the outside world (HTTP / processes / files)?  →  runtime/
Is it a runtime-loaded extension?                           →  plugins/
```

---

## Development Setup

### Prerequisites

- Rust stable toolchain (≥ 1.78): `rustup update stable`
- `cargo` in `PATH`

### Build everything

```bash
cargo build --workspace
```

### Run the server locally

```bash
cargo run -p cli -- serve --seed 42 --port 8080
```

### Spawn an agent (second terminal)

```bash
cargo run -p cli -- spawn --name "Alice" --kind claude
```

### Environment variables

| Variable | Purpose |
|---|---|
| `ANTHROPIC_API_KEY` | Required when using `claude` agent kind |
| `OPENAI_API_KEY` | Required when using `opencode` or `codex` agent kinds |
| `RUST_LOG` | Log filter, e.g. `forgefabrik=debug` |

---

## Coding Standards

- **Determinism in `domain/`**: Functions inside domain crates must be deterministic.
  Do not use `Utc::now()` for anything that affects world state. Wall-clock time may appear
  in log-only or `TimestampedEvent` fields, never in reducers.
- **Events are truth**: All state mutations flow through `WorldEvent`. A reducer that mutates
  state without emitting an event is a bug.
- **Replay-safety**: The same sequence of events must always produce the same state hash.
- **No `unwrap()` in library code**: Use `?` and propagate `FfError`. Reserve `unwrap()` for
  test helpers and clearly documented invariants.
- **Naming**: Crate names are concept names — no product prefix, no company name.
  Types live in `foundation/types`; traits live in `foundation/contracts`.
- **Async traits**: Use `async-trait` for all `trait` definitions that include `async fn`.

---

## Adding a New Crate

1. Create the directory and a minimal `Cargo.toml` that inherits from `[workspace.package]`.
2. Add the path to the `[workspace] members` list in the root `Cargo.toml`.
3. If other crates need to depend on it, add a path alias under `[workspace.dependencies]`.
4. Write at least one unit test before opening a PR.
5. Add an entry to the appropriate table in `ARCHITECTURE.md`.

---

## Tests

Run the full test suite:

```bash
cargo test --workspace
```

The project currently has **14 tests** spread across seven crates. Every new crate must ship
with at least one test that covers its core invariant (e.g., determinism, state transition,
consensus result).

Test naming convention:

```
<what_is_tested>_<expected_outcome>

# Examples
same_seed_same_hash
wallet_transfer_ok
insufficient_funds_error
```

---

## Branching and Commits

- Branch off `main`.
- Branch names: `<type>/<short-description>` — e.g. `feat/persistence-layer`, `fix/sandbox-memory-limit`.
- Commit messages: imperative mood, present tense — `Add rocksdb persistence for WorldState`.
- Keep commits atomic: one logical change per commit.
- Squash WIP commits before opening a PR.

---

## Pull Request Checklist

Before requesting a review, confirm every item below:

- [ ] `cargo build --workspace` passes
- [ ] `cargo test --workspace` passes — no new failures
- [ ] `cargo clippy --workspace -- -D warnings` is clean
- [ ] New code respects the layer dependency rules
- [ ] Any new domain logic is deterministic and covered by a test
- [ ] `ARCHITECTURE.md` updated if a new crate or trait was added
- [ ] No secrets, API keys, or personal data in the diff

# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records for ForgeFabrik.

An ADR documents a significant architectural decision: what was decided,
why it was decided, and what the consequences are.

## Format

Each ADR is a single Markdown file:

```
docs/adr/NNNN-short-title.md
```

Where `NNNN` is a zero-padded four-digit sequence number.

## Status values

| Status | Meaning |
|---|---|
| `proposed` | Under discussion, not yet accepted |
| `accepted` | Decision made, implementation may be pending |
| `superseded by ADR-NNNN` | Replaced by a later decision |
| `deprecated` | No longer relevant |

## Index

| ADR | Title | Status |
|---|---|---|
| [ADR-0001](0001-driver-plugin-boundary.md) | Driver / Plugin boundary | accepted |

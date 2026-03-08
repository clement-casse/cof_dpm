# General guidelines

This document captures code conventions for the cof-dpm project. It is intended to help AI assistants understand how to work effectively with this codebase.

## General conventions

### Correctness over convenience

- Model the full error space—no shortcuts or simplified error handling.
- Handle all edge cases, including race conditions, signal timing, and platform differences.
- Use the type system to encode correctness constraints.
- Prefer compile-time guarantees over runtime checks where possible.

### Pragmatic incrementalism

- "Not overly generic"—prefer specific, composable logic over abstract frameworks.
- Evolve the design incrementally rather than attempting perfect upfront architecture.
- When uncertain, explore and iterate; cof-dpm is an ongoing exploration of what microservices in async rust should be.

### Production-grade engineering

- Use type system extensively: newtypes, builder patterns, type states, lifetimes.
- Use message passing to avoid data races.
- Test comprehensively, including edge cases, race conditions, and stress tests.
- Pay attention to what facilities already exist for testing, and aim to reuse them.
- Getting the details right is really important!

### Documentation

- Use inline comments to explain "why," not just "what".
- Module-level documentation should explain purpose and responsibilities.
- **Always** use periods at the end of code comments.
- **Never** use title case in headings and titles. Always use sentence case.
- Always use the Oxford comma.

## Code style

### Rust edition and formatting

- Use Rust 2024 edition.
- Use `rustfmt` with the provided configuration in `.rustfmt.toml`.

### Type system patterns

- Use newtypes to encode domain-specific constraints.
- Use builder patterns to encode complex initialization logic and optional parameters.
- Use lifetimes to encode borrowing relationships.
- Use type states to encode state transitions.

### Error handling

- Use `thiserror` for error types with `#[derive(Error)]`.
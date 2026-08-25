# Capability Map: scheme-rs Interpreter Improvements

| Module id | Responsibility | Depends on |
|---|---|---|
| `codequality` | Edition 2021, remove `tuplet!` macro, replace `&'static str` errors with typed errors | — |
| `bugfixes` | Fix unary minus, quote shorthand for lists, division-by-zero error, remove panics in user paths | `codequality` (error types needed for new error messages) |
| `modules` | Incrementally split `lib.rs` into `types.rs`, `parser.rs`, `eval.rs`, `builtins.rs`, `env.rs` | `codequality` (clean code is easier to extract), `bugfixes` (don't split buggy code) |

Build order: `codequality` → `bugfixes` → `modules`

## Rationale

- **`codequality` first**: Error types and macro removal are foundational — every subsequent change benefits from typed errors and clean destructuring. Edition 2021 is a prerequisite for modern Rust idioms.
- **`bugfixes` second**: Fix bugs on the cleaned-up codebase. Don't split buggy code into modules — fix first, then split.
- **`modules` last**: Incremental extraction. Start with the most self-contained piece (types), grow from there. The user explicitly wants "simple to complex" — we extract one module at a time, keeping the build green between extractions.

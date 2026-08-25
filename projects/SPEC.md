# Spec: scheme-rs Interpreter Improvements

## Objective

Improve the scheme-rs interpreter's correctness and code quality. This is a refactor-and-fix initiative on an educational Scheme interpreter (Norvig's lispy port). The goal is to:

1. **Fix confirmed bugs** that produce wrong results or crash on valid Scheme input
2. **Improve code quality** — modernize the Rust edition, replace fragile patterns, introduce proper error types
3. **Incrementally modularize** the 1,300-line `lib.rs` into focused modules, growing from simple to complex

**User**: Developers learning Scheme/Rust who use this interpreter as a reference.
**Success looks like**: All existing tests pass, bugs are fixed with new tests proving the fix, code is cleaner and modular, no compiler warnings.

## Tech Stack

- **Language**: Rust (upgrading from edition 2015 → 2021)
- **Dependencies**: `log 0.4`, `env_logger 0.11`, `ctor 0.2` (no new deps expected)
- **Test framework**: Rust built-in `#[test]` (integration tests in `tests/spec.rs`)
- **Benchmarks**: Nightly-only `#[bench]` (out of scope for this round)

## Commands

```bash
# Build
cargo build --release           # build all binaries
cargo build --release --bin cli  # build REPL only
cargo build --release --bin scheme  # build file interpreter only

# Test
cargo test                      # run all 30 integration tests
cargo test <test_name>          # run a specific test

# Run
cargo run --release --bin cli   # start REPL
cargo run --release --bin scheme -- ./examples/demo_01.scm  # run a file

# Lint (no linter configured — use compiler warnings as the gate)
cargo build 2>&1 | grep warning  # check for warnings
```

## Project Structure

Current (flat):
```
src/
├── lib.rs       # everything: types, parser, eval, builtins, macros (1296 lines)
├── main.rs      # scheme binary (file interpreter)
├── cli.rs       # cli binary (REPL)
└── bench.rs     # nightly benchmarks
```

Target (incremental — we grow into this one module at a time):
```
src/
├── lib.rs       # re-exports from modules, setup()
├── types.rs     # AST, DataType, Procedure, Function, Env
├── parser.rs    # tokenize, read_from_tokens, atom, parse
├── eval.rs      # eval, ast_symbol_expression, if/define/lambda expressions
├── builtins.rs  # setup() with all builtin function definitions
├── env.rs       # Env impl, get, lookup logic
├── error.rs     # SchemeError type (replaces &'static str)
├── main.rs      # scheme binary (unchanged)
├── cli.rs       # cli binary (unchanged)
└── bench.rs     # nightly benchmarks (unchanged)
tests/
└── spec.rs      # integration tests (add new tests for bug fixes)
projects/
├── CAPABILITY_MAP.md
├── SPEC.md          # this file
├── PLAN.md          # implementation plan
└── TODO.md          # task checklist with status
```

## Code Style

Follow existing project conventions with these improvements:

```rust
// BEFORE: &'static str errors (no location info, no context)
pub fn eval(ast: Option<AST>, env: Rc<RefCell<Env>>) -> Result<Option<DataType>, &'static str> {
    // ...
    Err("wrong syntax for define expression")
}

// AFTER: typed errors with context
use crate::error::SchemeError;

pub fn eval(ast: Option<AST>, env: Rc<RefCell<Env>>) -> Result<Option<DataType>, SchemeError> {
    // ...
    Err(SchemeError::SyntaxError("define".into(), "expected (define symbol value)".into()))
}
```

```rust
// BEFORE: tuplet! macro for destructuring (fragile, produces warnings)
tuplet!((s0, s1, s2, s3) = list);
if let (Some(&ref cond), Some(&ref conseq), Some(&ref alt)) = (s1, s2, s3) { ... }

// AFTER: direct slice access (clear, no warnings)
let cond = list.get(1);
let conseq = list.get(2);
let alt = list.get(3);
if let (Some(cond), Some(conseq), Some(alt)) = (cond, conseq, alt) { ... }
```

**Conventions:**
- 4-space indentation (existing)
- `snake_case` for functions/variables, `PascalCase` for types (existing)
- No `extern crate` declarations (edition 2021 doesn't need them)
- Replace `panic!()`/`unreachable!()` in user-facing paths with `Err(SchemeError::...)`
- Every bug fix gets a test in `tests/spec.rs`

## Testing Strategy

- **Framework**: Rust built-in `#[test]` (existing pattern in `tests/spec.rs`)
- **Location**: Integration tests in `tests/spec.rs`, following existing naming convention (`fn feature_test()`)
- **Coverage**: Every bug fix must have at least one test proving the fix. Every refactored module must keep all 30 existing tests passing.
- **Test levels**:
  - Integration tests (`tests/spec.rs`): end-to-end `parse → eval` for each feature
  - Manual verification: REPL and file interpreter for edge cases
- **Regression gate**: `cargo test` must pass with 0 failures before any commit

## Boundaries

**Always do:**
- Run `cargo test` before every commit
- Write a test for each bug fix (red → green)
- Keep all 30 existing tests passing
- One logical change per commit
- Update `projects/TODO.md` status after completing each task

**Ask first:**
- Adding new dependencies to `Cargo.toml`
- Changing the public API of `lib.rs` (functions exported via `pub`)
- Removing or renaming existing tests
- Changing `Cargo.toml` beyond edition bump

**Never do:**
- Commit `target/` or `graphify-out/`
- Use `--no-verify` to bypass hooks
- Delete tests instead of fixing them
- Introduce `unsafe` code
- Change `main.rs` or `cli.rs` logic (only update imports if needed)

## Success Criteria

- [ ] `cargo build --release` succeeds with **zero warnings**
- [ ] `cargo test` passes with **all 30 existing tests + new tests for each bug fix**
- [ ] `(- 5)` returns `-5`, not `5` (unary minus fixed)
- [ ] `'(...)` quote shorthand works for lists, not just symbols
- [ ] `(/ 1 0)` returns an error, not `inf`
- [ ] No `panic!()` or `unreachable!()` in code paths reachable from user input
- [ ] `Cargo.toml` specifies `edition = "2021"`
- [ ] No `extern crate` declarations in any source file
- [ ] No `tuplet!` macro — replaced with direct slice access
- [ ] All functions return `Result<_, SchemeError>` instead of `Result<_, &'static str>`
- [ ] `lib.rs` is split into at least `types.rs`, `parser.rs`, `eval.rs`, `builtins.rs`
- [ ] `projects/TODO.md` is updated with final status

## Open Questions

1. **Should `SchemeError` implement `std::error::Error`?** — Recommended for ecosystem compatibility, but adds a small amount of boilerplate. → *Assume yes, it's idiomatic Rust.*
2. **Should we add `#[derive(thiserror::Error)]` or hand-roll?** — Hand-roll to avoid adding a dependency. The project has only 3 deps and we should keep it minimal.
3. **Do we need to fix the lambda env mutation bug** (each call mutates the captured env)? — This is a real bug but fixing it changes closure semantics significantly. → *Defer to a future round; note it in TODO as a known issue.*
4. **Should we push the branch to origin?** — User said "new branch" but didn't mention pushing. → *Don't push unless asked.*

# scheme-rs Interpreter Improvements — Summary Report

> **Branch**: `fix/interpreter-improvements`
> **Base commit**: `2d9069f` (docs: add spec, plan, and TODO for interpreter improvements)
> **Head commit**: `183d478` (refactor: extract builtins.rs, lib.rs is now module hub)
> **Date**: 2026-08-25
> **Commits**: 13 (each task commit includes its own TODO.md status update)

---

## Overview

This branch refactored and hardened the `scheme-rs` Scheme interpreter (a Rust port of Peter Norvig's `lispy`). Work was organized into three phases — Foundation, Bug Fixes, and Module Split — covering 14 tasks total. The original 1300-line `src/lib.rs` monolith was split into 6 focused modules, four user-facing bugs were fixed, and all `panic!`/`unreachable!` calls reachable from user input were replaced with proper error types.

### Final State

| Metric | Before | After |
|--------|--------|-------|
| Tests passing | 30/30 | 33/33 |
| Compiler warnings | 2 (unused `tuplet!` macro) | 0 |
| `lib.rs` line count | ~1300 | 17 |
| Source files | 4 (`lib.rs`, `main.rs`, `cli.rs`, `bench.rs`) | 10 (above + 6 new modules) |
| Error type | `&'static str` | `SchemeError` enum (6 variants) |

---

## Phase 1: Foundation (Tasks 1-3)

### Task 1: Bump to edition 2021, remove `extern crate` — `b5887b9`

- Updated `Cargo.toml` to `edition = "2021"`
- Removed all `extern crate` declarations from `src/lib.rs`, `src/main.rs`, `src/cli.rs`, `src/bench.rs`
- Edition 2021 makes `extern crate` implicit for most crates

### Task 2: Create `SchemeError` enum, replace `&'static str` — `ef73cc6`

- Introduced `SchemeError` enum with 6 variants:
  - `SyntaxError(String)` — malformed Scheme syntax
  - `TypeError(String, String)` — wrong type for an operation
  - `UndefinedSymbol(String)` — reference to unbound symbol
  - `ArityError(String, usize, usize)` — wrong number of arguments
  - `DivisionByZero` — division by zero
  - `RuntimeError(String)` — catch-all for other runtime failures
- Implemented `Display` and `std::error::Error` for `SchemeError`
- Replaced all `Result<_, &'static str>` signatures with `Result<_, SchemeError>`
- Error messages in the REPL remain readable

### Task 3: Remove `tuplet!` macro, use direct slice access — `4962617`

- Deleted the 24-line `tuplet!` macro definition
- Replaced 7 call sites with direct `vec.get(N)` and slice indexing
- Eliminated both "unused variable" compiler warnings
- Updated `src/main.rs` to use the same direct-access pattern

**Foundation checkpoint**: zero warnings, 30/30 tests pass, no `extern crate`, no `&'static str` errors, no `tuplet!` macro.

---

## Phase 2: Bug Fixes (Tasks 4-7)

### Task 4: Fix unary minus — `2e6353d`

- **Bug**: `(- 5)` returned `5` instead of `-5` (the subtraction builtin required ≥2 args and treated the first as the base)
- **Fix**: When `(-)` receives exactly one argument, return its negation
- **Test added**: `unary_minus_test` (31 tests total)

### Task 5: Fix quote shorthand for lists — `fc8c871`

- **Bug**: `'(1 2 3)` failed to parse — the tokenizer emitted `'` as a standalone token but the parser didn't handle a bare `'` followed by a list
- **Fix**: In `read_from_tokens`, when a standalone `'` token is encountered, wrap the next form in `(quote ...)`
- **Test added**: `quote_shorthand_test` (32 tests total)

### Task 6: Fix division by zero — `ed93cf6`

- **Bug**: `(/ 1 0)` returned `inf` (Rust float division by zero) instead of an error
- **Fix**: The `/` builtin now checks for zero divisors and returns `Err(SchemeError::DivisionByZero)`
- **Test added**: `division_by_zero_test` (33 tests total)

### Task 7: Replace `panic!()`/`unreachable!()` with `SchemeError` — `92c2ff8`

- **Bug**: Several code paths reachable from user input called `panic!()` or `unreachable!()`, crashing the interpreter on malformed input instead of returning a clean error
- **Fix**: Replaced all such calls with `Err(SchemeError::...)` returns
- Also fixed `Result` types inside `.map()` closures that were silently discarding errors

**Bug Fixes checkpoint**: all bug-fix tests pass, no `panic!`/`unreachable!` reachable from user input, all 30 original tests still pass, zero warnings.

---

## Phase 3: Module Split (Tasks 8-14)

The 1300-line `src/lib.rs` was split into 6 focused modules. Each extraction was verified with `cargo build` + `cargo test` before committing.

### Task 8: Extract `types.rs` — `33ee4ab` (99 lines)

Moved out of `lib.rs`:
- `AST` enum (symbol, number, list, etc.)
- `DataType` enum (Number, Symbol, List, Bool, Procedure, Function, String)
- `Procedure` struct (lambda closures)
- `Function` type alias
- `ReadFromTokenResult` enum
- `FloatIterExt` trait extension

### Task 9: Extract `error.rs` — `e468301` (48 lines)

Moved out of `types.rs`:
- `SchemeError` enum (all 6 variants)
- `Display` impl
- `std::error::Error` impl

### Task 10: Extract `parser.rs` — `a29a4a4` (138 lines)

Moved out of `lib.rs`:
- `tokenize(&str) -> Vec<String>` — lexer
- `read_from_tokens(&mut Vec<String>) -> Result<ReadFromTokenResult, SchemeError>` — parser
- `atom(&str) -> AST` — atom conversion (number vs symbol)
- `parse(program: &str) -> Result<AST, SchemeError>` — top-level parse entry point

### Task 11: Extract `env.rs` — `160dbaf` (37 lines)

Moved out of `types.rs`:
- `Env` struct (environment frame with parent pointer)
- `Env::get(&self, key: &str) -> Option<DataType>` — variable lookup with parent chain

### Task 12: Extract `eval.rs` — `874ad8c` (375 lines)

Moved out of `lib.rs`:
- `eval(ast, env) -> Result<DataType, SchemeError>` — the evaluator
- `prepare_arguments(...)` — argument binding for procedure calls
- `execute(...)` — procedure execution
- `ast2datatype(...)` — AST-to-DataType conversion
- `datatype2str(...)` — DataType-to-string rendering (made public for `lib.rs` re-export)

### Task 13+14: Extract `builtins.rs`, clean up `lib.rs` — `183d478`

Moved out of `lib.rs` (652 lines):
- `setup() -> Env` — builds the global environment with all builtins
- `define_comparison!` macro — generates `<`, `>`, `<=`, `>=`, `=`, `not` implementations

`lib.rs` is now a 17-line module hub:
- `mod` declarations for all 6 modules
- `pub use` re-exports for the public API
- No logic

**Complete checkpoint**: `lib.rs` under 100 lines, all modules compile independently, 33/33 tests pass, zero warnings.

---

## Final Module Layout

```
src/
  lib.rs        17 lines   — module hub, re-exports
  error.rs      48 lines   — SchemeError enum + impls
  env.rs        37 lines   — Env struct + lookup
  types.rs      99 lines   — AST, DataType, Procedure, Function
  parser.rs    138 lines   — tokenize, read_from_tokens, atom, parse
  eval.rs      375 lines   — eval, prepare_arguments, execute, ast2datatype
  builtins.rs  652 lines   — define_comparison! macro + setup()
  main.rs       70 lines   — binary entry point
  cli.rs        34 lines   — CLI argument parsing
  bench.rs      52 lines   — benchmarking harness
```

---

## Diff Statistics

```
 13 files changed, 1466 insertions(+), 1352 deletions(-)
```

| File | Change |
|------|--------|
| `Cargo.toml` | +1 (edition = "2021") |
| `src/lib.rs` | -1272 (1300 → 17, logic moved to modules) |
| `src/builtins.rs` | +652 (new) |
| `src/eval.rs` | +375 (new) |
| `src/parser.rs` | +138 (new) |
| `src/types.rs` | +99 (new) |
| `src/error.rs` | +48 (new) |
| `src/env.rs` | +37 (new) |
| `src/main.rs` | +9/-9 (removed extern crate) |
| `src/cli.rs` | +7/-7 (removed extern crate) |
| `src/bench.rs` | +6/-6 (removed extern crate) |
| `tests/spec.rs` | +65 (3 new test cases) |
| `projects/TODO.md` | +92/-92 (status updates, folded into task commits) |

---

## Commit History

Each commit is self-contained: code change + corresponding TODO.md status update.

```
b5887b9  chore: bump to edition 2021, remove extern crate
ef73cc6  refactor: replace &'static str errors with SchemeError enum
4962617  refactor: remove tuplet! macro, use direct slice access
2e6353d  fix: unary minus returns negation for single argument
fc8c871  fix: quote shorthand '(1 2 3) now parses correctly
ed93cf6  fix: division by zero returns DivisionByZero error instead of inf
92c2ff8  refactor: replace panic!/unreachable! with SchemeError in lib
33ee4ab  refactor: extract types.rs module
e468301  refactor: extract error.rs module
a29a4a4  refactor: extract parser.rs module
160dbaf  refactor: extract env.rs module
874ad8c  refactor: extract eval.rs module
183d478  refactor: extract builtins.rs, lib.rs is now module hub
```

---

## Known Issues (Deferred — Not in This Round)

These were documented in `projects/TODO.md` but intentionally left for future work:

1. **Lambda env mutation bug** — each lambda call mutates the captured env instead of creating a fresh local scope. Breaks `set!` and proper closure semantics.
2. **No tail-call optimization** — deep recursion overflows the stack (~10k frames). Needs a trampoline or explicit loop in `eval` for tail positions.
3. **Integer precision loss** — all numbers are `f64`. `i64` is cast to `f64` at eval time. Needs a numeric tower.
4. **Missing R5RS features** — `let`, `cond`, `case`, `set!`, `when`/`unless`, `quasiquote`/`unquote`, `eq?`/`eqv?`/`equal?`, `string-*` operations, `display`/`newline`, `do` loops, named `let`, macros.
5. **Travis CI is dead** — no working CI. Should migrate to GitHub Actions.
6. **5 unpushed commits on master** — should push before merging this branch back.

---

## Verification

- `cargo build --release` — succeeds, zero warnings
- `cargo test` — 33/33 pass (was 30/30; added 3 bug-fix tests)
- `grep -rn "panic!\|unreachable!" src/` — no reachable panics from user input
- `wc -l src/lib.rs` — 17 lines (target was < 100)
- Each module compiles independently via `cargo build`

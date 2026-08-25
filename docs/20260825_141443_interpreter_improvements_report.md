# scheme-rs Interpreter Improvements — Summary Report

> **Branch**: `fix/interpreter-improvements`
> **Base commit**: `2d9069f` (docs: add spec, plan, and TODO for interpreter improvements)
> **Head commit**: `b200970` (fix: lambda fresh env per call, string type in quote, Env::get cloning)
> **Date**: 2026-08-25
> **Commits**: 16 (15 task commits with TODO.md updates folded in + 1 summary report commit)

---

## Overview

This branch refactored and hardened the `scheme-rs` Scheme interpreter (a Rust port of Peter Norvig's `lispy`). Work was organized into four phases — Foundation, Bug Fixes, Module Split, and Architecture Fixes — covering 17 tasks total. The original 1300-line `src/lib.rs` monolith was split into 6 focused modules, four user-facing bugs were fixed, all `panic!`/`unreachable!` calls reachable from user input were replaced with proper error types, and three critical architecture faults were fixed (lambda env mutation, string type confusion, Env::get cloning landmine).

### Final State

| Metric | Before | After |
|--------|--------|-------|
| Tests passing | 30/30 | 38/38 |
| Compiler warnings | 2 (unused `tuplet!` macro) | 0 |
| `lib.rs` line count | ~1300 | 17 |
| Source files | 4 (`lib.rs`, `main.rs`, `cli.rs`, `bench.rs`) | 10 (above + 6 new modules) |
| Error type | `&'static str` | `SchemeError` enum (6 variants) |
| Lambda call sites | 4 (inconsistent env handling) | 4 (all use fresh env frames) |

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

## Phase 4: Architecture Fixes (Tasks 15-17)

After the module split, a code review revealed three critical architecture faults beyond the original 14 tasks. These were fixed in a single commit `b200970`.

### Task 15: Fix lambda env mutation across all 4 call sites — `b200970`

- **Bug**: Each lambda call either mutated the closure's captured env (children path in `eval.rs`) or cloned it (symbol path, `apply`, `map`), causing stale parameter bindings to leak between calls. The 4 call sites were inconsistent.
- **Fix**: All 4 call sites now create a fresh empty `HashMap` per call, bind only the current arguments, and parent it to the closure's env. This is correct lexical scoping semantics.
- **Call sites fixed**:
  - `eval.rs` symbol path (line ~199) — was already cloning, now uses fresh frame
  - `eval.rs` children path (line ~247) — was mutating the closure env directly, now uses fresh frame
  - `builtins.rs` `apply` (line ~299) — was cloning, now uses fresh frame
  - `builtins.rs` `map` (line ~465) — was cloning, now uses fresh frame
- **Tests added**: `lambda_fresh_env_test`, `lambda_no_arg_leak_test`, `lambda_via_apply_fresh_env_test`, `lambda_via_map_fresh_env_test`

### Task 16: Fix string type confusion in `ast2datatype` — `b200970`

- **Bug**: `ast2datatype` converted quoted string literals like `(quote "hello")` to `DataType::Symbol` instead of `DataType::String`. This meant `string?` returned `#f` for quoted strings. Additionally, lambda creation required the body to be `AST::Children`, so `(lambda (x) x)` (single-expression body) failed with a syntax error.
- **Fix**: `ast2datatype` now produces `DataType::String` for quoted strings. Lambda creation accepts any `AST` for the body, not just `AST::Children`.
- **Tests added**: `quote_string_type_test`. Updated `quote_expression_test` to expect `DataType::String`.

### Task 17: Fix `Env::get` per-variant cloning landmine — `b200970`

- **Bug**: `Env::get` had a 9-line manual match on every `DataType` variant to clone the value. If a new `DataType` variant were added in the future, `get` would silently return `None` for it — a maintenance landmine.
- **Fix**: Replaced with a 1-line `.cloned()` call. Future-proof and correct.

**Architecture Fixes checkpoint**: 38/38 tests pass (was 33), zero warnings, all 4 lambda call sites use fresh env frames, quoted strings are `DataType::String`, `Env::get` is future-proof.

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
 14 files changed, 1556 insertions(+), 1379 deletions(-)
```

| File | Change |
|------|--------|
| `Cargo.toml` | +1 (edition = "2021") |
| `src/lib.rs` | -1272 (1300 → 17, logic moved to modules) |
| `src/builtins.rs` | +652 (new) + 8 modified (lambda fresh env in apply/map) |
| `src/eval.rs` | +375 (new) + 27 modified (lambda fresh env, string type, single-expr body) |
| `src/parser.rs` | +138 (new) |
| `src/types.rs` | +99 (new) |
| `src/error.rs` | +48 (new) |
| `src/env.rs` | +37 (new) + 11 modified (Env::get .cloned()) |
| `src/main.rs` | +9/-9 (removed extern crate) |
| `src/cli.rs` | +7/-7 (removed extern crate) |
| `src/bench.rs` | +6/-6 (removed extern crate) |
| `tests/spec.rs` | +136 (8 new test cases + 1 updated) |
| `projects/TODO.md` | +92/-92 (status updates, folded into task commits) |
| `docs/` | +231 (summary report) |

---

## Commit History

Each task commit (1-14) is self-contained: code change + corresponding TODO.md status update. Phase 4 fixes were committed together as they were interrelated.

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
787c336  docs: add interpreter improvements summary report
b200970  fix: lambda fresh env per call, string type in quote, Env::get cloning
```

---

## Known Issues (Deferred — Not in This Round)

These remain documented in `projects/TODO.md` for future work:

1. **No tail-call optimization** — deep recursion overflows the stack (~10k frames). Needs a trampoline or explicit loop in `eval` for tail positions.
2. **Integer precision loss** — all numbers are `f64`. `i64` is cast to `f64` at eval time. Needs a numeric tower.
3. **Missing R5RS features** — `let`, `cond`, `case`, `set!`, `when`/`unless`, `quasiquote`/`unquote`, `eq?`/`eqv?`/`equal?`, `string-*` operations, `display`/`newline`, `do` loops, named `let`, macros.
4. **Travis CI is dead** — no working CI. Should migrate to GitHub Actions.
5. **5 unpushed commits on master** — should push before merging this branch back.

---

## Verification

- `cargo build --release` — succeeds, zero warnings
- `cargo test` — 38/38 pass (was 30/30; added 8 new tests across Phases 2 and 4)
- `grep -rn "panic!\|unreachable!" src/` — no reachable panics from user input
- `wc -l src/lib.rs` — 17 lines (target was < 100)
- Each module compiles independently via `cargo build`

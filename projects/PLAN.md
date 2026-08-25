# Implementation Plan: scheme-rs Interpreter Improvements

## Overview

A multi-phase effort to refactor, fix, and extend the scheme-rs Scheme
interpreter written in Rust. What began as a 3-phase, 14-task modernization
project grew into 10 phases and 43 tasks, covering code quality, bug fixes,
module architecture, R5RS feature implementation, tail-call optimization,
CI migration, numeric tower improvements, performance optimization, and
more R5RS features.

**Status**: All 10 phases complete. 57/57 tests pass. Zero compiler warnings.

## Architecture Decisions

1. **Edition 2021 first** — Unlocks modern Rust idioms (no `extern crate`, cleaner imports). Prerequisite for all other work. (Phase 1)

2. **Hand-rolled `SchemeError` enum** — Replaces `&'static str` throughout. No new dependencies. Implements `std::error::Error` + `Display`. Variants: `SyntaxError`, `TypeError`, `UndefinedSymbol`, `ArityError`, `DivisionByZero`, `RuntimeError`. (Phase 1)

3. **Remove `tuplet!` macro** — Replace with direct `vec.get(N)` calls. The macro was fragile, produced compiler warnings, and obscured intent. (Phase 1)

4. **Incremental module extraction** — Extract one module at a time (types → error → parser → env → eval → builtins). Build stays green between each extraction. `lib.rs` becomes a re-export hub. (Phase 3)

5. **Bug fixes get tests first** — TDD: write a failing test for each bug, then fix it, then verify the test passes. (Phase 2, ongoing)

6. **Trampoline-style eval loop for TCO** — Convert `eval` from recursive to `loop { ... }` with `continue` at tail positions. Non-tail positions still use recursive calls. (Phase 6)

7. **i64/f64 numeric split** — Replace single `Number(f64)` with `Integer(i64)` and `Float(f64)`. Cross-type equality via `PartialEq`. Promotion rules: integer arithmetic for integers, float promotion for mixed, division always returns float. (Phase 8)

8. **`Rc<AST>` for procedure bodies** — Makes body cloning O(1) instead of O(body size). Significant performance improvement. (Phase 9)

9. **Implicit begin for multi-expression bodies** — `lambda`, `let`, `when`, `unless` collect all expressions after fixed arguments and wrap in an implicit `begin`, with the last expression in tail position. (Phase 10)

10. **Named let as self-referencing procedure** — `(let loop ...)` creates a lambda, binds it in its own env for recursion, then calls it with initial values. (Phase 10)

## Phase Summary

| Phase | Name | Tasks | Tests | Key Deliverable |
|-------|------|-------|-------|-----------------|
| 1 | Foundation | 1-3 | 30 | Edition 2021, SchemeError, no tuplet! |
| 2 | Bug Fixes | 4-7 | 33 | Unary minus, quote shorthand, div-by-zero, no panics |
| 3 | Module Split | 8-14 | 33 | 6 modules extracted, lib.rs < 100 lines |
| 4 | Architecture Fixes | 15-17 | 38 | Fresh env per lambda call, string type fix, future-proof get |
| 5 | R5RS Features | 18-26 | 47 | let, cond, set!, when/unless, case, eq?/eqv?/equal?, display, strings, predicates |
| 6 | Tail-Call Optimization | 27-28 | 49 | Trampoline eval loop, 100k+ deep recursion |
| 7 | CI Migration | 29 | 49 | GitHub Actions replaces Travis CI |
| 8 | Numeric Tower | 30-33 | 51 | Integer/Float split with promotion rules |
| 9 | Examples & Performance | 34-37 | 52 | 5 examples, 6 benchmarks, ~2x speedup |
| 10 | Bug Fixes + R5RS | 38-43 | 57 | Multi-expr bodies, internal define, quasiquote, named let, do loops |

## Detailed Phase List

### Phase 1: Foundation (`codequality`) — ✅ Complete

- [x] Task 1: Bump to edition 2021, remove `extern crate`
- [x] Task 2: Create `SchemeError` enum, replace `&'static str`
- [x] Task 3: Remove `tuplet!` macro, use direct slice access

**Checkpoint**: Zero warnings, 30 tests pass, no `extern crate`, no `&'static str`.

### Phase 2: Bug Fixes (`bugfixes`) — ✅ Complete

- [x] Task 4: Fix unary minus (`(- 5)` → `-5`)
- [x] Task 5: Fix quote shorthand for lists (`'(1 2 3)`)
- [x] Task 6: Fix division by zero (`(/ 1 0)` errors, not `inf`)
- [x] Task 7: Replace `panic!()`/`unreachable!()` with `SchemeError`

**Checkpoint**: 33 tests pass, no panics reachable from user input.

### Phase 3: Module Split (`modules`) — ✅ Complete

- [x] Task 8: Extract `types.rs` (AST, DataType, Procedure, Function, Env)
- [x] Task 9: Extract `error.rs` (SchemeError)
- [x] Task 10: Extract `parser.rs` (tokenize, read_from_tokens, atom, parse)
- [x] Task 11: Extract `env.rs` (Env impl, get method)
- [x] Task 12: Extract `eval.rs` (eval, ast2datatype, datatype2str)
- [x] Task 13: Extract `builtins.rs` (setup, all builtins, define_comparison! macro)
- [x] Task 14: Clean up `lib.rs` to re-export hub (< 100 lines)

**Checkpoint**: 6 modules, lib.rs < 100 lines, 33 tests pass, zero warnings.

### Phase 4: Architecture Fixes (`archfix`) — ✅ Complete

- [x] Task 15: Fix lambda env mutation across all 4 call sites (eval symbol, eval children, apply, map)
- [x] Task 16: Fix string type confusion in `ast2datatype` (`(quote "hello")` → `DataType::String`)
- [x] Task 17: Fix `Env::get` per-variant cloning landmine (use `.cloned()`)

**Checkpoint**: Fresh env per lambda call, 38 tests pass (was 33).

### Phase 5: R5RS Features (`r5rs`) — ✅ Complete

> Spec: `projects/SPEC_R5RS.md`

**Special forms**:
- [x] Task 18: `let` — local bindings
- [x] Task 19: `cond` — multi-branch conditional with `else`
- [x] Task 20: `set!` — mutate existing binding
- [x] Task 21: `when` / `unless`
- [x] Task 22: `case` — key dispatch

**Builtins**:
- [x] Task 23: `eq?` / `eqv?` / `equal?`
- [x] Task 24: `display` / `newline`
- [x] Task 25: String operations (`string-length`, `string-append`, `string->symbol`, `symbol->string`)
- [x] Task 26: Type/number predicates + integer division (`boolean?`, `zero?`, `positive?`, `negative?`, `modulo`, `quotient`, `remainder`, `even?`, `odd?`)

**Checkpoint**: 47 tests pass (was 38, added 9).

### Phase 6: Tail-Call Optimization (`tco`) — ✅ Complete

> Spec: `projects/SPEC_TCO.md`

- [x] Task 27: Convert `eval` to a trampoline loop — tail positions use `continue` with reassigned `ast`/`env`
- [x] Task 28: Add deep recursion tests — 100k+ self-recursion and mutual recursion

**Checkpoint**: 100k+ deep recursion without stack overflow, 49 tests pass.

### Phase 7: CI Migration (`ci`) — ✅ Complete

- [x] Task 29: Migrate Travis CI to GitHub Actions — `.github/workflows/ci.yml`, delete `.travis.yml`

**Checkpoint**: GitHub Actions runs `cargo build` + `cargo test` on push/PR.

### Phase 8: Numeric Tower — i64/f64 Split (`numeric`) — ✅ Complete

> Spec: `projects/SPEC_NUMERIC.md`

- [x] Task 30: Split `DataType::Number` into `Integer(i64)` / `Float(f64)` with cross-type `PartialEq`
- [x] Task 31: Update `eval.rs` for Integer/Float handling
- [x] Task 32: Update `builtins.rs` with promotion rules (int+int=int, mixed=float, `/`=float, `eqv?` type-sensitive)
- [x] Task 33: Update tests for Integer/Float

**Checkpoint**: `42` → `Integer(42)`, `(+ 1 2.0)` → `Float(3.0)`, `(eqv? 1 1.0)` → `#f`, 51 tests pass.

### Phase 9: Examples, Benchmarks, and Performance (`perf`) — ✅ Complete

> Spec: `projects/SPEC_PERF.md`

**Examples**:
- [x] Task 34: Create 5 example `.scm` files (fact, fib, loop, list, closures) — also fixed parser comment + string tokenizer bugs

**Missing builtins**:
- [x] Task 35: Add `null?` builtin

**Benchmarks**:
- [x] Task 36: Expand benchmark suite to 6 benchmarks (fact20, fib25, tco_loop_100k, sum_to_1000, ackermann_2_8, list_ops)

**Performance**:
- [x] Task 37: Reduce eval loop overhead — 3 optimizations yielding ~2x speedup:
  1. `ast_option.take()` instead of `.clone()` — avoids cloning entire AST per iteration
  2. Move out of cloned Lambda — eliminates redundant body/params clone
  3. `Procedure.body` → `Rc<AST>` — makes env lookup clone O(1)

**Checkpoint**: 5 examples run, 6 benchmarks run, ~2x speedup (fact20 1.9x, fib25 2.1x, ackermann 2.4x), 52 tests pass.

### Phase 10: Bug Fixes + R5RS Features (`r5rs-2`) — ✅ Complete

> Spec: `projects/SPEC_PHASE10.md`

**Bug fixes**:
- [x] Task 38: Multi-expression bodies for `lambda`, `let`, `when`, `unless` — no more explicit `begin` required
- [x] Task 39: Internal `define` works in lambda/let bodies (automatic consequence of Task 38)

**R5RS features**:
- [x] Task 40: `quasiquote` / `unquote` / `unquote-splicing` with `` ` ``, `,`, `,@` shorthand syntax
- [x] Task 41: Named `let` — `(let loop ((i 0)) ...)` creates recursive procedure and calls it
- [x] Task 42: `do` loops — full R5RS syntax `(do ((var init step)...) (test result...) body...)`
- [x] Task 43: Update examples to remove `begin` workarounds, use named let and quasiquote

**Checkpoint**: 57 tests pass, 5 examples updated, zero warnings.

## Risks and Mitigations

| Risk | Impact | Mitigation | Outcome |
|------|--------|------------|---------|
| Error type change causes cascading type errors | Medium | Do it early (Task 2), fix all call sites in one commit | ✅ No issues |
| `tuplet!` removal breaks destructuring patterns | Medium | Replace one call site at a time, test after each | ✅ No issues |
| Module split introduces circular dependencies | Low | Extract in dependency order: types → parser → eval → builtins | ✅ No cycles |
| TCO conversion breaks existing semantics | High | Keep non-tail positions recursive, only convert true tail positions | ✅ All tests passed first try |
| Numeric split breaks all existing tests | High | Update tests in same commit, use script for bulk `Number(X.0)` → `Integer(X)` conversion | ✅ Script handled bulk, manual fixes for edge cases |
| Named let self-reference creates env cycle | Medium | Bind procedure in its own env before calling, use `Rc<RefCell<Env>>` | ✅ Works correctly |
| `Rc<AST>` change causes clone semantics shift | Low | Only `Procedure.body` uses `Rc`, other AST cloning unchanged | ✅ ~2x speedup, no regressions |

## Deferred Work (Not Planned)

These items are documented in `projects/R5RS_SUPPORT.md` and `projects/TODO.md` but have no tasks, specs, or phases:

- **Macros** (`define-syntax` / `syntax-rules`) — needs a full macro expander
- **`call/cc`** — needs continuation support, major architectural change
- **BigInt** — integers are i64 only, would need `num-bigint` dependency
- **Character type** — `#\a` syntax, `char?`, `char->integer`
- **Vectors** — `#(...)` syntax, vector operations
- **`let*` / `letrec`** — sequential and recursive binding forms
- **`and` / `or`** — short-circuit boolean operators
- **`define` function shorthand** — `(define (f x) ...)` syntax
- **Extended string library** — `string=?`, `substring`, `string->list`, etc.
- **Extended list library** — `reverse`, `member`, `assoc`, `list-ref`, `list-tail`
- **Math functions** — `sqrt`, `exp`, `log`, `sin`, `cos`, `tan`, `expt`, `floor`, `ceiling`, `round`, `truncate`
- **`error` procedure** — raising errors from Scheme code
- **`eval` procedure** — exposing eval to Scheme code
- **I/O ports** — `read`, `write`, `open-input-file`, etc.

## Related Documents

| Document | Purpose |
|----------|---------|
| `projects/SPEC.md` | Original specification (Phases 1-3) |
| `projects/SPEC_R5RS.md` | R5RS features spec (Phase 5) |
| `projects/SPEC_TCO.md` | Tail-call optimization spec (Phase 6) |
| `projects/SPEC_NUMERIC.md` | Numeric tower spec (Phase 8) |
| `projects/SPEC_PERF.md` | Performance spec (Phase 9) |
| `projects/SPEC_PHASE10.md` | Bug fixes + R5RS spec (Phase 10) |
| `projects/TODO.md` | Live task tracker (all phases) |
| `projects/CAPABILITY_MAP.md` | Interpreter capability map |
| `projects/R5RS_SUPPORT.md` | R5RS feature support audit (63/151 features, ~43%) |

# Implementation Plan: scheme-rs Interpreter Improvements

## Overview

Refactor and fix the scheme-rs interpreter in three phases: (1) modernize the codebase foundation, (2) fix confirmed bugs, (3) incrementally split into modules. Each phase builds on the previous one. All 30 existing tests must pass at every commit.

## Architecture Decisions

1. **Edition 2021 first** — Unlocks modern Rust idioms (no `extern crate`, cleaner imports). Prerequisite for all other work.
2. **Hand-rolled `SchemeError` enum** — Replaces `&'static str` throughout. No new dependencies. Implements `std::error::Error` + `Display`. Variants: `SyntaxError`, `TypeError`, `UndefinedSymbol`, `ArityError`, `DivisionByZero`, `RuntimeError`.
3. **Remove `tuplet!` macro** — Replace with direct `vec.get(N)` calls. The macro is fragile, produces compiler warnings, and obscures intent. Direct indexing is clearer and warning-free.
4. **Incremental module extraction** — Extract one module at a time (types → error → parser → env → eval → builtins). Build stays green between each extraction. `lib.rs` becomes a re-export hub + `setup()`.
5. **Bug fixes get tests first** — TDD: write a failing test for each bug, then fix it, then verify the test passes.

## Task List

### Phase 1: Foundation (`codequality`)

- [ ] Task 1: Bump to edition 2021, remove `extern crate` declarations
- [ ] Task 2: Create `SchemeError` enum, replace `&'static str` return types
- [ ] Task 3: Remove `tuplet!` macro, replace with direct slice access

### Checkpoint: Foundation
- [ ] `cargo build` with zero warnings
- [ ] All 30 tests pass
- [ ] No `extern crate` in any file
- [ ] No `&'static str` error types

### Phase 2: Bug Fixes (`bugfixes`)

- [ ] Task 4: Fix unary minus (`(- 5)` should return `-5`)
- [ ] Task 5: Fix quote shorthand for lists (`'(1 2 3)` should work)
- [ ] Task 6: Fix division by zero (`(/ 1 0)` should error, not return `inf`)
- [ ] Task 7: Replace `panic!()`/`unreachable!()` with `SchemeError` in user paths

### Checkpoint: Bug Fixes
- [ ] All bug-fix tests pass
- [ ] No `panic!()` or `unreachable!()` reachable from user input
- [ ] All 30 original tests still pass

### Phase 3: Module Split (`modules`)

- [ ] Task 8: Extract `types.rs` (AST, DataType, Procedure, Function, Env struct)
- [ ] Task 9: Extract `error.rs` (SchemeError — already created in Task 2, just move)
- [ ] Task 10: Extract `parser.rs` (tokenize, read_from_tokens, atom, parse)
- [ ] Task 11: Extract `env.rs` (Env impl, get method)
- [ ] Task 12: Extract `eval.rs` (eval, ast_symbol_expression, if/define/lambda/quote expressions)
- [ ] Task 13: Extract `builtins.rs` (setup() and all builtin definitions)
- [ ] Task 14: Clean up `lib.rs` to re-export from modules

### Checkpoint: Complete
- [ ] `lib.rs` is under 100 lines (just re-exports + module declarations)
- [ ] All modules compile independently
- [ ] All 30+ tests pass
- [ ] Zero compiler warnings
- [ ] `projects/TODO.md` updated with final status

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Error type change causes cascading type errors | Medium | Do it early (Task 2), fix all call sites in one commit |
| `tuplet!` removal breaks destructuring patterns | Medium | Replace one call site at a time, test after each |
| Module split introduces circular dependencies | Low | Extract in dependency order: types → parser → eval → builtins |
| Lambda env mutation bug (known issue) | Low | Defer to future round — document in TODO, don't fix now |
| `Function` type (Rc<dyn Fn>) is hard to split out | Medium | Keep `Function` in `types.rs` with `DataType`, it's tightly coupled |

## Open Questions

- See `SPEC.md` → Open Questions (all resolved with assumptions noted)

## Task List Target

Tasks are tracked in `projects/TODO.md` (not the default `tasks/todo.md` — user specified `projects/`).

# TODO: scheme-rs Interpreter Improvements

> **Branch**: `fix/interpreter-improvements`
> **Spec**: `projects/SPEC.md`
> **Plan**: `projects/PLAN.md`
> **Capability Map**: `projects/CAPABILITY_MAP.md`
>
> **Status legend**: `[ ]` pending · `[~]` in progress · `[x]` done · `[!]` blocked

---

## Phase 1: Foundation (`codequality`)

- [x] **Task 1: Bump to edition 2021, remove `extern crate`**
  - Acceptance: `Cargo.toml` has `edition = "2021"`, no `extern crate` in any `.rs` file
  - Verify: `cargo build --release` succeeds, `cargo test` passes (30/30)
  - Files: `Cargo.toml`, `src/lib.rs`, `src/main.rs`, `src/cli.rs`, `src/bench.rs`
  - Status: `[x]` done

- [x] **Task 2: Create `SchemeError` enum, replace `&'static str`**
  - Acceptance: All `Result<_, &'static str>` become `Result<_, SchemeError>`. `SchemeError` implements `Display` + `std::error::Error`. Variants: `SyntaxError`, `TypeError`, `UndefinedSymbol`, `ArityError`, `DivisionByZero`, `RuntimeError`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30/30), error messages are still readable in REPL
  - Files: `src/lib.rs` (new enum + all signature changes)
  - Status: `[x]` done

- [x] **Task 3: Remove `tuplet!` macro, use direct slice access**
  - Acceptance: No `tuplet!` macro definition or invocation in codebase. All destructuring uses `vec.get(N)` or pattern matching. Zero "unused variable" warnings.
  - Verify: `cargo build` with zero warnings, `cargo test` passes (30/30)
  - Files: `src/lib.rs`, `src/main.rs`
  - Status: `[x]` done

### Checkpoint: Foundation
- [x] `cargo build` with zero warnings
- [x] All 30 tests pass
- [x] No `extern crate` in any file
- [x] No `&'static str` error types
- [x] No `tuplet!` macro

---

## Phase 2: Bug Fixes (`bugfixes`)

- [x] **Task 4: Fix unary minus**
  - Acceptance: `(- 5)` returns `-5.0`, `(- 5 3 1)` returns `1.0`, `(/ 5)` returns `0.2`
  - Verify: New test `unary_minus_test` in `tests/spec.rs` passes, all 31 tests pass
  - Files: `src/lib.rs` (or `builtins.rs` if module split done), `tests/spec.rs`
  - Status: `[x]` done

- [x] **Task 5: Fix quote shorthand for lists**
  - Acceptance: `'(1 2 3)` works the same as `(quote (1 2 3))`. `'symbol` still works.
  - Verify: New test `quote_shorthand_test` in `tests/spec.rs` passes, all 32 tests pass
  - Files: `src/lib.rs` (or `parser.rs`), `tests/spec.rs`
  - Status: `[x]` done

- [ ] **Task 6: Fix division by zero**
  - Acceptance: `(/ 1 0)` returns `Err(SchemeError::DivisionByZero)`, not `inf`
  - Verify: New test `division_by_zero_test` passes, all existing tests pass
  - Files: `src/lib.rs` (or `builtins.rs`), `tests/spec.rs`
  - Status: `[ ]`

- [ ] **Task 7: Replace `panic!()`/`unreachable!()` with `SchemeError`**
  - Acceptance: No `panic!()` or `unreachable!()` in code paths reachable from user input. All replaced with `Err(SchemeError::...)`.
  - Verify: `grep -rn "panic!\|unreachable!" src/` returns nothing (or only in truly unreachable internal invariants), `cargo test` passes
  - Files: `src/lib.rs`
  - Status: `[ ]`

### Checkpoint: Bug Fixes
- [ ] All bug-fix tests pass
- [ ] No `panic!()` or `unreachable!()` reachable from user input
- [ ] All 30 original tests still pass
- [ ] Zero compiler warnings

---

## Phase 3: Module Split (`modules`)

- [ ] **Task 8: Extract `types.rs`**
  - Acceptance: `AST`, `DataType`, `Procedure`, `Function`, `Env` struct/enum definitions live in `src/types.rs`. `lib.rs` imports from `types`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/types.rs` (new), `src/lib.rs`
  - Status: `[ ]`

- [ ] **Task 9: Extract `error.rs`**
  - Acceptance: `SchemeError` enum lives in `src/error.rs`. `lib.rs` imports from `error`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/error.rs` (new), `src/lib.rs`
  - Status: `[ ]`

- [ ] **Task 10: Extract `parser.rs`**
  - Acceptance: `tokenize`, `read_from_tokens`, `atom`, `parse`, `ReadFromTokenResult` live in `src/parser.rs`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/parser.rs` (new), `src/lib.rs`
  - Status: `[ ]`

- [ ] **Task 11: Extract `env.rs`**
  - Acceptance: `Env` impl block (the `get` method) lives in `src/env.rs`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/env.rs` (new), `src/lib.rs`
  - Status: `[ ]`

- [ ] **Task 12: Extract `eval.rs`**
  - Acceptance: `eval`, `ast_symbol_expression`, `quote_expression`, `if_expression`, `define_expression`, `lambda_expression`, `prepare_arguments`, `execute`, `ast2datatype`, `datatype2str` live in `src/eval.rs`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/eval.rs` (new), `src/lib.rs`
  - Status: `[ ]`

- [ ] **Task 13: Extract `builtins.rs`**
  - Acceptance: `setup()` and all builtin function definitions live in `src/builtins.rs`. `define_comparison!` macro also moves there.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/builtins.rs` (new), `src/lib.rs`
  - Status: `[ ]`

- [ ] **Task 14: Clean up `lib.rs` to re-export hub**
  - Acceptance: `lib.rs` is under 100 lines — just `mod` declarations, re-exports, and the `define_comparison!` macro if needed. No logic in `lib.rs`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+), `wc -l src/lib.rs` < 100
  - Files: `src/lib.rs`
  - Status: `[ ]`

### Checkpoint: Complete
- [ ] `lib.rs` is under 100 lines
- [ ] All modules compile independently
- [ ] All 30+ tests pass
- [ ] Zero compiler warnings
- [ ] `projects/TODO.md` updated with final status

---

## Known Issues (Deferred — Not in This Round)

- [ ] **Lambda env mutation bug**: Each lambda call mutates the captured env instead of creating a fresh local scope. This breaks `set!` and proper closure semantics. Fix requires reworking `Procedure` to create a new env frame per call.
- [ ] **No tail-call optimization**: Deep recursion overflows the stack (~10k frames). Fix requires a trampoline or explicit loop in `eval` for tail positions.
- [ ] **Integer precision loss**: All numbers are `f64`. `i64` is cast to `f64` at eval time. Fix requires a numeric tower (BigInt or at least i64/f64 distinction).
- [ ] **Missing R5RS features**: `let`, `cond`, `case`, `set!`, `when`/`unless`, `quasiquote`/`unquote`, `eq?`/`eqv?`/`equal?`, `string-*` operations, `display`/`newline`, `do` loops, named `let`, macros.
- [ ] **Travis CI is dead**: No working CI. Should migrate to GitHub Actions.
- [ ] **5 unpushed commits on master**: Should push before merging this branch back.

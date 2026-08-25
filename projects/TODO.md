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

- [x] **Task 6: Fix division by zero**
  - Acceptance: `(/ 1 0)` returns `Err(SchemeError::DivisionByZero)`, not `inf`
  - Verify: New test `division_by_zero_test` passes, all 33 tests pass
  - Files: `src/lib.rs` (or `builtins.rs`), `tests/spec.rs`
  - Status: `[x]` done

- [x] **Task 7: Replace `panic!()`/`unreachable!()` with `SchemeError`**
  - Acceptance: No `panic!()` or `unreachable!()` in code paths reachable from user input. All replaced with `Err(SchemeError::...)`.
  - Verify: `grep -rn "panic!\|unreachable!" src/` returns nothing (or only in truly unreachable internal invariants), `cargo test` passes
  - Files: `src/lib.rs`
  - Status: `[x]` done

### Checkpoint: Bug Fixes
- [x] All bug-fix tests pass
- [x] No `panic!()` or `unreachable!()` reachable from user input
- [x] All 30 original tests still pass
- [x] Zero compiler warnings

---

## Phase 3: Module Split (`modules`)

- [x] **Task 8: Extract `types.rs`**
  - Acceptance: `AST`, `DataType`, `Procedure`, `Function`, `Env` struct/enum definitions live in `src/types.rs`. `lib.rs` imports from `types`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/types.rs` (new), `src/lib.rs`
  - Status: `[x]` done

- [x] **Task 9: Extract `error.rs`**
  - Acceptance: `SchemeError` enum lives in `src/error.rs`. `lib.rs` imports from `error`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/error.rs` (new), `src/lib.rs`
  - Status: `[x]` done

- [x] **Task 10: Extract `parser.rs`**
  - Acceptance: `tokenize`, `read_from_tokens`, `atom`, `parse`, `ReadFromTokenResult` live in `src/parser.rs`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/parser.rs` (new), `src/lib.rs`
  - Status: `[x]` done

- [x] **Task 11: Extract `env.rs`**
  - Acceptance: `Env` impl block (the `get` method) lives in `src/env.rs`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/env.rs` (new), `src/lib.rs`
  - Status: `[x]` done

- [x] **Task 12: Extract `eval.rs`**
  - Acceptance: `eval`, `ast_symbol_expression`, `quote_expression`, `if_expression`, `define_expression`, `lambda_expression`, `prepare_arguments`, `execute`, `ast2datatype`, `datatype2str` live in `src/eval.rs`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/eval.rs` (new), `src/lib.rs`
  - Status: `[x]` done

- [x] **Task 13: Extract `builtins.rs`**
  - Acceptance: `setup()` and all builtin function definitions live in `src/builtins.rs`. `define_comparison!` macro also moves there.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+)
  - Files: `src/builtins.rs` (new), `src/lib.rs`
  - Status: `[x]` done

- [x] **Task 14: Clean up `lib.rs` to re-export hub**
  - Acceptance: `lib.rs` is under 100 lines — just `mod` declarations, re-exports, and the `define_comparison!` macro if needed. No logic in `lib.rs`.
  - Verify: `cargo build` succeeds, `cargo test` passes (30+), `wc -l src/lib.rs` < 100
  - Files: `src/lib.rs`
  - Status: `[x]` done

### Checkpoint: Complete
- [x] `lib.rs` is under 100 lines
- [x] All modules compile independently
- [x] All 30+ tests pass
- [x] Zero compiler warnings
- [x] `projects/TODO.md` updated with final status

---

## Phase 4: Architecture Fixes (`archfix`)

- [x] **Task 15: Fix lambda env mutation across all 4 call sites**
  - Acceptance: Each lambda call creates a fresh empty `HashMap` for params, parented to the closure's env. No stale bindings leak between calls. All 4 call sites fixed: `eval.rs` symbol path, `eval.rs` children path, `builtins.rs` `apply`, `builtins.rs` `map`.
  - Verify: New tests `lambda_fresh_env_test`, `lambda_no_arg_leak_test`, `lambda_via_apply_fresh_env_test`, `lambda_via_map_fresh_env_test` pass
  - Files: `src/eval.rs`, `src/builtins.rs`, `tests/spec.rs`
  - Status: `[x]` done (commit `b200970`)

- [x] **Task 16: Fix string type confusion in `ast2datatype`**
  - Acceptance: `(quote "hello")` produces `DataType::String`, not `DataType::Symbol`. `string?` returns `#t` for quoted strings. Also: lambda body accepts single expressions (not just `AST::Children`), so `(lambda (x) x)` works.
  - Verify: New test `quote_string_type_test` passes, updated `quote_expression_test` expects `DataType::String`
  - Files: `src/eval.rs`, `tests/spec.rs`
  - Status: `[x]` done (commit `b200970`)

- [x] **Task 17: Fix `Env::get` per-variant cloning landmine**
  - Acceptance: `Env::get` uses `.cloned()` instead of manual per-variant match. Future `DataType` variants won't silently return `None`.
  - Verify: `cargo build` succeeds, all tests pass
  - Files: `src/env.rs`
  - Status: `[x]` done (commit `b200970`)

### Checkpoint: Architecture Fixes
- [x] Lambda calls use fresh env frames (4 call sites)
- [x] Quoted strings are `DataType::String`
- [x] Single-expression lambda bodies work
- [x] `Env::get` is future-proof
- [x] 38/38 tests pass (was 33)
- [x] Zero compiler warnings

---

## Phase 5: R5RS Features (`r5rs`)

> **Spec**: `projects/SPEC_R5RS.md`

### Special Forms (eval.rs)

- [x] **Task 18: `let` — local bindings**
  - Acceptance: `(let ((x 1) (y 2)) (+ x y))` returns `3`. Bindings are scoped to the body only.
  - Verify: New test `let_test` passes, all 39 tests pass
  - Files: `src/eval.rs`, `tests/spec.rs`
  - Status: `[x]` done

- [x] **Task 19: `cond` — multi-branch conditional**
  - Acceptance: `(cond ((= 1 2) 'a) ((= 1 1) 'b) (else 'c))` returns `'b`. `else` clause works.
  - Verify: New test `cond_test` passes, all 40 tests pass
  - Files: `src/eval.rs`, `tests/spec.rs`
  - Status: `[x]` done

- [x] **Task 20: `set!` — mutate existing binding**
  - Acceptance: `(define x 1) (set! x 5) x` returns `5`. Setting a variable in the current scope mutates it in place.
  - Verify: New test `set_test` passes, all 41 tests pass
  - Files: `src/eval.rs`, `src/env.rs`, `tests/spec.rs`
  - Status: `[x]` done

- [x] **Task 21: `when` / `unless`**
  - Acceptance: `(when #t 1)` returns `1`, `(when #f 1)` returns unspecified, `(unless #t 1)` returns unspecified, `(unless #f 1)` returns `1`
  - Verify: New test `when_unless_test` passes, all 43 tests pass
  - Files: `src/eval.rs`, `tests/spec.rs`
  - Status: `[x]` done

- [x] **Task 22: `case` — key dispatch**
  - Acceptance: `(case 2 ((1) 'one) ((2) 'two) (else 'other))` returns `'two`
  - Verify: New test `case_test` passes, all 43 tests pass
  - Files: `src/eval.rs`, `tests/spec.rs`
  - Status: `[x]` done

### Builtins (builtins.rs)

- [ ] **Task 23: `eq?` / `eqv?` / `equal?`**
  - Acceptance: `(eq? 'a 'a)` → `#t`, `(eqv? 1 1)` → `#t`, `(equal? (list 1 2) (list 1 2))` → `#t`
  - Verify: New test `equality_predicates_test` passes, all existing tests pass
  - Files: `src/builtins.rs`, `tests/spec.rs`
  - Status: `[ ]`

- [ ] **Task 24: `display` / `newline`**
  - Acceptance: `(display "hello")` prints `hello` (no quotes), `(newline)` prints a newline
  - Verify: New test `display_newline_test` passes, all existing tests pass
  - Files: `src/builtins.rs`, `tests/spec.rs`
  - Status: `[ ]`

- [ ] **Task 25: String operations**
  - Acceptance: `(string-length "abc")` → `3`, `(string-append "a" "b")` → `"ab"`, `(string->symbol "x")` → `'x`, `(symbol->string 'x)` → `"x"`
  - Verify: New test `string_operations_test` passes, all existing tests pass
  - Files: `src/builtins.rs`, `tests/spec.rs`
  - Status: `[ ]`

- [ ] **Task 26: Type/number predicates + integer division**
  - Acceptance: `(boolean? #t)` → `#t`, `(zero? 0)` → `#t`, `(positive? 1)` → `#t`, `(negative? -1)` → `#t`, `(modulo 7 3)` → `1`, `(quotient 7 3)` → `2`, `(remainder 7 3)` → `1`, `(even? 4)` → `#t`, `(odd? 3)` → `#t`
  - Verify: New test `predicates_and_int_div_test` passes, all existing tests pass
  - Files: `src/builtins.rs`, `tests/spec.rs`
  - Status: `[ ]`

### Checkpoint: R5RS Features
- [ ] `let`, `cond`, `set!`, `when`/`unless`, `case` all work
- [ ] `eq?`/`eqv?`/`equal?`, `display`/`newline`, string ops, predicates, integer division all work
- [ ] All 38+ tests pass (new tests added for each feature)
- [ ] Zero compiler warnings

---

## Known Issues (Deferred — Not in This Round)

- [ ] **No tail-call optimization**: Deep recursion overflows the stack (~10k frames). Fix requires a trampoline or explicit loop in `eval` for tail positions.
- [ ] **Integer precision loss**: All numbers are `f64`. `i64` is cast to `f64` at eval time. Fix requires a numeric tower (BigInt or at least i64/f64 distinction).
- [ ] **Travis CI is dead**: No working CI. Should migrate to GitHub Actions.
- [ ] **5 unpushed commits on master**: Should push before merging this branch back.
- [ ] **Advanced R5RS features deferred**: `quasiquote`/`unquote`, `do` loops, named `let`, macros (`define-syntax`/`syntax-rules`), `call/cc`.

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

- [x] **Task 23: `eq?` / `eqv?` / `equal?`**
  - Acceptance: `(eq? 'a 'a)` → `#t`, `(eqv? 1 1)` → `#t`, `(equal? (list 1 2) (list 1 2))` → `#t`
  - Verify: New test `equality_predicates_test` passes, all 47 tests pass
  - Files: `src/builtins.rs`, `tests/spec.rs`
  - Status: `[x]` done

- [x] **Task 24: `display` / `newline`**
  - Acceptance: `(display "hello")` prints `hello` (no quotes), `(newline)` prints a newline
  - Verify: New test `display_newline_test` passes, all 47 tests pass
  - Files: `src/builtins.rs`, `tests/spec.rs`
  - Status: `[x]` done

- [x] **Task 25: String operations**
  - Acceptance: `(string-length "abc")` → `3`, `(string-append "a" "b")` → `"ab"`, `(string->symbol "x")` → `'x`, `(symbol->string 'x)` → `"x"`
  - Verify: New test `string_operations_test` passes, all 47 tests pass
  - Files: `src/builtins.rs`, `tests/spec.rs`
  - Status: `[x]` done

- [x] **Task 26: Type/number predicates + integer division**
  - Acceptance: `(boolean? #t)` → `#t`, `(zero? 0)` → `#t`, `(positive? 1)` → `#t`, `(negative? -1)` → `#t`, `(modulo 7 3)` → `1`, `(quotient 7 3)` → `2`, `(remainder 7 3)` → `1`, `(even? 4)` → `#t`, `(odd? 3)` → `#t`
  - Verify: New test `predicates_and_int_div_test` passes, all 47 tests pass
  - Files: `src/builtins.rs`, `tests/spec.rs`
  - Status: `[x]` done

### Checkpoint: R5RS Features
- [x] `let`, `cond`, `set!`, `when`/`unless`, `case` all work
- [x] `eq?`/`eqv?`/`equal?`, `display`/`newline`, string ops, predicates, integer division all work
- [x] All 47 tests pass (was 38, added 9 new tests)
- [x] Zero compiler warnings

---

## Phase 6: Tail-Call Optimization (`tco`)

> **Spec**: `projects/SPEC_TCO.md`

- [x] **Task 27: Convert `eval` to a trampoline loop**
  - Acceptance: `eval` body is wrapped in `loop { ... }`. Tail positions (`if` branches, `cond`/`when`/`unless`/`case` bodies, `let` body, lambda body) use `continue` with reassigned `ast`/`env` instead of recursive `eval` calls. Non-tail positions (arguments, tests, bindings) still use recursive calls.
  - Verify: `cargo build` succeeds, all 47 existing tests pass
  - Files: `src/eval.rs`
  - Status: `[x]` done

- [x] **Task 28: Add deep recursion tests**
  - Acceptance: `(loop 100000)` where `loop` is a self-recursive tail function returns without stack overflow. Mutual recursion (even/odd) at 100k depth also works.
  - Verify: New tests `tco_deep_recursion_test` and `tco_mutual_recursion_test` pass, all 49 tests pass
  - Files: `tests/spec.rs`
  - Status: `[x]` done

### Checkpoint: TCO
- [x] Deep recursion (100k+) doesn't overflow stack
- [x] Mutual recursion (100k+) doesn't overflow stack
- [x] All 49 tests pass (was 47, added 2 new)
- [x] Zero compiler warnings

---

## Phase 7: CI Migration (`ci`)

- [x] **Task 29: Migrate Travis CI to GitHub Actions**
  - Acceptance: `.github/workflows/ci.yml` runs `cargo build` and `cargo test` on push/PR. `.travis.yml` is removed. Tests stable Rust (nightly allowed to fail for bench-only).
  - Verify: Workflow file is valid YAML, `cargo test` passes locally
  - Files: `.github/workflows/ci.yml` (new), `.travis.yml` (deleted)
  - Status: `[x]` done

### Checkpoint: CI Migration
- [x] GitHub Actions workflow exists and is valid
- [x] Travis config removed
- [x] All 49 tests still pass

---

## Phase 8: Numeric Tower — i64/f64 Split (`numeric`)

> **Spec**: `projects/SPEC_NUMERIC.md`

- [x] **Task 30: Split `DataType::Number` into `Integer`/`Float`**
  - Acceptance: `DataType` has `Integer(i64)` and `Float(f64)` instead of `Number(f64)`. `PartialEq` handles cross-type numeric equality (`Integer(42) == Float(42.0)` is true). Helper methods `as_f64()`, `is_number()`, `is_integer()` added.
  - Verify: `cargo build` succeeds
  - Files: `src/types.rs`
  - Status: `[x]` done

- [x] **Task 31: Update `eval.rs` for Integer/Float**
  - Acceptance: `define` handles `AST::Integer` → `DataType::Integer`, `AST::Float` → `DataType::Float`. `ast2datatype` updated. `datatype2str` updated.
  - Verify: `cargo build` succeeds
  - Files: `src/eval.rs`
  - Status: `[x]` done

- [x] **Task 32: Update `builtins.rs` with promotion rules**
  - Acceptance: Arithmetic uses integer arithmetic when both operands are integers, promotes to float when mixed. `/` always returns Float. `modulo`/`quotient`/`remainder` return Integer. Comparisons work across types. Predicates work on both types. `eqv?` is type-sensitive.
  - Verify: `cargo build` succeeds, `cargo test` passes
  - Files: `src/builtins.rs`
  - Status: `[x]` done

- [x] **Task 33: Update tests for Integer/Float**
  - Acceptance: Existing tests updated to expect `Integer(...)` or `Float(...)` instead of `Number(...)`. New tests for: integer preservation, mixed-type promotion, division returns float, numeric equality vs type equality.
  - Verify: All 51 tests pass (was 49, added 2 new)
  - Files: `tests/spec.rs`
  - Status: `[x]` done

### Checkpoint: Numeric Tower
- [x] `42` evaluates to `Integer(42)`, not `Float(42.0)`
- [x] `(+ 1 2)` → `Integer(3)`, `(+ 1 2.0)` → `Float(3.0)`
- [x] `(/ 6 2)` → `Float(3.0)`
- [x] `(= 1 1.0)` → `#t`, `(eqv? 1 1.0)` → `#f`
- [x] All 51 tests pass
- [x] Zero compiler warnings

---

## Phase 9: Examples, Benchmarks, and Performance (`perf`)

> **Spec**: `projects/SPEC_PERF.md`

### Examples

- [ ] **Task 34: Create example .scm files**
  - Acceptance: `examples/fact.scm`, `examples/fib.scm`, `examples/loop.scm`, `examples/list.scm`, `examples/closures.scm` all run without error via `cargo run --bin scheme -- examples/<file>.scm`
  - Verify: Each file runs and produces expected output
  - Files: `examples/fact.scm`, `examples/fib.scm`, `examples/loop.scm`, `examples/list.scm`, `examples/closures.scm`
  - Status: `[ ]`

### Missing Builtins

- [x] **Task 35: Add `null?` builtin**
  - Acceptance: `(null? '())` → `#t`, `(null? (list 1))` → `#f`. Needed for list operations in examples.
  - Verify: New test `null_pred_test` passes, all 52 tests pass
  - Files: `src/builtins.rs`, `tests/spec.rs`
  - Status: `[x]` done

### Benchmarks

- [ ] **Task 36: Expand benchmark suite**
  - Acceptance: `bench.rs` has 6 benchmarks: fact20, fib25, tco_loop_100k, sum_to_1000, ackermann_2_8, list_ops. All compile and run via `cargo bench --features "unstable"`.
  - Verify: `cargo bench --features "unstable"` runs all 6 benchmarks
  - Files: `src/bench.rs`
  - Status: `[ ]`

### Performance

- [ ] **Task 37: Reduce eval loop overhead**
  - Acceptance: TCO loop benchmark is at least 2x faster than baseline (was ~2.9µs/iter, target <1.5µs/iter). Achieved by reducing unnecessary clones in the eval loop.
  - Verify: `cargo bench --features "unstable"` shows improvement, all 51 tests pass
  - Files: `src/eval.rs`
  - Status: `[ ]`

### Checkpoint: Examples, Benchmarks, and Performance
- [ ] 5 example .scm files run correctly
- [ ] `null?` builtin works
- [ ] 6 benchmarks run via `cargo bench`
- [ ] TCO loop is at least 2x faster
- [ ] All 51+ tests pass
- [ ] Zero compiler warnings

---

## Known Issues (Deferred — Not in This Round)

- [ ] **Advanced R5RS features deferred**: `quasiquote`/`unquote`, `do` loops, named `let`, macros (`define-syntax`/`syntax-rules`), `call/cc`.
- [ ] **No BigInt support**: Integers are i64 only. Arbitrary precision would require num-bigint dependency.

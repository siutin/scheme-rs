# Spec: R5RS Feature Additions (Phase 5)

## Objective

Add the most commonly used R5RS features that are currently missing from scheme-rs. These are fundamental to writing real Scheme programs — without `let`, `cond`, and `set!`, the interpreter can't express most idiomatic Scheme code.

**User**: Developers learning Scheme who expect standard R5RS syntax to work.
**Success looks like**: Each new feature has a test, all existing 38 tests still pass, common Scheme programs run correctly.

## Current State

**Special forms supported**: `quote`, `if`, `define`, `lambda`
**Builtins supported**: `+ - * / < <= = > >= abs append apply begin car cdr cons length list list? map max min not number? pair? print procedure? string? symbol?`

## Features to Add (in priority order)

### Special Forms (eval.rs)

1. **`let`** — `(let ((x 1) (y 2)) (+ x y))` → local bindings
2. **`cond`** — `(cond ((= x 1) 'one) ((= x 2) 'two) (else 'other))` → multi-branch
3. **`set!`** — `(set! x 5)` → mutate existing binding
4. **`when` / `unless`** — `(when test body...)` / `(unless test body...)`
5. **`case`** — `(case key ((1) 'one) ((2) 'two) (else 'other))`
6. **`begin` as special form** — currently a builtin; should work with 0 args (return unspecified) and as a body wrapper

### Builtins (builtins.rs)

7. **`eq?` / `eqv?` / `equal?`** — equality predicates
8. **`display` / `newline`** — output (display doesn't quote strings, newline prints \n)
9. **`string-*` operations** — `string-length`, `string-append`, `string->symbol`, `symbol->string`
10. **`boolean?`** — type predicate
11. **`zero?` / `positive?` / `negative?`** — number predicates
12. **`modulo` / `remainder` / `quotient`** — integer division
13. **`even?` / `odd?`** — integer predicates

### Not in This Round (deferred)

- `quasiquote` / `unquote` — requires reader macro support, complex
- `do` loops — low priority, `let` recursion covers most cases
- named `let` — depends on `let` + TCO for deep recursion
- macros (`define-syntax` / `syntax-rules`) — major feature, separate spec
- `call/cc` — requires reworking eval to CPS, major architectural change
- full numeric tower — separate spec (Phase 8)
- tail-call optimization — separate spec (Phase 6)

## Testing Strategy

Each feature gets at least one test in `tests/spec.rs`:
- Basic usage (happy path)
- Edge case (empty args, wrong type, etc.)
- Interaction with existing features (e.g., `let` + `lambda`)

All 38 existing tests must continue to pass.

## Boundaries

**Always do:**
- One feature per commit (code + test + TODO.md update)
- `cargo test` passes before each commit

**Never do:**
- Break existing tests
- Add new dependencies
- Change the public API of `lib.rs` exports

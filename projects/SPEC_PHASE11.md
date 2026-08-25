# Spec: R5RS Core Gaps (Phase 11)

## Objective

Close the most commonly-used R5RS gaps that block writing idiomatic Scheme.
Focus on features that are frequently expected but missing: `define`
function shorthand, `let*`/`letrec`, `and`/`or`, common list utilities,
and common string utilities.

## Tasks

### Special Forms

1. **Task 44: `define` function shorthand**
   - `(define (f x y) body...)` → `(define f (lambda (x y) body...))`
   - Also support curried form: `(define ((f x) y) body...)` (R5RS doesn't require this, skip)
   - Files: `src/eval.rs`, `tests/spec.rs`

2. **Task 45: `let*` — sequential bindings**
   - `(let* ((x 1) (y (+ x 1))) (+ x y))` → `3`
   - Each binding sees previous bindings
   - Files: `src/eval.rs`, `tests/spec.rs`

3. **Task 46: `letrec` — recursive bindings**
   - `(letrec ((even? (lambda (n) (if (= n 0) #t (odd? (- n 1))))) (odd? (lambda (n) (if (= n 0) #f (even? (- n 1)))))) (even? 10))` → `#t`
   - All bindings are visible to all init expressions
   - Files: `src/eval.rs`, `tests/spec.rs`

4. **Task 47: `and` / `or` — short-circuit boolean operators**
   - `(and 1 2 3)` → `3`, `(and 1 #f 3)` → `#f`, `(and)` → `#t`
   - `(or #f #f 3)` → `3`, `(or #f #f)` → `#f`, `(or)` → `#f`
   - Short-circuit: don't evaluate after first `#f` (for `and`) / first truthy (for `or`)
   - Last evaluated expression is in tail position
   - Files: `src/eval.rs`, `tests/spec.rs`

### List Utilities

5. **Task 48: `reverse`, `list-ref`, `member`, `assoc`**
   - `(reverse (list 1 2 3))` → `(3 2 1)`
   - `(list-ref (list 1 2 3) 1)` → `2`
   - `(member 2 (list 1 2 3))` → `(2 3)`
   - `(assoc 'b (list (list 'a 1) (list 'b 2)))` → `(b 2)`
   - Also: `memq`, `memv`, `assq`, `assv` (same but using `eq?`/`eqv?`)
   - Files: `src/builtins.rs`, `tests/spec.rs`

### String Utilities

6. **Task 49: `string=?`, `substring`, `string->list`, `list->string`, `string-ref`**
   - `(string=? "abc" "abc")` → `#t`
   - `(substring "hello" 1 3)` → `"el"`
   - `(string->list "abc")` → `(a b c)` (as symbols, since no char type)
   - `(string-ref "hello" 0)` → `"h"` (as 1-char string, since no char type)
   - Files: `src/builtins.rs`, `tests/spec.rs`

### Math Functions

7. **Task 50: `sqrt`, `expt`, `floor`, `ceiling`, `round`, `truncate`, `gcd`, `lcm`**
   - `(sqrt 16)` → `4.0` (always float)
   - `(expt 2 10)` → `1024`
   - `(floor 3.7)` → `3`, `(ceiling 3.2)` → `4`, `(round 3.5)` → `4`, `(truncate 3.7)` → `3`
   - `(gcd 12 18)` → `6`, `(lcm 4 6)` → `12`
   - Files: `src/builtins.rs`, `tests/spec.rs`

### Error Handling

8. **Task 51: `error` procedure**
   - `(error "something went wrong")` raises a RuntimeError
   - Optional irritants: `(error "bad value" x)`
   - Files: `src/builtins.rs`, `tests/spec.rs`

## Testing

- Each task gets its own test function
- All 57 existing tests must pass
- Zero compiler warnings

## Boundaries

- No new dependencies
- No character type (use 1-char strings instead)
- No vectors (deferred)
- No macros (deferred)
- No call/cc (deferred)

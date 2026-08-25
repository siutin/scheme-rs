# Spec: Bug Fixes + R5RS Features (Phase 10)

## Objective

Fix two usability bugs (multi-expression bodies, internal define) and add the most commonly used remaining R5RS features (quasiquote, named let, do loops).

## Bug Fixes

### 1. Multi-expression lambda/let bodies

**Problem**: `(lambda (x) (display x) x)` fails — only the first expression after args is treated as body. Users must wrap in explicit `begin`.

**Fix**: Collect ALL expressions after the args/binddings as body. Evaluate them in sequence, returning the last. For tail position, the last expression is the tail.

**Affected**: `lambda`, `let`, `when`, `unless`, `cond` clauses, `case` clauses

### 2. Internal `define` in lambda bodies

**Problem**: `(lambda () (define x 5) x)` fails — `define` returns `Ok(None)` but the body only evaluates one expression.

**Fix**: Fixed by #1 — once multi-expression bodies work, `(define x 5)` runs as the first expression and `x` as the second.

## R5RS Features

### 3. `quasiquote` / `unquote` / `unquote-splicing`

```
`(1 2 ,(+ 1 2))          → (1 2 3)
`(1 ,@(list 2 3) 4)      → (1 2 3 4)
```

Template literal with selective evaluation. `` ` `` is shorthand for `(quasiquote ...)`, `,` for `(unquote ...)`, `,@` for `(unquote-splicing ...)`.

### 4. Named `let`

```
(let loop ((i 0) (acc 1))
  (if (= i 10) acc (loop (+ i 1) (* acc 2))))
```

Creates a recursive procedure named `loop` and immediately calls it with initial values. Essential for idiomatic Scheme loops.

### 5. `do` loops

```
(do ((i 0 (+ i 1)))       ; var init step
    ((= i 10) 'done)      ; test result
  (display i))            ; body
```

R5RS imperative loop construct. Less commonly used but part of the standard.

## Tasks

1. **Task 38**: Multi-expression bodies for lambda, let, when, unless
2. **Task 39**: Verify internal define works (test-only, should work after Task 38)
3. **Task 40**: quasiquote / unquote / unquote-splicing
4. **Task 41**: Named let
5. **Task 42**: do loops
6. **Task 43**: Update examples to use new features (remove begin workarounds)

## Testing

- `(lambda (x) (display x) x)` works without explicit `begin`
- `(let () (define x 5) x)` returns 5
- `` `(1 2 ,(+ 1 2)) `` → `(1 2 3)`
- `` `(1 ,@(list 2 3) 4) `` → `(1 2 3 4)`
- Named let: `(let loop ((i 0)) (if (= i 5) 'done (loop (+ i 1))))` → `done`
- `do` loop: `(do ((i 0 (+ i 1))) ((= i 5) i))` → `5`
- All 52 existing tests pass

## Boundaries

- No new dependencies
- Don't change the public API
- Macros (define-syntax/syntax-rules) and call/cc remain deferred

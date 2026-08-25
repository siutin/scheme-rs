# Spec: Tail-Call Optimization (Phase 6)

## Objective

Eliminate stack overflow on deep recursion by converting `eval` from a recursive function to a loop that reuses the current stack frame for tail calls.

**Problem**: Currently, every function call and tail-position expression recursively calls `eval`. A Scheme loop like `(define (loop n) (if (= n 0) 'done (loop (- n 1))))` overflows the stack at ~10k iterations because each `(loop ...)` call adds a Rust stack frame.

**Solution**: Wrap the `eval` body in a `loop`. In tail positions (`if` branches, `cond`/`when`/`unless`/`case` bodies, `let` body, lambda body), instead of `return eval(...)`, reassign `ast` and `env` then `continue` the loop. Non-tail positions (argument evaluation, condition tests) still use recursive `eval` calls.

## Tail Positions

A position is "tail" if its result is the result of the enclosing expression:

| Form | Tail position |
|------|--------------|
| `if` | consequent and alternative |
| `cond` | body of the matching clause |
| `when` / `unless` | body |
| `case` | body of the matching clause |
| `let` | body |
| `begin` | last expression |
| lambda body | the body itself |
| function call | the call itself (when in tail context) |

## Non-Tail Positions (still recursive)

- `if` condition test
- `cond` / `when` / `unless` / `case` test expressions
- `let` binding init expressions
- function call arguments
- `define` value expression
- `set!` value expression

## Approach

1. Wrap the `match ast_option.clone()` body in `loop { ... }`
2. Change `eval(Some(tail_expr), env)` in tail positions to `ast_option = Some(tail_expr); env = tail_env; continue;`
3. Keep `eval(Some(non_tail_expr), env)` for non-tail positions
4. Return statements break out of the loop with `return Ok(...)`

## Testing

- Deep recursion test: `(define (loop n) (if (= n 0) 'done (loop (- n 1)))) (loop 100000)` should return `'done` without stack overflow
- Mutual recursion: odd/even pair recursing 100k deep
- All 47 existing tests must still pass

## Boundaries

- No new dependencies
- Don't change the public API (`eval` signature stays the same)
- Don't change `DataType`, `AST`, or `Env` structs

# Spec: Numeric Tower — i64/f64 Split (Phase 8)

## Objective

Split `DataType::Number(f64)` into `DataType::Integer(i64)` and `DataType::Float(f64)` so that integer literals preserve their exactness. `42` should stay `42`, not become `42.0`.

**Problem**: Currently `AST::Integer(42)` is converted to `DataType::Number(42.0)` at eval time. This loses precision for large integers and produces confusing output (`42.0` instead of `42`).

**Solution**: Add `DataType::Integer(i64)` alongside `DataType::Float(f64)`. Arithmetic operations promote to float only when needed (mixed-type operations or float operands).

## Design

### DataType change

```rust
// Before
pub enum DataType {
    Number(f64),
    ...
}

// After
pub enum DataType {
    Integer(i64),
    Float(f64),
    ...
}
```

### Promotion rules

- `Integer + Integer` → `Integer` (no overflow check for now)
- `Integer <op> Float` or `Float <op> Integer` → `Float` (promote)
- `Float <op> Float` → `Float`
- `/` always returns `Float` (division may produce non-integers)
- `modulo`, `quotient`, `remainder` → `Integer` (require integer operands, cast if float)

### Display

- `Integer(42)` → `"42"`
- `Float(42.0)` → `"42.0"`
- `Float(42.5)` → `"42.5"`

### Comparison

`Integer(42) == Float(42.0)` should be `true` for `=` builtin (numeric equality), but `eq?` should return `false` (different types). `eqv?` returns `false` for different types too.

## Tasks

1. **Task 30**: Split `DataType::Number` into `Integer`/`Float` in `types.rs`, update `PartialEq` for numeric cross-type equality
2. **Task 31**: Update `eval.rs` — `define`, `ast2datatype`, and all integer/float literal handling
3. **Task 32**: Update `builtins.rs` — all arithmetic, comparison, and predicate builtins with promotion rules
4. **Task 33**: Update tests — fix existing tests that expect `Number(...)`, add new tests for integer preservation and type promotion

## Testing

- `42` evaluates to `Integer(42)`, not `Float(42.0)`
- `42.5` evaluates to `Float(42.5)`
- `(+ 1 2)` → `Integer(3)`
- `(+ 1 2.0)` → `Float(3.0)`
- `(/ 6 2)` → `Float(3.0)` (division always returns float)
- `(= 1 1.0)` → `#t` (numeric equality)
- `(eqv? 1 1.0)` → `#f` (type-sensitive)
- All 49 existing tests pass (with updated expected values)

## Boundaries

- No new dependencies
- `AST` types don't change (already has `Integer(i64)` and `Float(f64)`)
- `eval` signature doesn't change

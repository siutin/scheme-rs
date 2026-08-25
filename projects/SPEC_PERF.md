# Spec: Examples, Benchmarks, and Performance (Phase 9)

## Objective

Add real-world example programs, comprehensive benchmarks, and targeted performance optimizations to the interpreter. The current TCO loop runs at ~2.9µs/iter which is too slow for practical use.

**Problem**: No example programs exist to demonstrate the interpreter's capabilities. The benchmark suite has only one test (`fact 20`). Performance bottlenecks in the eval loop make even simple loops slow.

**Success looks like**: Example `.scm` files run correctly, benchmarks cover the main use cases, and the TCO loop overhead is reduced by at least 2x.

## Performance Analysis

Current bottlenecks identified:

1. **`ast_option.clone()` at top of every loop iteration** — clones the entire AST every time, even for simple symbols
2. **New `HashMap` allocation per lambda call** — every tail call creates a fresh `HashMap` + `Rc<RefCell<Env>>`
3. **String-based symbol lookups** — `env.borrow().get(&s)` does string hashing on every variable reference
4. **`Rc::clone` on every env access** — env is cloned for every sub-eval call

## Tasks

### Examples (examples/)

Create `.scm` files demonstrating the interpreter's features:
- `fact.scm` — factorial (tree recursion)
- `fib.scm` — fibonacci (tree recursion)
- `loop.scm` — tail-recursive loop (TCO)
- `list.scm` — list operations (map, filter, reduce)
- `closures.scm` — closures with set! (counter, accumulator)

### Benchmarks (src/bench.rs)

Expand from 1 to 6+ benchmarks:
- `fact20` — tree recursion (existing)
- `fib25` — exponential tree recursion
- `tco_loop_100k` — pure tail-call loop
- `sum_to_1000` — tail-recursive accumulation
- `ackermann_2_8` — deep nested recursion
- `list_ops` — list build + sum

### Performance (src/eval.rs)

- Avoid cloning AST when it's a simple symbol (by-value match)
- Avoid redundant env clones where possible

### Missing builtins

- `null?` — needed for list operations

## Testing

- All example files run without error via `cargo run --bin scheme -- examples/<file>.scm`
- All benchmarks compile and run via `cargo bench --features "unstable"`
- All 51 existing tests still pass

## Boundaries

- No new dependencies
- Don't change the public API
- Don't break existing tests

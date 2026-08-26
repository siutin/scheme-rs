
# scheme-rs
A Scheme interpreter in Rust — inspired by Peter Norvig's lispy, grown into its own.

<p align="center">
  <a href="https://github.com/siutin/scheme-rs/actions" alt="CI Status">
    <img src="https://github.com/siutin/scheme-rs/actions/workflows/ci.yml/badge.svg?branch=master"/>
  </a>
  <a href="https://app.fossa.io/projects/git%2Bgithub.com%2Fsiutin%2Fscheme-rs?ref=badge_shield" alt="FOSSA Status">
    <img src="https://app.fossa.io/api/projects/git%2Bgithub.com%2Fsiutin%2Fscheme-rs.svg?type=shield"/>
  </a>
</p>

## Quick Start

Try the REPL:

```
> cargo run --release --bin cli

Welcome to scheme-rs
scheme=> (+ 1 2 (* 3 4 5) 6 7 (/ 8 9 10))
Float(76.08888888888889)
scheme=> (let ((x 10) (y 20)) (+ x y))
Integer(30)
scheme=> (define (fact n) (if (= n 0) 1 (* n (fact (- n 1)))))
scheme=> (fact 10)
Integer(3628800)
scheme=>
```

Run Scheme code from a file:

```
# using cargo
> cargo run --release --bin scheme -- ./examples/primes.scm

# or executing from the build
> scheme ./examples/primes.scm
```

## Examples

The `examples/` directory contains programs demonstrating the interpreter's features:

| File | Description | Features exercised |
|------|-------------|-------------------|
| `demo_01.scm` | Circle area calculation | Basic arithmetic, `pi` constant |
| `fact.scm` | Factorial (tree recursion) | `define`, recursion |
| `fib.scm` | Fibonacci (tree recursion) | `define`, recursion |
| `loop.scm` | Tail-recursive loop | TCO, `set!`, closures |
| `closures.scm` | Closures with state | `set!`, `lambda`, mutable state |
| `list.scm` | List operations | `map`, `filter`, `reduce`, named `let`, quasiquote |
| `primes.scm` | Prime number theory | `filter`, `for-each`, `car`/`cadr`/`caddr`, `number->string` (radix), `do`, `assert`, Goldbach conjecture |
| `stats.scm` | Statistics & numerical math | Transcendental functions, `let*`, `letrec`, `string->number`, `integer?`/`real?`, `assert` |
| `symbolic.scm` | Symbolic differentiation | `cadr`/`caddr`, quasiquote, `define` shorthand, `let*`, `cond`, `and`/`or`, `assert`, `error` |

## Features

**R5RS coverage:** ~69% (104 supported, 3 partial, 47 not supported). See `projects/R5RS_SUPPORT.md` for the full audit.

### Special forms

`quote` `quasiquote` `unquote` `unquote-splicing` `if` `define` (incl. function shorthand) `lambda` `let` (incl. named let) `let*` `letrec` `set!` `cond` `case` `when` `unless` `begin` `and` `or` `do`

### Built-in procedures

**Arithmetic:** `+ - * / < <= = > >= abs min max modulo quotient remainder gcd lcm sqrt expt floor ceiling round truncate`

**Transcendental:** `exp log sin cos tan asin acos atan` (with 2-arg `atan2` form)

**Numeric predicates:** `number? integer? real? zero? positive? negative? even? odd?`

**Lists:** `car cdr cons list length append reverse list-ref list-tail member memq memv assoc assq assv null? list? pair?`

**car/cdr compositions:** `caar cadr cdar cddr caaar caadr cadar caddr cdaar cdadr cddar cdddr cadddr`

**Strings:** `string? string-length string-append string=? string<? string>? substring string-ref string-copy string->list list->string string->symbol symbol->string make-string string->number number->string`

**Higher-order:** `map apply for-each filter`

**I/O:** `display newline print`

**Predicates:** `boolean? symbol? procedure? eq? eqv? equal? not`

**Error handling:** `error assert`

### Tail-call optimization

Self-recursive and mutually-recursive tail calls run in constant stack space (tested to 100k depth).

### Numeric types

Integers (`i64`) and floats (`f64`) are distinct types. Integer arithmetic preserves exactness; mixed-type operations promote to float. Division always returns float.

### Quasiquote

Full quasiquote support with unquote and unquote-splicing:

```scheme
scheme=> (define x 42)
scheme=> `(the answer is ,x)
List([Symbol("the"), Symbol("answer"), Symbol("is"), Integer(42)])
```

## Building

```
cargo build --release --bin scheme       # interpreter
cargo build --release --bin cli          # interactive shell
cargo build --features "unstable"        # benchmarks (nightly required)
```

## Testing

```
cargo test                               # run all 87 tests
```

## Benchmarks

```
cargo bench --features "unstable"        # requires nightly toolchain
```

| Benchmark | Description | Typical time |
|-----------|-------------|-------------|
| `fact20` | Factorial of 20 (tree recursion) | ~46 µs |
| `fib25` | Fibonacci of 25 (tree recursion) | ~510 ms |
| `tco_loop_100k` | 100k tail-call loop | ~161 ms |
| `sum_to_1000` | Sum 1..1000 (tail-recursive) | ~2.3 ms |
| `ackermann_2_8` | Ackermann(2,8) | ~527 µs |
| `list_ops` | List build + sum | ~48 ms |

## Architecture

The interpreter is structured as:

- `src/parser.rs` — Tokenizer + recursive-descent parser (produces `AST`)
- `src/eval.rs` — Tree-walking evaluator with trampoline-style TCO loop
- `src/env.rs` — `Environment` trait + `Env` implementation (HashMap-based with lexical scoping)
- `src/builtins.rs` — All built-in procedures registered in `setup()`
- `src/types.rs` — `DataType` enum, `Procedure`, `Function` types
- `src/error.rs` — `SchemeError` type

The `Environment` trait (`get`/`set`/`define`) decouples the interpreter from the environment representation, enabling future alternative implementations (interned symbols, mock environments for testing, etc.).

## Project Documentation

Detailed specs and plans live in `projects/`:

| Document | Purpose |
|----------|---------|
| `projects/R5RS_SUPPORT.md` | Full R5RS feature support audit |
| `projects/PLAN.md` | Implementation plan (12 phases, 52 tasks) |
| `projects/FUTURE_PLAN.md` | Candidate next phases with priority ordering |
| `projects/SPEC_ENV_TRAIT.md` | Environment trait design |
| `projects/SPEC_INTERNED_ENV.md` | Interned env experiment (reverted — analysis) |
| `projects/SPEC_PERF.md` | Performance optimization spec |
| `projects/TODO.md` | Live task tracker |

## License
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fsiutin%2Fscheme-rs.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2Fsiutin%2Fscheme-rs?ref=badge_large)

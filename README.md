
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
Try REPL:

```
> cargo run --release --bin cli

Welcome to scheme-rs
scheme=> (+ 1 2 (* 3 4 5) 6 7 (/ 8 9 10))
Float(76.08888888888889)
scheme=> (let ((x 10) (y 20)) (+ x y))
Integer(30)
scheme=>

```

Run scheme code from a file using the interpreter:

```
# using cargo
> cargo run --release --bin scheme -- ./examples/demo_01.scm

# or executing from the build
> scheme ./examples/demo_01.scm
```

## Features

**Special forms:** `quote`, `if`, `define`, `lambda`, `let`, `cond`, `set!`, `when`, `unless`, `case`

**Builtins:** `+ - * / < <= = > >= abs append apply begin car cdr cons length list list? map max min not number? pair? print procedure? string? symbol? eq? eqv? equal? display newline string-length string-append string->symbol symbol->string boolean? zero? positive? negative? even? odd? modulo quotient remainder`

**Tail-call optimization:** Self-recursive and mutually-recursive tail calls run in constant stack space (tested to 100k depth).

**Numeric types:** Integers (`i64`) and floats (`f64`) are distinct types. Integer arithmetic preserves exactness; mixed-type operations promote to float. Division always returns float.

## Building
```

cargo build --release --bin scheme       # interpreter

cargo build --release --bin cli          # interactive shell

```

## Testing
```
cargo test                               # run all 51 tests
```


## License
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fsiutin%2Fscheme-rs.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2Fsiutin%2Fscheme-rs?ref=badge_large)

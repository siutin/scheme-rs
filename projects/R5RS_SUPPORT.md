# R5RS Feature Support Status

Reference: [R5RS Specification](https://schemers.org/Documents/Standards/R5RS/HTML/)

Legend:
- ✅ Supported
- ⚠️ Partial — works but with limitations
- ❌ Not supported

---

## 1. Lexical Conventions

| Feature | Status | Notes |
|---------|--------|-------|
| Identifiers | ✅ | Letters, digits, special chars (`+`, `-`, `*`, `/`, etc.) |
| Numbers (integer) | ✅ | i64 only — no arbitrary precision |
| Numbers (float) | ✅ | f64 |
| Numbers (rational) | ❌ | No exact/inexact rational type |
| Numbers (complex) | ❌ | No complex numbers |
| Strings | ✅ | Double-quoted, with whitespace |
| Booleans | ✅ | `#t`, `#f` |
| Characters | ❌ | No `#\a` character literal syntax |
| Comments (`;`) | ✅ | Single-line comments |
| Block comments (`#;`, `#|...|#`) | ❌ | Only `;` line comments |
| Datum comments (`#;`) | ❌ | |
| Quote shorthand `'` | ✅ | `'x` → `(quote x)` |
| Quasiquote shorthand `` ` `` | ✅ | `` `x `` → `(quasiquote x)` |
| Unquote shorthand `,` | ✅ | `,x` → `(unquote x)` |
| Unquote-splicing `,@` | ✅ | `,@x` → `(unquote-splicing x)` |
| Vectors `#(...)` | ❌ | No vector type |

---

## 2. Basic Concepts

| Feature | Status | Notes |
|---------|--------|-------|
| Variables | ✅ | Lexical scoping via environment chains |
| `define` (top-level) | ✅ | `(define x 5)` |
| `define` (internal) | ✅ | `(lambda () (define x 5) x)` |
| `define` (function shorthand) | ✅ | `(define (f x) ...)` → `(define f (lambda (x) ...))` |
| `set!` | ✅ | Mutation of existing bindings |
| Lexical scoping | ✅ | Nested env with parent chain |
| Tail calls | ✅ | TCO via trampoline-style eval loop |

---

## 3. Standard Procedures — Numbers

| Feature | Status | Notes |
|---------|--------|-------|
| `+` `-` `*` `/` | ✅ | Integer/float promotion; `/` always returns float |
| `<` `>` `=` `<=` `>=` | ✅ | Numeric comparison |
| `abs` | ✅ | |
| `min` `max` | ✅ | |
| `modulo` `quotient` `remainder` | ✅ | Integer operations |
| `zero?` `positive?` `negative?` | ✅ | |
| `even?` `odd?` | ✅ | |
| `number?` | ✅ | True for Integer and Float |
| `integer?` | ✅ | True for Integer only |
| `real?` | ✅ | True for Integer and Float |
| `rational?` `complex?` | ❌ | |
| `exact?` `inexact?` | ❌ | No exact/inexact distinction |
| `exact->inexact` `inexact->exact` | ❌ | |
| `gcd` `lcm` | ✅ | |
| `floor` `ceiling` `truncate` `round` | ✅ | Return Integer |
| `sqrt` | ✅ | Always returns Float |
| `expt` | ✅ | Integer if both args integer and result is whole |
| `exp` `log` `sin` `cos` `tan` | ✅ | Transcendental functions, return Float |
| `atan` `asin` `acos` | ✅ | `atan` supports 2-arg form (atan2) |
| `string->number` `number->string` | ✅ | With optional radix (2/8/10/16) |
| BigInt / arbitrary precision | ❌ | i64 only |

---

## 4. Standard Procedures — Lists & Pairs

| Feature | Status | Notes |
|---------|--------|-------|
| `cons` | ✅ | |
| `car` `cdr` | ✅ | |
| `list` | ✅ | |
| `length` | ✅ | |
| `append` | ✅ | |
| `null?` | ✅ | |
| `list?` | ✅ | |
| `pair?` | ✅ | |
| `reverse` | ✅ | |
| `list-ref` `list-tail` | ✅ | |
| `member` `memq` `memv` | ✅ | |
| `assoc` `assq` `assv` | ✅ | |
| `set-car!` `set-cdr!` | ❌ | Pairs are not mutable |
| `cadr` `caddr` `cddr` etc. | ✅ | All 2/3/4-level car/cdr compositions |
| Improper lists (dotted pairs) | ❌ | `(cons 1 2)` creates a 2-element list, not a pair |

---

## 5. Standard Procedures — Symbols & Strings

| Feature | Status | Notes |
|---------|--------|-------|
| `symbol?` | ✅ | |
| `string?` | ✅ | |
| `string-length` | ✅ | |
| `string-append` | ✅ | |
| `string->symbol` | ✅ | |
| `symbol->string` | ✅ | |
| `string=?` `string<?` `string>?` | ✅ | |
| `string->list` `list->string` | ✅ | Returns list of 1-char strings (no char type) |
| `substring` | ✅ | |
| `string-ref` | ✅ | Returns 1-char string (no char type) |
| `string-set!` | ❌ | Strings are not mutable |
| `string-copy` | ✅ | With optional start/end indices |
| `make-string` | ✅ | |
| `char?` `char->integer` `integer->char` | ❌ | No character type |

---

## 6. Standard Procedures — Booleans & Predicates

| Feature | Status | Notes |
|---------|--------|-------|
| `boolean?` | ✅ | |
| `not` | ✅ | |
| `eq?` | ✅ | Identity/symbol equality |
| `eqv?` | ✅ | Type-sensitive for numbers: `(eqv? 1 1.0)` is `#f` |
| `equal?` | ✅ | Deep equality |

---

## 7. Standard Procedures — Control Flow

| Feature | Status | Notes |
|---------|--------|-------|
| `if` | ✅ | With else branch |
| `cond` | ✅ | With `else` |
| `case` | ✅ | With `else` |
| `when` | ✅ | Multi-expression body |
| `unless` | ✅ | Multi-expression body |
| `and` | ✅ | Short-circuit, last in tail position |
| `or` | ✅ | Short-circuit, last in tail position |
| `begin` | ✅ | |
| `do` | ✅ | Full R5RS syntax |
| `let` | ✅ | Multi-expression body |
| `let*` | ✅ | Sequential bindings |
| `letrec` | ✅ | Recursive bindings for mutual recursion |
| Named `let` | ✅ | `(let loop ((i 0)) ...)` |
| `quasiquote` / `unquote` / `unquote-splicing` | ✅ | With shorthand syntax |
| `apply` | ✅ | |
| `map` | ✅ | Single-list only |
| `for-each` | ✅ | Single-list, for side effects |
| `call/cc` `call-with-current-continuation` | ❌ | No continuation support |
| `dynamic-wind` | ❌ | |
| `force` `delay` | ❌ | No promises/delays |

---

## 8. Standard Procedures — I/O

| Feature | Status | Notes |
|---------|--------|-------|
| `display` | ✅ | |
| `newline` | ✅ | |
| `print` | ✅ | Non-standard, prints with newline |
| `write` | ❌ | |
| `read` | ❌ | |
| `read-char` `peek-char` | ❌ | |
| `open-input-file` `open-output-file` | ❌ | |
| `with-input-from-file` | ❌ | |
| `current-input-port` `current-output-port` | ❌ | |

---

## 9. Standard Procedures — Vectors

| Feature | Status | Notes |
|---------|--------|-------|
| `vector` `make-vector` | ❌ | No vector type |
| `vector?` `vector-ref` `vector-set!` | ❌ | |
| `vector-length` `vector->list` `list->vector` | ❌ | |
| `vector-fill!` | ❌ | |

---

## 10. Standard Procedures — Evaluation

| Feature | Status | Notes |
|---------|--------|-------|
| `eval` | ❌ | No eval procedure exposed to Scheme |
| `scheme-report-environment` | ❌ | |
| `null-environment` | ❌ | |
| `interaction-environment` | ❌ | |
| `load` | ❌ | |

---

## 11. Macros

| Feature | Status | Notes |
|---------|--------|-------|
| `define-syntax` | ❌ | No macro system |
| `let-syntax` `letrec-syntax` | ❌ | |
| `syntax-rules` | ❌ | |
| `syntax-case` | ❌ | |
| `er-macro` (explicit renaming) | ❌ | |

---

## 12. Higher-Order Procedures

| Feature | Status | Notes |
|---------|--------|-------|
| `map` | ⚠️ | Single-list only — `(map f lst1 lst2)` not supported |
| `apply` | ✅ | |
| `for-each` | ✅ | Single-list, for side effects |
| `filter` | ✅ | Not in R5RS but commonly expected |
| `reduce` `fold` | ❌ | Not in R5RS but commonly expected |

---

## 13. Error Handling

| Feature | Status | Notes |
|---------|--------|-------|
| `error` | ✅ | Raises RuntimeError with message + irritants |
| `assert` | ✅ | Raises error if argument is #f |
| `raise` `with-exception-handler` | ❌ | No condition system (R6RS+) |

---

## Summary

| Category | Supported | Partial | Not Supported |
|----------|-----------|---------|---------------|
| Lexical | 9 | 1 | 5 |
| Basic Concepts | 8 | 0 | 0 |
| Numbers | 30 | 0 | 6 |
| Lists & Pairs | 15 | 0 | 4 |
| Symbols & Strings | 13 | 0 | 2 |
| Booleans & Predicates | 4 | 0 | 0 |
| Control Flow | 17 | 1 | 5 |
| I/O | 3 | 0 | 7 |
| Vectors | 0 | 0 | 6 |
| Evaluation | 0 | 0 | 5 |
| Macros | 0 | 0 | 5 |
| Higher-Order | 3 | 1 | 1 |
| Error Handling | 2 | 0 | 1 |
| **Total** | **104** | **3** | **47** |

**Overall**: ~69% of R5RS features supported (was ~58%). The interpreter covers
the core subset needed for most programs: lexical scoping, closures, tail calls,
the essential list operations (including all car/cdr compositions), conditionals,
`let`/`let*`/`letrec`/named let, `lambda`/`define` (including function shorthand),
`and`/`or`, quasiquote, `do` loops, full math (including transcendental functions),
string utilities, `for-each`/`filter`, `string->number`/`number->string`, and
`error`/`assert`. The main gaps are: no character type, no vectors, no macro
system, no continuations, no exact/inexact numeric distinction, and no I/O ports.

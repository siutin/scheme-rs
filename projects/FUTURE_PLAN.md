# Future Plan: scheme-rs Next Steps

> **Status**: Planning document — no tasks are committed or scheduled yet.
> This describes candidate work items, ordered by impact and dependency.

## Current State (after Phase 11 + Environment trait)

- 65 tests, zero warnings, ~58% R5RS coverage
- 11 phases complete (51 tasks)
- Environment trait in place — `EnvRef` everywhere, `Env::new()`/`child()`/`root()` constructors
- TCO via trampoline loop, ~2x performance from Phase 9 optimizations
- Numeric tower: Integer(i64) / Float(f64) with promotion rules
- Core R5RS: lambda, let, let*, letrec, named let, cond, case, when/unless, do, quasiquote, and/or, define shorthand

## Candidate Phases

### Phase 12: Env Trait Follow-up — `set!` via trait, cleanup

**Dependency**: Environment trait (done)

The Environment trait introduced `set()` but `eval.rs` still uses the old
`env.borrow_mut().local.borrow_mut().insert()` pattern in the `set!` handler.
This phase finishes the migration:

- Replace `set!` implementation to use `env.borrow().set(name, val)`
- Audit all remaining direct `.local.borrow_mut()` accesses outside env.rs
- Add `Env::with_capacity()` for perf-sensitive construction
- Verify zero `.local` accesses outside `env.rs`

**Effort**: Small (1-2 tasks)

### Phase 13: Performance — InternedEnv

**Status**: EXPERIMENTED, REVERTED — runtime interning is 5-9% slower.
See `projects/SPEC_INTERNED_ENV.md` for full analysis.

Runtime symbol interning (string → u32 at lookup time) adds an extra
indirection that's more expensive than the direct string → value lookup.
The intern table lookup itself requires string hashing, negating the
savings from u32 HashMap keys.

**What would work instead**:
- **Parse-time interning** — intern at parse time so AST carries u32 IDs.
  Eliminates ALL string hashing from the hot path. Large refactor (8-12 tasks).
- **Faster hasher** — replace SipHash with `ahash` or `fxhash`. Small change,
  adds dependency. Expected 20-40% speedup on HashMap operations.
- **Vec for small scopes** — `Vec<(String, DataType)>` for child envs (<10
  bindings). Linear scan faster than HashMap for small N. No dependency.

**Effort**: Medium (runtime interning — tried, failed). Parse-time interning
would be Large (8-12 tasks). Faster hasher or Vec scopes would be Small.

### Phase 14: R5RS Remaining Core

**Dependency**: None (independent of trait work)

Close more R5RS gaps that don't require architectural changes:

- `for-each` — like `map` but for side effects, single-list
- `cadr` / `caddr` / `cdddr` etc. — car/cdr compositions (generate via macro or manual)
- `string->number` / `number->string` — numeric/string conversion
- `integer?` / `real?` — numeric type predicates
- `string-copy` — copy substring
- `list-copy` — shallow copy list
- `filter` / `reduce` (not R5RS but commonly expected)
- Transcendental math: `exp`, `log`, `sin`, `cos`, `tan`, `atan`, `asin`, `acos`

**Effort**: Small-Medium (4-6 tasks, all builtins, no eval.rs changes)

### Phase 15: Vectors

**Dependency**: None

Add vector type and operations:

- Parser: `#(1 2 3)` vector literal syntax
- `DataType::Vector(Vec<DataType>)` variant
- `vector`, `make-vector`, `vector-ref`, `vector-set!`, `vector-length`
- `vector->list`, `list->vector`, `vector-fill!`
- `vector?` predicate

**Effort**: Medium (3-4 tasks: parser, type, builtins, tests)

### Phase 16: Character Type

**Dependency**: None, but enables fuller string support

Add character type:

- Parser: `#\a`, `#\space`, `#\newline` character literal syntax
- `DataType::Char(char)` variant
- `char?`, `char->integer`, `integer->char`, `char=?`, `char<?`, `char>?`
- Update `string-ref` to return `Char` instead of 1-char `String`
- Update `string->list` to return list of `Char`

**Effort**: Medium (3-4 tasks: parser, type, builtins, update string ops)

### Phase 17: Dotted Pairs & Mutable Pairs

**Dependency**: None, but changes core data representation

Currently `(cons 1 2)` creates a 2-element list, not a pair. This is
semantically wrong for R5RS. Fixing it requires:

- Distinguish `Pair` (cons cell) from `List` (proper list)
- Parser: `(1 . 2)` dotted-pair syntax
- `cons` creates a `Pair`, `car`/`cdr` work on pairs
- `set-car!` / `set-cdr!` mutate pairs
- List operations handle improper lists correctly

**Effort**: Large (5-8 tasks, touches types.rs, eval.rs, builtins.rs, parser.rs, all tests)

**Risk**: High — changes core data representation, may break many tests

### Phase 18: Macros (define-syntax / syntax-rules)

**Dependency**: None, but largest single feature

R5RS hygienic macro system:

- `define-syntax` / `syntax-rules` parser
- Macro expander pass (between parse and eval)
- Pattern matching with ellipsis (`...`)
- Hygiene (no variable capture)
- `let-syntax` / `letrec-syntax`

**Effort**: Very large (8-12 tasks, new subsystem)

### Phase 19: call/cc (call-with-current-continuation)

**Dependency**: Major architectural change

Continuations require either:
- CPS (continuation-passing style) transformation of the eval loop
- Or a stack-copying approach (capture the Rust call stack)

Either way, this fundamentally changes how `eval` works.

**Effort**: Very large (10+ tasks, rewrites eval core)

**Note**: May not be worth it — `call/cc` is rarely used in practice and
the performance cost of CPS is significant. Many real Scheme implementations
make it optional.

## Priority Recommendation

| Phase | Impact | Effort | Priority |
|-------|--------|--------|----------|
| 12: Env trait follow-up | Low (cleanup) | Small | ✅ Done |
| 13: InternedEnv | ~~High (perf)~~ | ~~Medium~~ | ❌ Tried, reverted — runtime interning is slower. Parse-time interning or faster hasher would work but are separate efforts. |
| 14: R5RS remaining core | Medium (coverage) | Small-Medium | Do next — quick wins, no architecture |
| 15: Vectors | Medium (coverage) | Medium | Do after Phase 14 — commonly expected |
| 16: Character type | Low-Medium | Medium | Do after Phase 15 — enables better strings |
| 17: Dotted pairs | Medium (correctness) | Large | Do after Phase 16 — semantically important but risky |
| 18: Macros | High (coverage) | Very large | Defer unless specifically needed |
| 19: call/cc | Low (rarely used) | Very large | Defer indefinitely |

## Not Recommended

- **BigInt** — would require `num-bigint` dependency, changes to all arithmetic,
  and the i64 range is sufficient for most use cases. Only worth it if the
  interpreter is used for exact-integer-heavy workloads.
- **Exact/inexact distinction** — adds complexity for little practical benefit
  in a teaching/learning interpreter.

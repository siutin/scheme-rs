# Spec: Environment Trait Abstraction

## Objective

Decouple the interpreter from `Env`'s concrete internal representation by
introducing an `Environment` trait. This creates a single seam between
`eval`/`Procedure`/`Function`/harnesses and the environment implementation,
enabling future work: interned-symbol environments for performance, mock
environments for unit testing, and alternative scope representations.

## Problem

`Env` was a god object:

- **79 `Rc<RefCell<Env>>` occurrences** across 7 files — every caller depended
  on `Env`'s concrete struct layout (`HashMap<String, DataType>` + `Box<Rc<RefCell<Env>>>` parent)
- **17 direct `Env { ... }` construction sites** — ad-hoc struct literals scattered
  across `eval.rs`, `builtins.rs`, with inconsistent patterns
- **Double `borrow_mut()` pattern** — `env.borrow_mut().local.borrow_mut().insert(...)`
  appeared at every `define` site, a readability and correctness hazard
- **No abstraction boundary** — swapping the scope representation (e.g. to
  interned symbols) would require touching every file

## Design

### The trait

```rust
pub trait Environment: fmt::Debug {
    fn get(&self, key: &str) -> Option<DataType>;
    fn set(&self, key: &str, value: DataType) -> bool;
    fn define(&self, key: String, value: DataType);
}
```

Three methods — the minimal interface the interpreter needs:
- `get` — look up a symbol, walking the parent chain
- `set` — mutate an existing binding (for `set!`), walking the parent chain
- `define` — create a new local binding (for `define`), shadows parent

### The type alias

```rust
pub type EnvRef = Rc<RefCell<dyn Environment>>;
```

Replaces all `Rc<RefCell<Env>>` usage. One change point if the representation
ever needs to swap (e.g. `Rc<RefCell<InternedEnv>>`).

### The constructors

```rust
Env::new(local: HashMap<String, DataType>, parent: Option<EnvRef>) -> Self
Env::root(local: HashMap<String, DataType>) -> EnvRef
Env::child(parent: EnvRef) -> EnvRef
Env::child_with(parent: EnvRef, bindings: HashMap<String, DataType>) -> EnvRef
```

Centralized env creation. Eliminates 17 ad-hoc struct literals. `root()` and
`child()` are the two most common patterns (root env from `setup()`, child env
per lambda call).

### What changed

| Before | After |
|--------|-------|
| `Rc<RefCell<Env>>` in 79 sites | `EnvRef` everywhere |
| `Env { local: Box::new(...), parent: Some(Box::new(...)) }` × 17 | `Env::new()` / `Env::child()` / `Env::root()` |
| `env.borrow_mut().local.borrow_mut().insert(name, val)` | `env.borrow().define(name, val)` |
| `env.borrow().get(&s)` | `env.borrow().get(s)` |
| `#[derive(PartialEq)]` on `Procedure` (broken — Env has no PartialEq) | Manual `PartialEq` using `Rc::ptr_eq` for env |

### What didn't change

- `Env` struct fields are still `pub` (needed by `Env::new()` impl and tests)
- The HashMap-based implementation logic is identical
- All 65 tests pass, zero warnings
- No new dependencies
- `eval` semantics unchanged — same scoping, same mutation, same TCO

## Verification

- `cargo build` — zero warnings
- `cargo test` — 65/65 pass
- `cargo bench --features "unstable"` — compiles and runs (no perf regression expected; trait dispatch is a single vtable call per `get`/`set`/`define`)

## What This Enables

### 1. InternedEnv (performance)

The SPEC_PERF bottleneck #3 is string-based symbol lookups: `env.borrow().get(&s)`
does string hashing on every variable reference. An `InternedEnv` could use
`u32` symbol IDs as keys, eliminating hashing in the hot path. With the trait,
this is a new `impl Environment for InternedEnv` — `eval.rs` doesn't change.

### 2. Mock environments (testing)

Unit tests for `eval` currently require constructing a real `Env` with a full
builtin `setup()`. A `MockEnv` implementing `Environment` could return canned
values, enabling isolated eval tests without the builtin dependency.

### 3. Alternative scope representations

- **Vec-based scopes** — for small numbers of bindings, a `Vec<(String, DataType)>`
  with linear scan is faster than `HashMap`
- **Persistent scopes** — immutable linked-list scopes for a pure-functional
  interpreter variant
- **Hybrid scopes** — Vec for locals, HashMap for globals

All are now possible without touching `eval.rs`, `builtins.rs`, `types.rs`,
`cli.rs`, `main.rs`, or `bench.rs`.

## Process Note

This refactor was committed (78eb9f7) without a formal plan review. The code
is sound — 65 tests pass, the trait is minimal, the constructors are a genuine
improvement. This spec is a retroactive document to fill the planning gap.

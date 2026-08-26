# Spec: Interned Environment (Phase 13 — Experiment)

## Status: REVERTED — runtime interning is slower than direct string lookup

## Objective

Eliminate string hashing in the variable lookup hot path by interning
symbols to `u32` IDs and using `HashMap<u32, DataType>` instead of
`HashMap<String, DataType>` for environment bindings.

## What Was Tried

### Design

1. **`SymbolTable`** — shared `Vec<String>` + `HashMap<String, u32>` mapping
   strings to compact u32 IDs. Shared across an entire env chain via `Rc`.
2. **Extended `Environment` trait** — added `get_by_id(u32)`, `set_by_id(u32, ...)`,
   and `symbol_table()` with default implementations (returning `None`/`false`).
3. **Modified `Env`** — changed `local` from `HashMap<String, DataType>` to
   `HashMap<u32, DataType>`, added `symbols: Rc<RefCell<SymbolTable>>` field.
4. **`get(&str)` optimization** — intern the string once, then walk the parent
   chain via `get_by_id(u32)` to avoid re-hashing at each scope level.

### Two variants benchmarked

**Variant 1**: `get()` always calls `symbols.borrow_mut().intern(key)` then
`get_by_id(id)`.

**Variant 2** (fast path): `get()` first tries `symbols.borrow().lookup(key)`
(immutable borrow). If the symbol is already interned (common case), use the
ID directly. Only on the cold path (new symbol) does it `borrow_mut()` + intern.

## Benchmark Results

| Benchmark | Before | Variant 1 | Variant 2 | V2 Change |
|-----------|--------|-----------|-----------|-----------|
| ackermann_2_8 | 526,960 ns | 562,712 ns | 557,894 ns | +5.9% slower |
| fact20 | 46,063 ns | 49,375 ns | 48,976 ns | +6.3% slower |
| fib25 | 509,679,723 ns | 541,900,305 ns | 535,405,752 ns | +5.1% slower |
| list_ops | 48,449,142 ns | 48,958,700 ns | 50,791,119 ns | +4.8% slower |
| sum_to_1000 | 2,333,203 ns | 2,504,322 ns | 2,492,252 ns | +6.8% slower |
| tco_loop_100k | 160,752,453 ns | 175,847,992 ns | 175,095,963 ns | +8.9% slower |

**Result**: 5-9% regression across all benchmarks. Reverted.

## Root Cause Analysis

The intern approach adds an extra indirection:

```
Current:  string → HashMap<String, V> → value     (1 string hash per level)
Interned: string → SymbolTable → u32 → HashMap<u32, V> → value  (1 string hash + 1 u32 hash per level)
```

The intern table lookup (`SymbolTable::lookup(&str)`) is itself a
`HashMap<String, u32>` lookup — it requires the same string hashing we're
trying to eliminate. The u32 hash for the local HashMap is faster, but not
enough to offset the extra intern lookup.

### Why the parent chain optimization didn't help

The `get_by_id(u32)` parent chain walk does avoid re-hashing at each level.
However, most variable lookups are either:
1. **Local hits** (found in the first scope) — the intern overhead is pure waste
2. **Shallow chain walks** (1-2 levels) — the intern overhead isn't amortized

For deep chains (3+ levels), the interned version would be faster. But most
Scheme lookups are shallow.

## What Would Work

### Parse-time interning (not attempted)

The only way to make interning faster is to intern at **parse time**, so the
AST carries `u32` IDs instead of `String`:

1. Change `AST::Symbol(String)` to `AST::Symbol(u32)`
2. Thread a `SymbolTable` through the parser
3. Change `env.get(&str)` to `env.get_by_id(u32)` in the eval hot path
4. All eval.rs pattern matches on `AST::Symbol(ref name)` use the u32 directly

This eliminates ALL string hashing from the hot path — the string is hashed
once at parse time, and never again. But it's a large refactor touching the
parser, AST type, and eval loop.

**Estimated effort**: 8-12 tasks (parser, AST, eval, builtins, tests)
**Expected gain**: 1.5-3x on symbol-heavy benchmarks

### Alternative optimizations (not attempted)

1. **Faster hasher** — Replace SipHash with `ahash` or `fxhash`. Simple change
   but adds a dependency. Expected 20-40% speedup on HashMap operations.
2. **Vec for small scopes** — Use `Vec<(String, DataType)>` for child envs
   (usually <10 bindings). Linear scan is faster than HashMap for small N.
   Root env keeps HashMap. Requires an enum or two Env types.
3. **Inline cache** — Cache the last looked-up (string, u32) pair in each Env.
   If the same symbol is looked up consecutively, skip the intern.

## Conclusion

Runtime symbol interning (Phase 13 as designed) is a net regression. The
approach should be either:
- **Parse-time interning** (large refactor, high gain) — defer to a future phase
- **Faster hasher** (small change, medium gain) — quick win if dependency is OK
- **Vec for small scopes** (medium change, medium gain) — no dependency needed

The `Environment` trait extension (`get_by_id`, `set_by_id`, `symbol_table`)
is sound and would be reused by parse-time interning. But it's not needed for
the current string-based `Env`, so it was reverted to keep the trait minimal.

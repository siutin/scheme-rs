# Work Workflow

> How to proceed a "work" — from idea to merged commit.

---

## 1. Plan Before Code

Before writing any code, produce four documents:

| Document | Purpose | Answers |
|----------|---------|---------|
| `SPEC.md` | What and why | What does this do? Why now? What's in/out of scope? |
| `CAPABILITY_MAP.md` | Structure | What modules/capabilities are involved? How do they relate? |
| `PLAN.md` | Phases | What are the phases and checkpoints? |
| `TODO.md` | Tasks | What are the concrete, verifiable tasks? |

Each task in `TODO.md` has:
- **Acceptance**: a testable condition
- **Verify**: a command to run
- **Files**: what will change
- **Status**: `[ ]` pending, `[~]` in progress, `[x]` done

> No code until these exist. If the work is small (one fix, one feature), a single `TODO.md` is enough.

---

## 2. One Task, One Commit

Each task is exactly one commit. That commit contains:

1. **The code change** — the actual fix/feature/refactor
2. **The test** — new or updated test proving it works
3. **The TODO.md update** — flip the checkbox, update status line

All three go in the same commit. Not separate. Not after. Together.

```
commit "fix: unary minus returns negation for single argument"
  ├── src/eval.rs          (the fix)
  ├── tests/spec.rs        (the test)
  └── projects/TODO.md     (checkbox flip)
```

**Commit message format:**
```
<type>: <one-line description>

<optional body explaining why, not what>
```

Types: `fix`, `feat`, `refactor`, `chore`, `docs`.

---

## 3. Verify Before Committing

Before `git add` + `git commit`, run:

```
cargo build          # compiles with zero warnings
cargo test           # all tests pass (including new one)
```

If either fails, fix it. Don't commit broken code.

---

## 4. Checkpoint at Phase Boundaries

At the end of each phase in `TODO.md`, there's a checkpoint section. Mark all checkpoint items `[x]` in the **last commit of that phase** — not in a separate docs commit later.

```
### Checkpoint: Foundation
- [x] cargo build with zero warnings
- [x] all tests pass
- [x] acceptance criteria met
```

---

## 5. Review After Each Phase

When a phase completes, review the work:

- Does the code do what the spec said?
- Are there architecture faults the plan didn't catch?
- Are there new bugs or issues discovered?

If yes, **add them as new tasks** in `TODO.md` under a new phase. Don't silently fix them. Don't defer them without writing them down. Update the plan.

---

## 6. Report at the End

When all phases are done, write a summary report:

- File: `docs/<timestamp>_<description>_report.md`
- Format: `YYYYMMDD_HHMMSS_description.md`
- Contents: what changed, before/after metrics, commit history, deferred issues, verification results

Commit the report. Then update it if later phases add more work.

---

## 7. History Hygiene

- **Never** push before the branch is clean
- **Never** leave TODO.md updates uncommitted — fold them into the task commit
- If you mess up the history (wrong commit order, missing TODO updates), rewrite it **before pushing** using `git filter-branch` or `git rebase -i`
- Commit messages explain **why**, not what
- No `Co-Authored-By` or generated-by attribution lines

---

## The Complete Cycle

```
Plan → Task → Code+Test+TODO → Verify → Commit → Next Task
                                                        │
                                                   Phase done?
                                                        │
                                              Review → New tasks?
                                                        │
                                                    Report
                                                        │
                                                    Merge
```

---

## Anti-Patterns to Avoid

- **"I'll update TODO.md later"** — no. Update it in the same commit as the work.
- **"Catch-all docs commit"** — no. Each commit should be self-contained.
- **"Commit broken, fix later"** — no. Every commit compiles and passes tests.
- **"Silently fix an architecture bug"** — no. Add it as a task, track it, commit it.
- **"Push first, clean history later"** — no. Clean before push. Rewrite is only safe pre-push.

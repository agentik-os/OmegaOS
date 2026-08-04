# The Oracle lifecycle contract (R-ORACLE-LEDGER)

An oracle owns one artifact end to end: **the ledger**, the persisted list of what the
operator asked for and where each item stands. Everything below is the lifecycle of that
ledger, from the first enumeration to a close that leaves nothing running.

The rule is `R-ORACLE-LEDGER`, compiled into `crates/omega-core/src/rules.rs` and exported
to `~/.omega/rules/` by the installer. Every dispatched Oracle receives it automatically
through `rules::agent_context_block(RuleScope::Oracle)`. Print what an Oracle actually
sees with:

```bash
omega rules context oracle
```

This page is the long form. It duplicates no other rule: `R-PLAN` owns the harness-side
plan every agent keeps, `R-VERIFY` owns adversarial verification, `R-SCOPE` owns one
writer per file, `L4` and `L6` own what "done" means. `R-ORACLE-LEDGER` binds those to the
oracle's own durable state and to its closing sequence.

---

## The state files

Everything lives under the state dir, `~/.omega/state/`.

| File | Written by | Holds |
|---|---|---|
| `oracle-<key>.progress.json` | `omega progress` | the ledger: `{tasks: [{t, s}], done, total, ts}` |
| `oracle-<key>.done.json` | `omega done` | the closing signal: status, summary, pending actions |
| `scope-<session>.json` | worker spawn | a file-scope claim: `{session, files_owned, claimed_at}` |
| `worker-<session>.done.json` | a worker's `omega done` | that worker's own signal |
| `worker-blocked-<session>.json` | a blocked session | the block-file and its reason |

`<key>` is the session name minus **one** leading `oracle-` prefix, and any numeric index
is kept: session `oracle-OmegaOS-2` writes `oracle-OmegaOS-2.progress.json` and
`oracle-OmegaOS-2.done.json` under key `OmegaOS-2`. The normalization is shared by
`progress.rs` and `OracleDoneSignal::oracle_key`, so the writer and the close-gate agree.
The Telegram bot adds its own routing fields (`chat`, `thread`, `msgId`) to the progress
file and `omega progress` preserves them on every merge-write.

---

## 1. Enumerate, in the operator's own order

Before the first dispatch, read the mission and write down one entry per distinct ask, in
the order the operator asked for them. A mission routinely carries three to six asks, and
the ones that get silently dropped are always the last. Keeping the operator's order is
what makes a dropped item visible instead of buried.

Discovered work is **appended** as new entries. It never replaces something the operator
asked for.

## 2. Persist it, do not narrate it

```bash
omega progress <session> --plan "audit code|fix N+1|merge branches|report"
```

That writes `oracle-<key>.progress.json`, and **that file is the mission state**, not the
transcript. Set the plan once, right after you build it.

A plan that lives only in prose is gone the moment the context compacts. Worse, the
operator's live checklist stays empty (the Telegram card renders this file) while the
oracle believes it is tracking the work.

## 3. Exactly one task `doing`

```bash
omega progress <session> --task "audit code" --status doing
omega progress <session> --task "audit code" --status done
```

Statuses are `todo`, `doing`, `done`, `fail`. Legal transitions are `todo` to `doing` to
`done` or `fail`. Send each transition **at the moment it happens**, never batched at the
end: a ledger updated only at the close never told anybody anything.

A task marked `done` does not silently revert. If it turns out unfinished, say so in the
report rather than quietly rewriting the ledger behind the operator's back.

## 4. Independent evidence closes a task

A worker's `done_clean` is an **input**, never the verdict (`R-VERIFY`). So:

1. name the verification command in the worker brief (`R-RUBRIC`),
2. run that command **yourself** when the worker reports,
3. only then move the ledger entry to `done`.

Until then the entry stays `doing` under the oracle's own name. Read a worker's live pane
with `omega status <worker>`.

This is also enforced at closing time: `omega done <session> done_clean` downgrades itself
to `pending` when no independent quality-gate result has been accepted for the session,
and records why in the signal's pending actions.

## 5. Resume from the file, never from memory

After a compaction, a restart, or a `claude --resume`, read the persisted plan back and
continue at the first entry that is not `done`. The memory of a plan is precisely what a
compaction destroys, which is why the plan is a file.

## 6. Closure refuses while workers run, and is safe to repeat

```bash
omega done <session> done_clean "<what was asked, what shipped, what was verified>"
```

`omega done` accepts `done_clean`, `pending`, `failed`, `blocked`, and optionally a commit
hash. A `done_clean` is **refused** while any worker of this oracle is live and unfinished:

```
done_clean REFUSED ... 2 worker(s) of this oracle still running: Proj-worker-a, Proj-worker-b.
An oracle cannot close while its workers run (zombie-worker guard).
Wait for their done signals, or close them explicitly (`omega kill <worker>`), then re-run `omega done`.
```

An oracle's workers are resolved from its own `OracleState.workers` registry, with a
fallback that sweeps live `<project>-worker-*` sessions no other oracle claims, so a lost
state file cannot silently exempt a worker from the gate. A worker counts as finished when
its registry status is terminal or it has written any done signal.

So before signaling, account for every worker you spawned: wait for its signal, or close it
deliberately with `omega kill <worker>`.

Closure is **idempotent**. The live set is recomputed on every run, so running `omega done`
a second time writes the signal again and re-kills nothing, because sessions that are
already gone are no longer in the live set.

## 7. The kill is controlled, never a sweep

On an accepted clean close, and only then:

- the **finished** worker sessions are cascaded closed with the oracle,
- each of their `scope-<session>.json` claims is released,
- the oracle's own scope claim is released,
- the panes are closed after a short delay, so the done-notifier can read the signal before
  the pane dies.

Releasing the claims is the point. A leaked `scope-<session>.json` outlives its dead
session and then rejects the **next** `omega spawn-worker` that touches the same files
(`R-SCOPE`), so healthy work gets blocked by a corpse.

The close never destroys uncommitted work. A worker commits on its own branch and the close
does not touch the branch, the worktree contents, or anything the worker left behind. A
non-clean status leaves the sessions open on purpose, so the work can be inspected.

## 8. The signal is honest

| Status | When |
|---|---|
| `done_clean` | every ledger entry is `done` and independently verified (`L4`) |
| `pending` | work remains; list exactly what, in the summary |
| `failed` | it is broken; carry the evidence |
| block-file | genuinely blocked, with the fallback already started (`L3`) |

An incomplete plan reported as `done_clean` is the failure this contract exists to stop,
and it is worse than an honest `pending`, because it ends the mission for everyone
downstream (`L6`).

---

## Reading a mission back

```bash
omega status <session>          # live pane + session status
omega rules context oracle      # exactly what an Oracle is told at dispatch
```

The progress and done files are plain JSON under `~/.omega/state/`, readable directly when
you want the ledger rather than the rendering.

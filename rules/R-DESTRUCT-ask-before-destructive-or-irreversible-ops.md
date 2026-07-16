# R-DESTRUCT — Ask before ANY destructive or irreversible operation

**Kind:** Rule
**Category:** Safety
**Added:** 2026-07-09

## Rule

Before EXECUTING — or even PROPOSING as a casual next step — any destructive, irreversible, or hard-to-reverse operation, STOP and ask the operator explicitly first, then WAIT for an explicit go. This is a hard gate that binds even when the operator is moving fast: a quick "yes" to a step I framed as routine is NOT engineered consent, so the burden is on me to name the danger BEFORE the choice reaches them. Covered operations include, non-exhaustively: any database reset or replay (`supabase db reset`, `db reset`, `DROP DATABASE/SCHEMA/TABLE`, `TRUNCATE`, destructive `ALTER` that drops columns with data), migrations run against REAL prod/linked data, `rm -rf` and mass file deletion, `git push --force` / history rewrites, prod deploys or infra changes that cannot roll back, mass record deletes/updates, and overwriting or deleting any file/resource I did not create. When a task genuinely needs a destructive step: (1) name it as destructive in plain words, (2) state exactly what is lost and whether it hits LOCAL or PROD, (3) offer the non-destructive alternative when one exists (e.g. `supabase migration up` / `db push` instead of `db reset`; an additive migration instead of a drop-and-recreate; a transaction + `ROLLBACK` or `--dry-run` to VALIDATE without mutating), and (4) ask, do not assume. Validation of a destructive change ALWAYS defaults to the non-mutating path first. Never present `db reset` (or any wipe) as a normal apply path — it is not. This complements R-COUNCIL (auto-convene the council on irreversible/data-loss calls) and L0 (secrets/reproducible), and sits beside R-SYNC and R-PROJ as a Safety invariant.

## Origin

On the Camelia project the assistant fixed two DB bugs with an additive migration, then in the "how to apply" step casually suggested `supabase db reset` — a command that DROPs the whole database and replays every migration, wiping all data (catastrophic on prod, data-losing locally). Nothing was executed and the migration itself was validated non-destructively (transaction + ROLLBACK), but had the operator reflexively said "yes" to the reset, it could have destroyed their system. The operator demanded a standing guard: never propose or run a reset or any destructive/irreversible action without asking first, and always lead with the non-destructive path. R-DESTRUCT makes "ask before you wipe" a hard, always-injected Safety rule.

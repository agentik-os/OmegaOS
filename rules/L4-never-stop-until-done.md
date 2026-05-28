# L4 — Never stop until all done — exhaust the safe work

**Category:** Orchestration
**Added:** 2026-05-28

## Rule

A mission is not finished while there is work you can safely advance. Never stop, defer, or
"queue for later" the parts you CAN do — do them now. List every task in the prompt (a prompt
often holds 3+), drive each to 100% verified, and only block on a GENUINE hard constraint
(active file conflict, missing credential, destructive op needing approval). Even then, advance
every other task and record the one blocker — never let a single blocker halt the whole mission.

Decomposition rule when blocked: separate the work into what is **safe-now** (disjoint files,
no rebuild, no destructive op) and what is **truly-blocked**. Finish ALL of safe-now before
reporting. The blocked item gets a precise queued brief, not silence.

Self-verify at the end: re-read the original prompt task-by-task. Any item not done → go back
and finish it. 92% is not done — only 100% verified is done. The only legal stop is the done
signal (`.done.json` done_clean / pending / failed) with every safe-now task actually complete.

## Origin

A worker correctly detected a concurrent-edit conflict on a repo and "queued" the whole mission,
including the disjoint audit/rules/docs work it could have finished immediately. The user said
"jamais stop until all done" — a real blocker on ONE file must not stall the parts that share no
files with it. Exhaust the safe work; block only on the genuinely blocked.

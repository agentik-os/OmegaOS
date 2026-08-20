# Workflow: compose a stack

**Trigger.** The user states an objective rather than naming an OS.

**Produces.** An ordered list of OS units, each with the reason it is present
and what it hands to the next.

## Steps

1. Restate the objective in your own words and get agreement. A stack built on a
   misread objective is worse than no stack.
2. Identify the value chain the objective sits on, using the nine groups in
   registry order: Runtime, Personal, Discover & Decide, Build, Grow, Operate,
   Own, Capital, AI & Systems.
3. Select the fewest units that close the chain end to end. Prefer a short stack
   that finishes over a complete one that stalls.
4. Order them by real data flow, using each manifest's `requires`,
   `consumes_from`, `emits_to` and `handoffs`. If unit B does not read unit A's
   output, there is no edge and no ordering constraint between them.
5. For each unit, state in one clause why it is there.
6. Name the first unit to install and the first artifact it will produce.

## Completion test

The user accepts the stack, or edits it. Every unit in the accepted stack has a
stated reason, and the order matches declared data flow rather than intuition.

## Failure paths

| Situation | Response |
|---|---|
| the objective spans several unrelated chains | propose separate stacks, do not merge them |
| a needed capability has no OS | say so plainly, do not stretch a neighbouring unit to cover it |
| the stack exceeds about seven units | cut to the chain that reaches a result soonest |

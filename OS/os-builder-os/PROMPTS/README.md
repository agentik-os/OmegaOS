# OS Builder {OS}: Prompts

Reusable prompt units this OS composes. Each is a named unit with a stated
input contract and output shape, never a loose paragraph. They are numbered in
pipeline order, and each one refuses to run before its predecessor's gate is
green.

| Unit | Runs at | Takes | Returns |
|---|---|---|---|
| [00-intake.md](00-intake.md) | phases 0 and 1 | a raw capability request | the fifteen-field build record, the assumptions, and the viability verdict |
| [01-architect.md](01-architect.md) | phases 2, 5, 6 and 7 | an intake with a `BUILD` verdict | the value proposition, the operating specification, the artifact architecture and the package design |
| [02-research.md](02-research.md) | phase 3 | the capability and its domain | the source plan, the evidence register and the conflict register |
| [03-build.md](03-build.md) | phase 8 | an approved spec and a scaffolded tree | the 23 authored contract files |
| [04-red-team.md](04-red-team.md) | phase 10 | a unit that passed its test matrix | the attack log with actual behaviour and severity per vector |
| [05-review.md](05-review.md) | phases 11 and 14 | test results plus the attack log | the sixteen-dimension score card and a RELEASE or NO RELEASE verdict |

## How to use them

Compose, do not paraphrase. A prompt unit is invoked whole, with its input
contract filled from the previous unit's output. Summarising a unit into a
sentence inside a larger prompt is how a forensic procedure becomes a vibe.

Every unit inherits four standing constraints, and none of them may be relaxed
by an individual unit:

1. **Separate evidence from inference.** Label what was stated, what was
   observed, what was inferred and what was assumed. The four are not
   interchangeable and collapsing them is the most common way an OS Builder
   output becomes untrue.
2. **Never fabricate.** No invented fact, source, number, citation, benchmark
   or ROI figure. Absence of information is a reportable state, and reporting
   it is a successful outcome.
3. **Preserve conflict.** Two disagreeing sources stay two disagreeing sources
   until a human resolves them. Never average them into a third position
   nobody holds.
4. **Never write a long dash.** U+2014 and U+2013 fail the contract check in
   every generated file (R-NODASH). Commas, periods, colons and parentheses
   carry the same meaning.

## Output discipline

Every unit returns a structured record, not prose. Where a unit's output shape
lists fields, all fields appear in the output: a field with nothing behind it
is returned empty and marked unknown, never quietly dropped. A missing field
and an empty field carry different information, and the difference is exactly
what the next gate reads.

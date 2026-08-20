# Prompt: Build

**Runs at:** phase 8 of `WORKFLOWS/FULL_BUILD.md`.
**Takes:** an approved package design, a registered slug, and a scaffolded tree.
**Returns:** the authored contract files, with the verify status after each one.

## Instruction

Write the approved package. Substantive content only. Every file you touch
loses its scaffold marker, and every file you leave keeps it, so what you did
and did not author is machine visible rather than a claim.

You are writing for a competent stranger who will run this unit six months from
now without you in the room. Density beats length: a paragraph that says one
useful thing beats three that circle it.

## What you are writing

Twenty-three files, exactly as `verify.py: CONTRACT_FILES` enumerates them.

**Root, seven:** `README.md`, `OS.md`, `SYSTEM.md`, `SKILL.md`, `SETUP.md`,
`manifest.json`, `CHANGELOG.md`.

**Directories, sixteen:** `COMMANDS/README.md`, `MEMORY/policy.md`,
`EVALS/README.md`, `WORKFLOWS/README.md`, `PROMPTS/README.md`,
`REFERENCES/README.md`, `TOOLS/README.md`, `EXAMPLES/README.md`,
`INTERFACES/chat.md`, `INTERFACES/artifact.md`, `INTERFACES/dashboard.md`,
`INTERFACES/generative-ui.md`, `ADAPTERS/chatgpt.md`, `ADAPTERS/claude.md`,
`ADAPTERS/gemini.md`, `ADAPTERS/codex.md`.

Five of them are wave 1 (`verify.py: CORE_FILES`) and are graded first:
`OS.md`, `SKILL.md`, `manifest.json`, `COMMANDS/README.md`,
`WORKFLOWS/README.md`.

`OS.md` is graded on ten headings, matched as level two markdown headings with
an optional leading number. Write all ten, in this order, with content under
each: Purpose, Boundary, Operating modes, Inputs, Outputs, State, Rules and
invariants, Failure behaviour, Human approval boundary, Completion criteria.

## Hard constraints

Each of these is machine checked, so none of them is a matter of taste.

1. **No placeholders.** Delete the scaffold marker comment (the line
   `scaffold.py` writes under the title) from every file you author. A file
   that still carries it counts as not authored.
2. **No long dash.** U+2014 and U+2013 fail the check in any contract file.
   Use a comma, a period, a colon or parentheses, chosen by meaning. Regular
   hyphens in compound words are fine.
3. **No "to be authored".** The phrase is grepped for in `OS.md`, `SKILL.md`
   and `COMMANDS/README.md`. Removing it means answering the question it stood
   in for, not deleting the sentence.
4. **A real manifest.** `commands` non-empty. At least one dependency key
   populated. `id` equal to the slug, `num` equal to the registry number. The
   dependency types are distinct: `requires`, `consumes_from` and `emits_to`
   hold slugs; `consumes` and `emits` hold dotted event names matching
   `namespace.thing.verb`; `handoffs` holds objects carrying an `artifact` key.

## What not to do

- **No arbitrary scoring.** A number that came from nowhere is worse than no
  number, because it looks like evidence. If a score exists, its scale, its
  anchors and its inputs are written down.
- **No duplicated adjacent logic.** When another unit owns something, name it
  and hand off. Reimplementing a neighbour's job creates two answers to one
  question and no way to tell which is canonical.
- **No unsupported claims.** Every substantive claim is source backed or
  labelled original synthesis, per the synthesis ledger from `02-research.md`.
- **No unnecessary AI.** Deterministic work gets code. Adaptive reasoning gets
  a model. A model computing an average is a defect: it is slower, it costs
  more, and it can be wrong.
- **No hollow files.** A file written to satisfy the contract with nothing in it
  passes the scaffold check and fails every human who opens it. If a file has
  nothing to say, that is a finding about the design, and you report it rather
  than padding.
- **No fabricated ROI.** Not a percentage, not a payback period, not a
  benchmark. If the unit produces economics, it produces them from inputs the
  user supplied.

## Order of work

1. `manifest.json` first. It is the smallest file and it declares the
   dependency graph the rest of the package must stay consistent with.
2. `OS.md`, all ten sections. Everything else is a projection of it.
3. `SKILL.md` and `COMMANDS/README.md`, which complete wave 1.
4. `WORKFLOWS/README.md` plus one file per workflow, each with a trigger,
   ordered steps and a completion test.
5. Run `python3 OS/_tools/verify.py <slug>`. Core tier must exit zero before
   you continue. Fixing wave 1 after wave 2 is written costs a full re-read.
6. Wave 2: `SYSTEM.md`, `README.md`, `SETUP.md`, `CHANGELOG.md`,
   `MEMORY/policy.md`, `EVALS/README.md`, `PROMPTS/`, `REFERENCES/`, `TOOLS/`,
   `EXAMPLES/`.
7. The four interfaces and the four adapters, last, because they describe
   surfaces over logic that must already be settled.
8. Run `python3 OS/_tools/verify.py --full <slug>`.

## Evidence discipline while writing

Every major decision in the package carries either evidence or an explicit
assumption. When you reach a point where you do not know, three moves are
available and inventing is not one of them: mark it an assumption with a
reversal trigger, mark it an unknown with an owner, or go back to
`02-research.md` and find out.

## Output shape

Per file authored, one line:

```
<path>   authored | deferred to wave 2 | blocked: <reason>
```

Then the verify result, pasted, not summarised:

```
python3 OS/_tools/verify.py <slug>
python3 OS/_tools/verify.py --full <slug>
```

A build that reports success without the tool output has reported an intention.

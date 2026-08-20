# OS Builder {OS}: Evaluations

This directory is the point of the whole unit. OS Builder's claim is not that it
writes nice packages, it is that it produces OSes **that can be graded**, and an
OS Builder whose own output cannot be graded has refuted itself.

So evaluation here runs at two levels, and keeping them apart is what makes the
verdict mean something:

- **Level 1, the meta OS itself.** Does OS Builder behave correctly when handed
  a vague request, a fake ROI demand, a duplicate capability, or a deadline?
  That is [`TEST-PLAN.md`](TEST-PLAN.md).
- **Level 2, the OS it produced.** Is the package it just built actually good?
  That is [`OS-QUALITY-RUBRIC.md`](OS-QUALITY-RUBRIC.md) and
  [`RELEASE-GATE.md`](RELEASE-GATE.md), applied to the candidate rather than to
  the builder.

Both run before anything is called done. A builder that passes its own tests and
ships a weak package has failed, and so has one that produces good packages by
being talked into whatever the requester wanted.

## Three gates, in order

| Gate | Question | Mechanism | Authority |
|---|---|---|---|
| A: CONTRACT | is the package structurally a real OS | `TOOLS/validate_os.py`, which calls `OS/_tools/verify.py` | mechanical, exit code |
| B: QUALITY | is it good enough to release | the sixteen dimension rubric, threshold applied by `TOOLS/score_os.py` | judged scores, mechanical threshold |
| C: BEHAVIOUR | does it hold up under pressure | the family suites and the twelve adversarial cases | judged against fail signatures |

Gate A first, always. Scoring a package that fails the contract is scoring a
draft, and the effort spent judging its prose is wasted the moment a `STRUCTURE`
failure sends it back.

## Gate A: the suite's own grader, not a parallel one

There is exactly one authority on whether a package satisfies the AGENTIK {OS}
contract, and it is `OS/_tools/verify.py`. OS Builder does not reimplement it.
`TOOLS/validate_os.py` imports it and calls `verify.check()`, so the checks a
candidate faces are byte for byte the checks the registered suite faces. A
second implementation would drift, and the drift would always be discovered in
the wrong direction: a candidate that passed the builder's private grader and
turns the suite red on registration.

`verify.py` grades in **two tiers**, and OS Builder uses both deliberately:

| Tier | Files | Meaning |
|---|---|---|
| CORE, wave 1 | `OS.md`, `SKILL.md`, `manifest.json`, `COMMANDS/README.md`, `WORKFLOWS/README.md` | the OS exists and is usable |
| FULL, wave 2 | all 23 contract files, adding the surfaces: `SYSTEM.md`, `README.md`, `SETUP.md`, `CHANGELOG.md`, `PROMPTS/`, `REFERENCES/`, `MEMORY/`, `TOOLS/`, `EVALS/`, `EXAMPLES/`, `INTERFACES/` and `ADAPTERS/` | the OS is releasable |

The tiers exist so that finished wave 1 work is not reported as failing for not
yet being wave 2. OS Builder maps them onto its own pipeline: **CORE is the exit
condition of phase 8 (Build)** and **FULL is a release gate item**. A build that
cannot pass CORE has no business advancing to test.

Six check classes run in both tiers: `STRUCTURE` every contract file present,
`AUTHORED` no scaffold marker left, `MANIFEST` valid JSON with thirteen required
keys, `DEPS` every dependency slug resolving and every event a dotted name,
`NODASH` no em dash or en dash anywhere, `SUBSTANCE` the ten required `OS.md`
sections with no "to be authored" surviving.

```bash
python3 OS/os-builder-os/TOOLS/validate_os.py <path>           # CORE
python3 OS/os-builder-os/TOOLS/validate_os.py <path> --full    # FULL
python3 OS/_tools/verify.py <slug> --full                      # after registration
python3 OS/_tools/graph.py --strict                            # does the graph join
```

## Gate B: the rubric and its threshold

Sixteen dimensions, 0 to 5, each score carrying evidence that cites a file. Five
are CRITICAL and admit no waiver: evidence discipline, operating logic, artifact
quality, security, testability.

Release requires every dimension at 4 or above **and** a mean of 4.3 or above.
The two conditions together are stricter than either alone: sixteen dimensions
at exactly 4 average 4.0 and are blocked, which is intended. A package that is
merely adequate everywhere is not a release.

```bash
python3 OS/os-builder-os/TOOLS/score_os.py --template > scorecard.json
python3 OS/os-builder-os/TOOLS/score_os.py scorecard.json
```

Worked example: [`scorecard.example.json`](scorecard.example.json), which clears
the threshold by 0.0125 and shows exactly how narrow the margin is.

## Gate C: behaviour under pressure

The family suites and the twelve adversarial cases in
[`TEST-PLAN.md`](TEST-PLAN.md). Every case carries a fail signature, because a
case without one is graded by whoever wants it to pass.

| Suite | Asserts |
|---|---|
| `contract` | a produced package satisfies the file contract |
| `happy` | a well formed intake produces a complete package |
| `missing` | gaps are asked about or marked `UNKNOWN`, never guessed |
| `conflict` | contradictory sources are preserved, never averaged |
| `weak-evidence` | thin evidence produces abstention, not confidence |
| `boundary` | out of scope requests hand off to a slug that resolves |
| `security` | controls arrive before features, credentials are never stored |
| `regression` | every repaired defect has a case and stays green |

## Running everything

```bash
agentik eval os-builder-os                      # all three gates
agentik eval os-builder-os --suite security     # one family
agentik eval os-builder-os --case A10           # one adversarial case
```

## The evaluation invariants

1. **A delegate's own claim of done is an input, never the verdict.** Adversarial
   case A12 exists because a completion report whose evidence is a summary
   rather than an exit code is the most consequential failure this OS can make:
   it ends the mission for everyone downstream.
2. **A defect is not repaired until it has a regression case.** Otherwise the
   same defect returns in the next build and is discovered as if for the first
   time.
3. **Do not re-score an unchanged package.** A second opinion is not a repair,
   and it is how a scorecard gets talked upward.
4. **A test that has never been red is suspect.** It is either proof of a solid
   invariant or proof that it tests nothing, and the two look identical from the
   outside. Review it rather than trusting it.
5. **Self review is not review.** The eleven judged gate items are answered by
   someone who did not write the package. The items self review passes are
   precisely the ones that were hardest to write.

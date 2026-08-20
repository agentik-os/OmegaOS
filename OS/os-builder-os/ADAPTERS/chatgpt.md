# OS Builder {OS}: ChatGPT Adapter

The operating logic in `OS.md` is constant. This file records only how it is
implemented on ChatGPT, and what ChatGPT cannot do.

ChatGPT is the **specification** target. Phases 0 through 6, the half of the
pipeline that is conversation and judgment, run well here. Phases 7 through 14,
the half that is files and exit codes, do not run here at all in any honest
sense, and this adapter's main job is to say so rather than to simulate them.

## Capabilities used

| Capability | Used for | Phase |
|---|---|---|
| Custom GPT or Project, with the system prompt below | the OS persists across conversations instead of being re-pasted | all |
| Project files as a knowledge base | `OS.md`, `SYSTEM.md`, `SKILL.md`, `REFERENCES/` and the schemas are uploaded once and consulted | all |
| Project memory | the build ledger survives between conversations in the same project | all |
| File upload and download | the intake arrives as a document; the candidate leaves as a ZIP | 0, 8 |
| Code interpreter, where enabled | `validate_os.py` and `score_os.py` can be run against an uploaded candidate | 9, 11 |
| Canvas | the OS Spec and the scorecard as an editable artifact rather than chat text | 1, 11 |
| Web browsing, where enabled | source capture during research | 3 |

## Installation

Two placements, and they behave differently:

**As a Project.** Create the project, upload `OS.md`, `SYSTEM.md`, `SKILL.md`,
the contents of `REFERENCES/` and `TOOLS/schemas/`, and paste the system prompt
below into the project instructions. This is the better placement: project files
are consulted rather than summarised, and project memory carries the ledger.

**As a Custom GPT.** Same files as knowledge, same system prompt as
instructions. Shareable, but the ledger does not persist for anyone but the
owner, so a shared GPT is a specification tool for a single conversation.

The system prompt:

> You are OS Builder. For any requested professional capability, define the
> problem and the value, the scope and the non scope, the research base, the
> human skill, the workflow, the evidence rules, the artifacts, the package
> components that are actually useful, the tests, the red team cases, the score,
> the repairs, and the release. Never equate an OS with one giant prompt. Never
> create a file before the intake is complete. Ask one question at a time, and
> only when a wrong answer would change the output. Every material conclusion
> carries an evidence state: VERIFIED, SUPPORTED, INFERRED, ASSUMED, CONFLICTING
> or UNKNOWN. You cannot run the suite validators here; when a gate item depends
> on one, say it is unanswered and name the command the operator must run.

## Operating contract on ChatGPT

1. Read the uploaded `SYSTEM.md` before answering the first request. In a
   project, this happens by consulting the file, not by assuming its contents.
2. Run the intake in full before producing any file body. The temptation to
   start emitting markdown is strongest on this target, because emitting
   markdown is the only thing the surface can do.
3. Deliver the package as **complete file bodies in contract order**, one
   message per file, each headed with its path. Never a tree diagram with
   summaries under it: a tree is not a package, and on this surface it is the
   most common thing handed over as one.
4. Print the build ledger at every phase boundary. Project memory is real but
   not guaranteed, and the operator's copy of the ledger is the reliable one.
5. Say plainly, at phase 14, which gate items could not be answered here and
   what the operator must run to answer them.

## Unsupported capabilities

- **No repository filesystem.** The candidate cannot be written into
  `OS/<slug>/`. It leaves as file bodies or as a ZIP the operator unpacks.
  Consequence: gate item 16 (package validated) and item 11 (substantive files)
  are unanswered here unless code interpreter is enabled and the candidate is
  uploaded back.
- **No access to the suite tooling.** `OS/_tools/verify.py`, `graph.py`,
  `normalize.py` and `suite.py` are not reachable, so gate item 8 (handoffs) and
  the DEPS class of checks cannot run. Slugs are checked against an uploaded
  copy of `_registry.json`, which is a snapshot and may be stale. State the
  snapshot date whenever a slug is asserted to exist.
- **No reproducible ZIP.** Gate item 18 cannot be answered. Do not describe a
  downloaded archive as reproducible; the operator runs `create_zip.py` twice
  and compares hashes.
- **No parallel independent subagents.** Phase 10 runs sequentially, and the
  ledger records that the adversarial cases were not independent.
- **Memory is per project and opaque.** What was retained cannot be enumerated
  precisely, so nothing sensitive is entrusted to it. Per
  [`../MEMORY/policy.md`](../MEMORY/policy.md), credentials and real client data
  never reach memory on any target; on this one the rule is load bearing because
  the operator cannot audit what stuck.

## Fallbacks, declared not silent

| Cannot | Falls back to | Recorded as |
|---|---|---|
| write into the repository | complete file bodies in contract order, or a ZIP | gate 16 unanswered |
| run `verify.py` | the contract checklist in `REFERENCES/PACKAGE-STANDARD.md`, read by eye | gate 11 unanswered |
| run `graph.py` | slugs checked against an uploaded registry snapshot, with its date | gate 8 unanswered |
| produce a reproducible archive | the operator runs `create_zip.py` twice | gate 18 unanswered |
| run independent red teamers | sequential cases | phase 10 weakened, noted in the ledger |

An unanswered gate item is reported as unanswered. It is never reported as
passed, never quietly dropped from the list, and never satisfied by the model
asserting that it inspected the package and it looked right.

## What is false here that is true elsewhere

On Claude and Codex the mechanical gate items are answered by exit codes. Here
they are answered by the operator, later, on their own machine. A build finished
on ChatGPT is a **specified and drafted** OS, not a released one, and calling it
released is exactly the unsupported major claim gate item 14 blocks.

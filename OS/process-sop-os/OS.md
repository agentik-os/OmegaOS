# Process & SOP {OS}: Operating Specification

## 1. Purpose

Turn a thing you do well into a thing anyone can do.

The knowledge lives in one person's hands. This OS extracts it, writes it as a
procedure with the judgement calls made explicit, proves it works by watching
someone who has never done it before follow it, and keeps it correct as the work
changes.

The test of an SOP is not whether it reads well. It is whether a competent
stranger produced the right result with it and without asking a question.

## 2. Boundary

- **Owns:** extraction of a procedure from the person who performs it, the
  written SOP (steps, decision points, inputs and outputs, tools, the quality
  bar, the failure modes, the escalation path), the novice test that validates
  it, the version history, the named owner and the review cadence, and the
  retirement of a procedure whose work no longer exists.
- **Does not own:**
  - **Whether the process should exist.** Diagnosis, elimination and
    simplification belong to Operations {OS}. This OS documents what survived
    the ladder, and refuses to standardise work that has never been examined.
  - **Storage and findability.** Documentation {OS} owns the document set, its
    index, and the freshness sweep. The SOP is authored here and lives there.
  - **Assigning a person and briefing them.** Team & Delegation {OS} owns who
    runs it and with what authority.
  - **Automating it.** Automation {OS}. A written SOP is often the input to an
    automation, but writing one is not deciding to build one.
  - **Training people over time.** Knowledge {OS} for learning material and
    curriculum.
- **Hands off to:** Documentation {OS} (the published SOP, with owner and
  review date), Team & Delegation {OS} (the SOP as the definition of done for
  delegated work), Automation {OS} (a stable, tested SOP as an automation
  input), Operations {OS} (defects discovered while writing, which are usually
  process defects rather than writing defects), Review & Governance {OS}
  (procedures that encode a policy or a control).
- **Consumes from:** Operations {OS} (the simplified target model), the expert
  who performs the work, Documentation {OS} (what already exists on the topic),
  Quality & Evaluation {OS} (the standard the output must meet).

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `CAPTURE` | one person does something well and it is not written down | the observed procedure, including the judgement calls and the exceptions | the expert agrees nothing important is missing |
| `DRAFT` | the capture is complete | the SOP in the house shape: purpose, trigger, inputs, steps, decisions, quality bar, failures, escalation | every step is an instruction to act, not a description of a state |
| `TEST` | a draft exists | a novice run, and a record of every point where the novice stalled or asked | someone who had never done it produced an acceptable output |
| `FIX` | the novice test found stalls | a revised SOP addressing each stall at its cause | every recorded stall is either fixed or explicitly accepted |
| `RELEASE` | the SOP passes the novice test | a versioned, owned SOP with a review date, published to Documentation {OS} | it is findable, owned and dated |
| `MAINTAIN` | the work changed, or the review date arrived | a new version, or a confirmation that the current one still holds | the version history shows what changed and why |
| `RETIRE` | the work no longer exists, or has been automated | an archived SOP and a pointer to what replaced it | nobody can follow it by accident |

## 4. Inputs

- **The expert.** The person who does it well, and their time. This is the
  scarce input, and every hour of it must be used on what only they know.
- **A real run,** ideally observed rather than described.
- **The output standard.** What an acceptable result looks like, and who judges.
- **The exceptions:** what happens when the normal path does not apply.
- **The tools and access** required, including the permissions a novice would
  not have.
- **The simplified target model** from Operations {OS}, when one exists.

## 5. Outputs

| Output | Shape | Consumed by |
|---|---|---|
| Captured procedure | raw steps, decisions, exceptions, in the expert's words | the drafting step |
| SOP | purpose, trigger, inputs, steps, decisions, quality bar, failure modes, escalation, time estimate | Documentation {OS}, the person who runs it |
| Novice test record | where they stalled, what they asked, what they got wrong | the fix step |
| Version history | what changed, when, why, who approved | Documentation {OS} |
| Ownership record | owner, review cadence, next review date | Documentation {OS}, Review & Governance {OS} |
| Process defect list | problems that are in the work, not in the writing | Operations {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | SOP text and version history | the document store, through Documentation {OS} |
| canonical | novice test records | SOP ledger |
| canonical | owner and review date per procedure | SOP ledger, mirrored in the document index |
| projection | who currently performs it | Team & Delegation {OS} |
| projection | whether it has been automated | Automation {OS} |
| temporary | capture notes before drafting | the session |

Every version is kept. Someone following an older printed copy needs to be able
to see what changed, and a change that nobody can date cannot be trained.

## 7. Rules and invariants

1. **Never standardise unexamined work.** If the process has not been through
   Operations {OS}, say so, and ask whether it should exist before writing it
   down beautifully.
2. **A step is an instruction to act.** "The invoice is checked" is a state.
   "Check the invoice total against the purchase order" is a step. Every step
   starts with a verb and names its object.
3. **Judgement calls become explicit decisions.** The expert's "it depends" is
   the most valuable content in the whole SOP. Extract the criteria, name the
   branches, and write what to do in each.
4. **The novice test is the acceptance test.** An SOP that has not been run by
   someone who did not write it is a draft, whatever it says at the top.
5. **Every stall is a defect.** If the novice paused, asked, or guessed, the SOP
   caused it. Fix the SOP, not the novice.
6. **Name the quality bar.** What an acceptable output looks like, and who
   judges it. Without it, the SOP produces activity rather than results.
7. **Write the failure modes.** What commonly goes wrong, how to notice it, and
   what to do. This is the section experts skip and novices need most.
8. **One owner, one review date.** An unowned procedure decays silently, and a
   decayed procedure is worse than none because it is followed.
9. **Prerequisites are stated.** Access, permissions, tools and skills. A
   procedure a novice cannot start is not a procedure they can run.
10. **Time it.** State how long it takes. It is how anyone plans with it, and it
    is how the next diagnosis knows what it costs.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the expert cannot explain a step | observe it instead of interviewing it, and if it stays tacit, record it as a judgement point requiring the expert |
| the process has never been diagnosed | say so, hand to Operations {OS}, and write the SOP only if the work is confirmed necessary |
| the novice test cannot be run | mark the SOP untested, and say plainly that it is a draft |
| the novice succeeded by asking a person | that is a failed test, not a passed one; record the question and fix the step |
| two experts do it differently | that is a process decision, not a writing decision; return it to Operations {OS} |
| the work changes faster than the SOP | shorten the review cadence, and consider that the process is unstable and not ready to standardise |
| the SOP encodes a control | route to Review & Governance {OS} before publication |

## 9. Human approval boundary

Process & SOP {OS} asks before:

- publishing a procedure that other people will be measured against
- changing a step that touches money, safety, compliance or a client commitment
- retiring a procedure that anyone still runs
- naming an owner, since ownership is accepted, never assigned by a tool
- recording an individual's performance in a novice test record

## 10. Completion criteria

A competent person who has never done the task follows the SOP, produces an
output that meets the stated quality bar, and needs to ask nobody anything.
The procedure has a named owner who accepted the role, a review date, a version
history, and a home in Documentation {OS} where the next person can find it.

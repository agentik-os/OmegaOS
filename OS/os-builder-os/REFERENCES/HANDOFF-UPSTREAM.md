# Upstream Handoff

What OS Builder accepts as an intake, and how anything it accepts is normalised
into the one shape phase 1 can work with.

## Accepted intake forms

| Form | Looks like | Typical gap |
|---|---|---|
| direct capability request | "I want an OS that does X" | no operator, no artifact, no boundary |
| academy or curriculum capability | a taught skill someone wants operationalised | teaching order is not operating order |
| existing OS rebuild | a unit already in the suite that needs re-authoring | its real boundary differs from its stated one |
| workflow | a documented process someone runs today | exceptions hidden behind the happy path |
| source material | a book, a corpus, a set of notes | knowledge with no decision attached |
| blueprint | a product definition pack from Blueprint {OS} | product scope, not capability scope |
| pipeline output | a request arriving from another OS | the requesting OS's framing, not the operator's |

All seven normalise into the same intake record. None of them is allowed to
proceed as it arrived: an unnormalised intake is how a build ends up producing a
package for a capability nobody named.

## The intake record

Twelve fields. Every one is filled before phase 1 opens. A field that cannot be
filled from the request is asked for, one question at a time, and only when a
wrong answer would change the output.

```
NAME              what the OS is called, and its slug
CAPABILITY        the repeatable professional capability, in one sentence
OPERATOR          who runs it, and what they already know
ENVIRONMENT       where it runs: which adapters must work
PROBLEM           what goes wrong today, concretely
DESIRED OUTCOME   what is true afterwards that is not true now
PRIMARY ARTIFACT  the one thing it produces that a person keeps
ADJACENT SYSTEMS  which existing OSes touch this, and where the seam is
CONSTRAINTS       time, budget, tooling, jurisdiction, house rules
SECURITY LEVEL    the highest sensitivity class it will handle
SCOPE             what it does
NON SCOPE         what it deliberately refuses to do
```

`PRIMARY ARTIFACT` and `NON SCOPE` are the two fields most often skipped and the
two that most reliably predict a failed build. An OS with no primary artifact is
a chat persona. An OS with no non scope will absorb every adjacent request until
its boundary means nothing.

## Normalisation rules

1. **A capability, not a topic.** "Pricing" is a topic. "Set and defend a price
   for a specific offer, with evidence" is a capability. Topics do not have
   completion criteria, so they cannot be graded.
2. **The operator is a person, not a role abstraction.** What they know decides
   how much the skill layer has to teach and how much can be assumed.
3. **The problem is stated as an observable failure**, not as an absence of the
   solution. "We have no pricing OS" is not a problem statement.
4. **Adjacent systems are resolved against the real registry.** Every named
   neighbour must be an actual slug in `OS/_registry.json`. A handoff to an OS
   that does not exist is caught by `validate_os.py` at DEPS, but catching it at
   intake is cheaper than catching it at release.
5. **The security level is the maximum, not the typical.** An OS that usually
   handles `internal` data and occasionally handles `sensitive` data is a
   `sensitive` OS.

## The intake gate

Phase 1 does not open until all three are true:

- Every one of the twelve fields is filled, or explicitly marked `UNKNOWN` with
  the reason and what would resolve it.
- The capability can be explained without mentioning folders, prompts or models.
  If the explanation needs the word "prompt", it is not yet a capability.
- The decision "should this be an OS at all" has been answered YES with reasons.

## Should this be an OS at all

The intake's most valuable output is sometimes a refusal. The tree:

```
Is it a repeatable professional capability?
  NO  -> do not build an OS.
  YES -> does it involve recurring decisions, a workflow, or artifacts?
    NO  -> a prompt, a checklist, a template or a skill is the right size.
    YES -> can it be bounded?
      NO  -> split the capability and re-enter with one half.
      YES -> does reusable operating infrastructure add value here?
        NO  -> ship the lighter artifact.
        YES -> BUILD OS.
```

Two extra checks the tree does not cover, both of which have killed real builds:

- **Adjacent duplication.** If an existing OS in the registry already owns 70
  percent of this capability, the answer is a handoff and an extension to that
  OS, not a new unit. Two units that overlap are worse than one unit that is too
  big, because neither knows which one owns the decision.
- **One OS, one core capability.** A request carrying two capabilities produces
  two intake records, built in dependency order, joined by a declared handoff.

## What OS Builder never accepts

- A request whose only content is a model instruction. A giant prompt is not an
  OS and rewriting it into folders does not make it one.
- A capability the requester cannot describe a single real decision inside.
- A rebuild request that does not say what is wrong with the current unit.
- An intake carrying live credentials, real client data, or an unredacted
  corpus. Those are returned before phase 1, per
  [`SECURITY.md`](SECURITY.md).

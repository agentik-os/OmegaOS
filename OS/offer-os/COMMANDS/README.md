# Offer {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install offer-os` | Installs this OS into your environment | Once, first |
| `agentik configure offer-os` | Collects the minimum context it needs | After install |
| `agentik run offer-os` | Starts the OS | Every session |
| `agentik doctor offer-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update offer-os` | Updates to the latest version | When a release lands |
| `agentik eval offer-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/offer` | Opens the OS and reports the state of every offer | nothing | the offer set with lifecycle state per offer |
| `/offer-draft` | Drafts a candidate offer from the claim and the job to be done | positioning claim, job to be done | a draft offer definition |
| `/offer-scope` | Draws the scope boundary and gives each deliverable an acceptance form | a draft offer | a scope boundary artifact |
| `/offer-exclude` | States what the offer deliberately does not cover | a scope boundary | the exclusion list |
| `/offer-deliverables` | Enumerates the deliverables and their acceptance conditions | a draft offer | the deliverable list with acceptance forms |
| `/offer-guarantee` | Designs a guarantee and models its worst case cost | scope boundary, fulfilment cost model | a guarantee record with a cost ceiling |
| `/offer-proof` | Assembles the proof set behind each promise | the promise list | proof items, each with a source |
| `/offer-cost-check` | Reconciles the offer against what fulfilment actually costs | Delivery fulfilment cost model | a viable or not viable verdict with the gap named |
| `/offer-stress-test` | Runs the offer against real objections | the offer, an objection set | an objection log and the revisions it forces |
| `/offer-review` | Audits the live offer set for contradiction and drift | the live offer set | findings, each with the contradicting statement quoted |
| `/offer-publish` | Publishes a versioned offer after human approval | an approved offer definition | a live, versioned offer emitted to consuming units |
| `/offer-retire` | Retires an offer and migrates the customers on it | an offer id | a retirement record and a per customer migration path |

---

### `/offer`

```
/offer
```

Opens Offer {OS} and reports every offer it knows about, with its lifecycle
state: draft, live, frozen or retired. A frozen offer is named as frozen with
the reason, never quietly listed as live.

**When to reach for it:** at the start of any session where you are not sure
what is currently being sold.
**Returns:** the offer set, one line per offer, plus any blocking condition
(missing claim, missing cost model, unresolved contradiction).

### `/offer-draft`

```
/offer-draft "onboarding for teams migrating off spreadsheets"
```

Turns a claim and a job to be done into a candidate offer: promise, named
outcome, first pass at deliverables. It refuses to run when Positioning {OS}
has no claim, because an offer drafted before the claim is a guess about what
somebody might buy.

**When to reach for it:** when a new thing is about to be sold.
**Returns:** a draft offer definition, marked draft, not emitted to anyone.

### `/offer-scope`

```
/offer-scope offer-2
```

Draws the boundary. Each deliverable gets an acceptance form: the condition
under which it counts as done, checkable without a conversation.

**When to reach for it:** immediately after a draft, before anyone quotes it.
**Returns:** the scope boundary artifact that Sales {OS} and Delivery &
Customer Success {OS} will both check specific requests against.

### `/offer-exclude`

```
/offer-exclude offer-2
```

Writes down what the offer deliberately does not cover. An offer with no
stated exclusion has an unbounded scope, so this command is a gate, not a
suggestion: the offer cannot leave scoping without at least one exclusion.

**When to reach for it:** every time, before publishing.
**Returns:** the exclusion list, and a warning for each exclusion a buyer is
likely to assume is included.

### `/offer-deliverables`

```
/offer-deliverables offer-2
```

Enumerates what is handed over and when. Rewrites any deliverable phrased as
seller activity into buyer outcome where the outcome is the real promise.

**When to reach for it:** while scoping, and again after any stress test that
changed the offer.
**Returns:** the deliverable list with an acceptance condition per line.

### `/offer-guarantee`

```
/offer-guarantee offer-2 --remedy "full refund within 30 days"
```

Designs the guarantee and models its worst case cost against the fulfilment
cost model. It computes the worst case, not the expected case, and writes the
ceiling above which the guarantee is withdrawn from new sales.

**When to reach for it:** when a guarantee is being considered, and again
whenever the fulfilment cost model changes.
**Returns:** a guarantee record: condition, remedy, worst case cost, ceiling.
If the worst case cannot be modelled, it abstains and names the missing
quantity rather than producing a number.

### `/offer-proof`

```
/offer-proof offer-2
```

Maps every promise to the evidence that makes it credible, and removes any
proof item that cannot name its source.

**When to reach for it:** before the offer meets a prospect.
**Returns:** the proof set, one item per promise, each with a source and a
consent status for anything naming a customer.

### `/offer-cost-check`

```
/offer-cost-check offer-2
```

Pulls the fulfilment cost model from Delivery & Customer Success {OS} and
compares it against the offer economics.

**When to reach for it:** before publishing, and on every review cycle.
**Returns:** viable or not viable, with the gap quantified. Not viable freezes
the offer for new sales and produces three options: narrow the scope, change
the guarantee, or hand the problem to Pricing {OS}.

### `/offer-stress-test`

```
/offer-stress-test offer-2
```

Runs the offer against the objections it will actually meet, sourced from
Sales {OS} where real ones exist.

**When to reach for it:** before publishing anything.
**Returns:** an objection log, each entry answered or marked as forcing a
revision. An offer that survived only objections invented by its own author is
reported as untested.

### `/offer-review`

```
/offer-review
```

Audits the live offer set against the positioning claim, the scope boundary
and the cost model, looking for drift and contradiction.

**When to reach for it:** on a cadence, and after any positioning change.
**Returns:** findings, each quoting both contradicting statements. It changes
nothing on its own.

### `/offer-publish`

```
/offer-publish offer-2
```

Publishes a versioned offer and emits it to Pricing {OS}, Sales {OS}, Revenue
{OS}, Delivery & Customer Success {OS} and Content {OS}.

**When to reach for it:** only after a human has approved the exact wording.
**Returns:** the live offer definition with a version stamp, plus the list of
units it was emitted to. It refuses to run on an unapproved draft, on an
unresolved contradiction with the claim, or with no cost model present.

### `/offer-retire`

```
/offer-retire offer-1 --effective 2026-10-01
```

Withdraws an offer and handles the customers on it.

**When to reach for it:** when an offer is being replaced or discontinued.
**Returns:** a retirement record with a named migration destination for every
live customer. It refuses to complete while any live customer has no
destination, because retiring an offer does not retire the obligation.

## Command summary

| Command | Does |
|---|---|
| `/offer` | the state of every offer |
| `/offer-draft` | claim plus job to be done, into a candidate offer |
| `/offer-scope` | the boundary and the acceptance forms |
| `/offer-exclude` | what it deliberately does not cover |
| `/offer-deliverables` | what is handed over, and when it counts as done |
| `/offer-guarantee` | the guarantee, and what it costs at worst |
| `/offer-proof` | the evidence behind each promise |
| `/offer-cost-check` | can this be fulfilled at a sane cost |
| `/offer-stress-test` | the objections, answered or absorbed |
| `/offer-review` | drift and contradiction in the live set |
| `/offer-publish` | approved wording, versioned and emitted |
| `/offer-retire` | withdrawal, with a path for every live customer |

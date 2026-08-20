# Business Model {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install business-model-os` | Installs this OS into your environment | Once, first |
| `agentik configure business-model-os` | Collects the minimum context it needs | After install |
| `agentik run business-model-os` | Starts the OS | Every session |
| `agentik doctor business-model-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update business-model-os` | Updates to the latest version | When a release lands |
| `agentik eval business-model-os` | Runs its evaluation suite | Before trusting it |

## OS commands

### `/business-model [<idea, plan or business>]`

The root command. Given an idea or a plan with no explicit model, it runs `MAP`
and returns the canvas. Given an existing model, it reports what is missing and
routes you to the mode that fills it. Given a running business, it starts from
the delivery cost actually incurred rather than from the pitch.

**When to use it:** when you do not know which piece is missing yet.
**Returns:** either the canvas, or the ordered list of what has to exist before
the economics can be computed, with the first step named.

### `/canvas`

Fill the model blocks: segments, value proposition per segment, delivery
mechanism, channels, revenue mechanics, cost structure, key resources and
external dependencies.

**When to use it:** at the start, and again whenever the business has changed
shape and the written model no longer describes it.
**Returns:** the canvas, with every unknown marked as unknown rather than filled
with something plausible, and every segment traced to a value proposition and a
way it pays.

### `/value-map [<segment>]`

Per segment: the job they are hiring you for, the alternative they use today
(including doing nothing), what you deliver, and why it beats that alternative
for that segment specifically.

**When to use it:** when the value proposition is one generic sentence covering
three different kinds of buyer.
**Returns:** one value map per segment, with the alternative named. A segment
whose alternative cannot be named is flagged: it usually means the segment is
invented.

### `/revenue-mechanics`

Specify every revenue line: what triggers the payment, who pays it, at what
frequency, under what commitment, and how it expands or churns.

**When to use it:** when "we charge a subscription" is the entire description of
how money arrives.
**Returns:** one row per revenue line with trigger, payer, frequency,
commitment, expansion path and churn behaviour. Any line whose trigger cannot be
stated is flagged as not yet a revenue line.

### `/cost-structure`

Build the cost base and split it fixed versus variable against the named unit.
Includes the costs people leave out: support time, onboarding effort, failed
deliveries, refunds, payment fees, and human hours nobody logs.

**When to use it:** before any margin number is quoted to anybody.
**Returns:** the cost structure with each figure origin-labelled, the fixed base
per period, and the variable cost per unit that gross margin will be computed
against.

### `/unit-economics [--unit <unit>]`

Compute the economics of one unit: contribution, cost of acquisition per
channel, retention, payback period, lifetime value.

**When to use it:** once the unit is countable and the cost structure exists.
**Returns:** the full unit economics with an origin label on every input. In a
recurring model with no retention figure, it stops before lifetime value and
reports the retention the model would need instead of inventing one.

### `/breakeven`

Compute the volume per period at which the model stops losing money, and compare
it against the volume the pipeline can plausibly produce.

**When to use it:** before committing capital, headcount or a launch date.
**Returns:** the breakeven volume, the fixed base it was computed against, the
plausible pipeline volume with its source, and the gap between them stated in
units rather than in adjectives.

### `/viability [--bar <margin | payback | return>]`

Issue the verdict against a bar you state first: VIABLE, VIABLE UNDER
CONDITIONS, NOT VIABLE, or INSUFFICIENT DATA.

**When to use it:** when someone is about to decide to build, fund or continue.
**Returns:** the verdict, the bar it was measured against, the conditions
attached if any, and what specifically would have to change by how much to move
it. It refuses to issue a verdict against a bar that was not stated before the
assessment ran.

### `/variants [<shape> <shape> [<shape>]]`

Compare two or three model shapes on the same unit, the same bar and identical
assumptions. For example subscription versus per-project, self-serve versus
assisted, take rate versus listing fee.

**When to use it:** when the argument is about which model to run rather than
whether the business works.
**Returns:** the comparison table, what must be true for each variant to win,
which assumptions genuinely differ by shape (each registered as a claim), and
what is identical between them and therefore not a reason to choose.

### `/stress`

Move each load-bearing input until the model turns negative.

**When to use it:** immediately before committing to a model, and any time the
numbers look comfortable.
**Returns:** the break value per input, how far it is from the current value,
how likely that move is, and the inputs ranked by fragility. The top of that
ranking is the number carrying your business.

### `/model-audit <artifact>`

Inspect a model you inherited: a deck, a spreadsheet, a plan, a previous team's
assumptions.

**When to use it:** before repeating any number from it out loud.
**Returns:** per number, what was measured and what was chosen, plus the
standard defect sweep: a unit that is not countable, a lifetime value with no
retention behind it, a channel entered at zero acquisition cost, a margin
computed on intended rather than incurred cost, and a breakeven the pipeline
cannot reach.

## Command summary

| Command | Does | Returns |
|---|---|---|
| `/business-model` | entry point: idea or business to a written model | the canvas, or the ordered list of what is missing |
| `/canvas` | fill the model blocks | canvas with unknowns marked as unknown |
| `/value-map` | job, alternative and why you win, per segment | one value map per segment |
| `/revenue-mechanics` | how money actually arrives | trigger, payer, frequency, commitment, per line |
| `/cost-structure` | what delivery really costs | fixed base and variable cost per unit, origin-labelled |
| `/unit-economics` | the economics of one unit | contribution, acquisition cost, retention, payback, lifetime value |
| `/breakeven` | when it stops losing money | breakeven volume versus plausible pipeline volume |
| `/viability` | does it clear the stated bar | VIABLE, VIABLE UNDER CONDITIONS, NOT VIABLE, INSUFFICIENT DATA |
| `/variants` | compare model shapes fairly | comparison on identical assumptions, with what must be true for each |
| `/stress` | find what breaks it | break value per input, ranked by fragility |
| `/model-audit` | check an inherited model | measured versus chosen, per number, plus the defect sweep |

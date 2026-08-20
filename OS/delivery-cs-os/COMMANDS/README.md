# Delivery & Customer Success {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install delivery-cs-os` | Installs this OS into your environment | Once, first |
| `agentik configure delivery-cs-os` | Collects the minimum context it needs | After install |
| `agentik run delivery-cs-os` | Starts the OS | Every session |
| `agentik doctor delivery-cs-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update delivery-cs-os` | Updates to the latest version | When a release lands |
| `agentik eval delivery-cs-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/delivery` | Opens the delivery portfolio | every live engagement | health, promises outstanding, and what needs a decision |
| `/handoff-client` | Runs the sales to delivery transfer | the signed commitment and the promises made | an itemised promise register with a scope verdict per promise |
| `/onboard-client` | Creates the onboarding plan | an accepted handoff | a plan with a first value date and named owners |
| `/success-plan` | Defines outcomes and measures | discovery with the customer | the customer's own measures, agreed in writing |
| `/client-plan` | Creates milestones and governance | the success plan | milestones, cadence, escalation path and decision rights |
| `/client-update` | Drafts a transparent status update | delivery state and risk | an unsent update stating risk with options, before the deadline |
| `/scope-change` | Processes a change request | a request outside the agreed scope | a priced change or a stated refusal, never silent absorption |
| `/client-risk` | Creates an escalation plan | a risk to date, health or trust | the escalation path, the ask, and the point of no return |
| `/adoption` | Builds an adoption intervention | usage and health signals | an intervention with a measurable target and a review date |
| `/value-proof` | Compiles outcome evidence | measures, usage and business results | attributed value, and an explicit list of what is unattributable |
| `/qbr` | Prepares a business review | value proof and the success plan | a review the customer could read line by line without flinching |
| `/renew-client` | Prepares the renewal recommendation | health, adoption and value signals | a recommendation with its signals, sent to Revenue {OS} to decide |
| `/case-study` | Requests and builds a case study | a proven outcome | scoped written consent, then a draft the customer sees first |
| `/offboard` | Closes an engagement responsibly | a decision to end | a handover pack, data control, and the learning captured |

---

## Portfolio and start

### `/delivery`

Open the delivery portfolio: every live engagement, its health, the promises
still outstanding, and the one thing that most needs a decision.

```text
/delivery
/delivery "Meridian Health"
```

**When to reach for it:** at the start of a delivery week.
**Returns:** health per engagement with the raw signals behind each score, and a
list of decisions waiting on a human.

### `/handoff-client`

Run the transfer from Sales {OS}. This is the most important command in the OS,
because everything downstream is built on it.

```text
/handoff-client "Meridian Health"
```

**When to reach for it:** after the post-payment gate clears
(`contract.signed`, then `payment.reconciled`), and before any work starts.
**Returns:** the promise register: every promise made in the sale, itemised,
each carrying a scope verdict against the Offer {OS} boundary. A promise that is
not in the register was not sold. A promise outside the scope boundary is
returned as an escalation, not scheduled as work.

### `/onboard-client`

Build the onboarding plan, aimed at the first thing the customer will actually
feel.

```text
/onboard-client "Meridian Health"
```

**When to reach for it:** immediately after the handoff is accepted.
**Returns:** a plan with a first value date, owners on both sides, and what the
customer must supply for each step.

## Planning and delivery

### `/success-plan`

Define what success means, in the customer's words, with measures they agree to.

```text
/success-plan "Meridian Health"
```

**When to reach for it:** after discovery, before the delivery plan hardens.
**Returns:** outcomes and measures with agreed baselines and target values, and
written agreement from the customer. Success defined internally is not a success
plan, it is a wish.

### `/client-plan`

Turn the success plan into milestones and governance: cadence, escalation path,
decision rights.

```text
/client-plan "Meridian Health"
```

**When to reach for it:** once the success plan is agreed.
**Returns:** milestones with acceptance criteria, the meeting cadence, and who
can decide what on each side.

### `/client-update`

Draft the status update, with risk stated early and with options attached.

```text
/client-update "Meridian Health" --period week
```

**When to reach for it:** on the agreed cadence, and immediately when a risk
appears.
**Returns:** an unsent draft. Nothing customer-facing is sent without an
explicit human approval on the exact text. Risk is communicated before the
deadline, with options, never afterwards as an explanation.

## Change, risk and adoption

### `/scope-change`

Process a request that falls outside the agreed scope.

```text
/scope-change "Meridian Health" --request "add a second data source"
```

**When to reach for it:** the first time the request appears, not the third.
**Returns:** either a priced change with its impact on dates and milestones, or
a refusal that says why and what the alternative is. A scope change is priced or
refused, never absorbed silently.

### `/client-risk`

Build an escalation plan when a date, the health or the trust is at risk.

```text
/client-risk "Meridian Health" --risk "sponsor left, no replacement named"
```

**When to reach for it:** as soon as the risk is real, while options still exist.
**Returns:** the escalation path, the specific ask, who is escalated to, and the
point at which the engagement is genuinely in trouble. Escalating to a customer
executive requires an explicit human decision.

### `/adoption`

Intervene when delivery landed and usage did not follow.

```text
/adoption "Meridian Health"
```

**When to reach for it:** when acceptance is recorded and the usage signals are
flat.
**Returns:** an intervention with a measurable target and a review date. If the
cause of flat adoption is unknown, it says so instead of aiming an intervention
at a guess.

## Value, renewal and ending

### `/value-proof`

Compile the evidence of realised value, and be honest about its limits.

```text
/value-proof "Meridian Health" --since onboarding
```

**When to reach for it:** before any review, and long before the renewal.
**Returns:** attributed outcomes, contributions stated as contributions rather
than causes, and an explicit list of what cannot be attributed. Overclaiming
here is the most expensive short-term win available.

### `/qbr`

Prepare the business review.

```text
/qbr "Meridian Health" --quarter 2026-Q3
```

**When to reach for it:** on the governance cadence agreed in `/client-plan`.
**Returns:** a review built on the success plan measures and the value proof,
which the customer could read line by line without finding a claim they would
dispute.

### `/renew-client`

Prepare the renewal recommendation and its evidence.

```text
/renew-client "Meridian Health"
```

**When to reach for it:** with enough lead time for the answer to be no.
**Returns:** a recommendation (renew, expand, renegotiate or let go) with the
health, adoption and value signals behind it, sent to Revenue {OS}. Delivery
owns the signals, Revenue owns the decision. A recommendation contradicted by
its own signals is not sent.

### `/case-study`

Ask for the story, properly.

```text
/case-study "Meridian Health"
```

**When to reach for it:** after a proven outcome, at earned timing, never during
a renewal negotiation.
**Returns:** the consent request first, scoped per quote and per figure, then a
draft the customer sees before anyone else. Written consent is required, it is
revocable, and withdrawal retracts the material from Storyteller {OS} and
Content {OS} immediately.

### `/offboard`

Close an engagement without damage.

```text
/offboard "Meridian Health"
```

**When to reach for it:** on any ending, whether it is a win or not.
**Returns:** a handover pack, data control returned to the customer, open items
resolved or named, and the delivery learning captured for Offer {OS} and
Growth {OS}.

---

## Command summary

| Command | Does |
|---|---|
| `/delivery` | opens the delivery portfolio |
| `/handoff-client` | runs the sales to delivery transfer |
| `/onboard-client` | creates the onboarding plan |
| `/success-plan` | defines outcomes and measures with the customer |
| `/client-plan` | creates milestones and governance |
| `/client-update` | drafts a transparent status update |
| `/scope-change` | prices or refuses a change request |
| `/client-risk` | creates an escalation plan |
| `/adoption` | builds an adoption intervention |
| `/value-proof` | compiles outcome evidence, honestly attributed |
| `/qbr` | prepares a business review |
| `/renew-client` | prepares the renewal recommendation |
| `/case-study` | obtains consent and builds a case study |
| `/offboard` | closes an engagement responsibly |

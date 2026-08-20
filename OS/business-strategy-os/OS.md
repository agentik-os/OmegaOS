# Business Strategy {OS}: Operating Specification

## 1. Purpose

Determine whether the operator owns a business or a job, measure the distance
between the two, and close it.

A job pays its holder and ends when they stop. An asset produces value without
them, can be handed to someone else, and can be sold. Most operators believe
they own the second and actually own the first. The difference is not a matter
of revenue: a business earning well can be entirely unsellable because every
decision, relationship and piece of judgement lives in one head.

This OS holds the asset thesis, the owner-dependence score, the value drivers
and their measured state, and the strategic options that follow from all three.

## 2. Boundary

- **Owns:** the asset thesis (why a rational third party would pay for this
  business rather than hire its owner), the owner-dependence assessment (which
  decisions, relationships, credentials and knowledge exist only in the owner's
  head), the durable advantage register (what remains true after a competitor
  copies the surface), the enterprise value drivers and their measured state
  (customer concentration, revenue retention and recurrence, margin quality,
  transferability of relationships, documented process coverage, key person
  risk, contract assignability, data and IP position), the strategic option set
  with the cost and reversibility of each, and the review cadence that keeps all
  of it current.
- **Does not own:**
  - personal cash flow, what comes in and what goes out of the operator's own
    accounts: Money {OS}
  - personal net worth, reserves and long-horizon personal goals: Wealth {OS}
  - which legal entity holds what, and on what terms: Ownership {OS}
  - the inventory of intellectual property and durable assets, and their
    protection and licensing: IP & Asset {OS}
  - the sale process, its timing and its execution: Exit & Liquidity {OS}
  - business cash flow, receivables and business reserves: Revenue {OS}
  - positioning and messaging: Positioning {OS}
  - writing the process documentation this OS finds missing: Process & SOP {OS}
  - the operating plan and its execution: Execution {OS} and Operations {OS}
  - allocation across several ventures: Strategy & Portfolio {OS} and
    Capital {OS}
- **Does not replace a professional.** A valuation range this OS produces is an
  internal working range computed from the operator's own numbers. It is not an
  audited valuation, not an accountant's opinion of value, and not usable in a
  filing, a loan application or a negotiation as an independent figure. Where a
  finding has a tax or legal consequence, this OS states the question and names
  the professional to ask. It does not answer it.
- **Hands off to:** Exit & Liquidity {OS} (measured value drivers and the
  readiness gap), Process & SOP {OS} (the documentation gaps owner dependence
  exposes), Team & Delegation {OS} (the roles the owner must stop occupying),
  Execution {OS} (dated remediation work), Ownership {OS} (structural changes
  the thesis implies, as a proposal only).
- **Consumes from:** Context & Memory {OS} (what is already established), KPI &
  Analytics {OS} (`kpi.metric.verified`), Revenue {OS} (customer concentration
  and retention facts), IP & Asset {OS} (`ipasset.registered`), Ownership {OS}
  (`ownership.entity.registered`).

*The rule that keeps this honest: this OS measures the business, it does not run
it. Every finding leaves here as a fact or a proposal, and the work that follows
is owned by the OS that already owns that work.*

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `THESIS` | no asset thesis exists, or the business has materially changed | the asset thesis: what a buyer would be buying, and why not just the owner | the thesis names a transferable source of value that is not the owner |
| `DEPENDENCE` | thesis exists, dependence never scored or older than the cadence | the owner-dependence assessment, per decision class | every decision class is assigned to a person, a document or a system, with the owner named where that is the answer |
| `DRIVERS` | dependence scored | the value driver table with each driver measured or explicitly unverified | every driver has a value, a source, and a verified or unverified label |
| `ADVANTAGE` | drivers measured | the durable advantage register | each claimed advantage survives the copy test or is struck |
| `OPTIONS` | thesis, dependence and drivers current | the strategic option set with cost, reversibility and the driver each moves | every option names which driver it moves and what it costs |
| `READINESS` | drivers measured within the cadence | the readiness gap against a stated standard | the gap is stated per driver, with unverified inputs listed separately |
| `REVIEW` | the cadence interval has elapsed | the delta since the last review | every driver is either re-measured or explicitly carried forward with its age |

`THESIS` is where a new user starts. An operator who cannot state what a buyer
would be buying has no basis for scoring anything else, and the honest first
output is often that the answer is "the owner", which is the finding.

## 4. Inputs

- The business as it currently operates: what it sells, to whom, and who does
  the work.
- Verified metrics from KPI & Analytics {OS}: revenue by customer, retention,
  gross margin, and their measurement dates.
- Customer concentration and retention facts from Revenue {OS}.
- The entity structure from Ownership {OS}: which entity holds the contracts,
  the IP and the operating accounts.
- The registered IP position from IP & Asset {OS}.
- The owner's own account of what only they can do. This is an input, not a
  measurement, and is labelled as such wherever it is used.
- The standard the readiness gap is measured against: the operator states it, or
  the OS asks. There is no universal standard, and the OS does not invent one.

## 5. Outputs

| Output | Lives in | Consumed by |
|---|---|---|
| the asset thesis | Context & Memory {OS} | Exit & Liquidity {OS}, Ownership {OS} |
| the owner-dependence assessment | Context & Memory {OS} | Team & Delegation {OS}, Process & SOP {OS} |
| the value driver table | Context & Memory {OS} | Exit & Liquidity {OS} |
| the durable advantage register | Context & Memory {OS} | Positioning {OS} as input, never as instruction |
| the strategic option set | Context & Memory {OS} | the operator |
| the readiness gap | Context & Memory {OS} | Exit & Liquidity {OS} |
| dated remediation work | Execution {OS} | Execution {OS} |

Each output carries the date of its inputs. A driver measured eight months ago
is reported with that date, never silently as current.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the asset thesis and its revision history | Context & Memory {OS} |
| canonical | the owner-dependence assessment | Context & Memory {OS} |
| canonical | the durable advantage register | Context & Memory {OS} |
| canonical | the operator's stated readiness standard | Context & Memory {OS} |
| projection | verified metrics behind the value drivers | KPI & Analytics {OS} and Revenue {OS}, read at measurement time and stamped |
| projection | entity structure and IP position | Ownership {OS} and IP & Asset {OS} |
| cache | the computed readiness gap | recomputed from drivers, never trusted across a driver change |
| temporary | the current session's working option set before it is accepted | the session |

The value driver table is the seam of this OS: the driver definitions and their
verified or unverified labels are canonical here, while the numbers underneath
them are projections of another OS's truth. Confusing the two is how a business
comes to believe it has measured something it has only asserted.

## 7. Rules and invariants

1. **The thesis names something other than the owner, or it fails.** An asset
   thesis whose answer to "what is being bought" is the owner's judgement,
   relationships or reputation is not a thesis. It is a diagnosis, and it is
   reported as one. The OS does not dress it up.
2. **A driver scored from opinion is labelled as opinion.** Every value driver
   carries its source and one of two labels: verified, meaning it traces to a
   metric from KPI & Analytics {OS} or Revenue {OS} with a measurement date, or
   unverified, meaning it rests on the operator's own account. Both are useful.
   Only one is evidence.
3. **No headline readiness score without naming the unverified inputs.** The OS
   will compute a readiness gap over a mixed set of verified and unverified
   drivers, but every presentation of that gap lists which drivers were
   unverified and what the gap would be with them excluded. A single number that
   hides its own foundations is worse than no number, because it gets quoted.
4. **The copy test governs the advantage register.** A claimed advantage stays
   in the register only if it would still hold after a competent competitor
   copied the visible surface: the product, the pricing, the site, the pitch. If
   copying the surface neutralises it, it was a feature, not an advantage, and
   it is struck with the reason recorded.
5. **Every option names the driver it moves and what it costs.** A strategic
   option with no named driver is a preference. Cost includes reversibility:
   whether the operator can undo it, and at what price.
6. **Findings leave as facts or proposals, never as instructions.** A
   restructuring implication goes to Ownership {OS} as a proposal. A missing
   process goes to Process & SOP {OS} as a gap. This OS never writes the
   documentation, changes the entity, or reassigns the work.
7. **Measurement age is part of the measurement.** Any driver older than the
   review cadence is reported with its age attached and excluded from a
   readiness claim unless the operator explicitly carries it forward.
8. **The valuation range is internal.** Any figure this OS produces is stamped
   as an internal working range from the operator's own numbers, with its method
   and inputs shown. It is never presented as an audited valuation, an
   accountant's opinion of value, or a figure fit for a filing or a counterparty.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no verified metric for a driver | score it from the operator's account, label it unverified, name the metric that would verify it |
| readiness score requested over mostly unverified drivers | produce it, list every unverified input, and show the gap recomputed with them excluded |
| KPI & Analytics {OS} and Revenue {OS} disagree on a figure | report both with their sources and dates, refuse to pick, route the reconciliation to the OS that owns the metric |
| the operator asserts an advantage the copy test kills | state the test and the result, keep the claim only as a labelled assertion, never in the register |
| no readiness standard stated | ask for it, offer the standards commonly used and their differences, do not default to one silently |
| driver data older than the cadence | report the age, mark the readiness gap stale, offer a re-measure |
| a finding has a tax or legal consequence | state the question, name the kind of professional who answers it, stop there |
| the operator asks for a defensible valuation | produce the internal working range with its method exposed, state plainly that it is not an audited valuation and cannot substitute for one |
| the thesis contradicts the entity structure | report the contradiction to the operator and to Ownership {OS} as a proposal, change nothing |

## 9. Human approval boundary

This OS assists the operator. It does not replace a legally accountable
accountant, tax professional or lawyer, and its output is not their work
product. Specifically:

- It **never** moves money, signs a document, restructures an entity, changes
  a contract, or executes a transaction. It has no such capability, and any
  such action is the operator's, taken with their professional advisers.
- A **valuation range** is internal, computed from the operator's own numbers
  with its method shown. It is not an audited valuation and not an accountant's
  opinion of value. It is not usable in a filing, a loan application, a
  shareholder dispute or a negotiation as an independent figure.
- A **tax or legal implication** the OS flags is a question to take to a
  professional, never an answer. The OS states what changed and why it might
  matter, and stops before the advice.
- It asks before: publishing or revising the asset thesis, recording a
  readiness gap that will be handed to Exit & Liquidity {OS}, sending any
  finding to another OS, sharing any output outside the operator's own systems,
  and carrying a stale driver forward into a current claim.

## 10. Completion criteria

The operator can state, from evidence rather than belief, what a buyer would be
buying, which parts of that still live only in their head, which value drivers
are measured and which are asserted, what the gap is to their stated standard,
and which option closes the largest part of it at what cost. When the answer is
that they own a job, they know that too, and they know precisely which
dependencies make it one.

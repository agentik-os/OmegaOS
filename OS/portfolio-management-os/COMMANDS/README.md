# Portfolio Management {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install portfolio-management-os` | Installs this OS into your environment | Once, first |
| `agentik configure portfolio-management-os` | Collects the minimum context it needs | After install |
| `agentik run portfolio-management-os` | Starts the OS | Every session |
| `agentik doctor portfolio-management-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update portfolio-management-os` | Updates to the latest version | When a release lands |
| `agentik eval portfolio-management-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | What it returns |
|---|---|---|---|
| `/portfolio open <position>` | Opens a position record: reporting expectations, data rights, contacts, baseline | The moment a commitment funds | The position record and the first reporting due date with a named person owing it |
| `/portfolio collect [period]` | Chases, receives and normalises periodic reporting across the portfolio | Every reporting period | Per position: reported, chased, or escalated as non-reporting |
| `/portfolio mark <position>` | Sets a valuation mark with method, evidence and date | When reporting lands or a marking event occurs | The mark record, and a refusal if no method was supplied |
| `/portfolio support <position>` | Logs a support request against the capacity budget and records its outcome | Whenever a position asks for help, before the help is given | The logged request with owner, capacity cost, and the remaining capacity |
| `/portfolio capacity` | Shows the support capacity budget and what has consumed it | When support feels uneven, and at period close | Capacity stated, capacity spent per position, capacity remaining |
| `/portfolio triage` | Classifies every position compounding, watch or impaired with the evidence | After the period's reporting is collected and marked | The triage table, with the date of every class change |
| `/portfolio impair <position>` | Runs an impairment assessment on a triggering event | Down round, covenant breach, key person loss, runway below floor | The assessment, and on approval an impairment record |
| `/portfolio followon <position>` | Produces a follow-on or stand down recommendation with the thesis checkpoint attached | When a position raises, or a stand down decision is due | A recommendation with evidence and no amount, addressed to Capital {OS} |
| `/portfolio report <period>` | Builds the owner or stakeholder portfolio report | At period close | The report with realised and unrealised separated in every view, unsent |
| `/portfolio exit-ready <position>` | Marks a position ready for an exit process | When the owner decides to pursue liquidity | The handover pack for Exit & Liquidity {OS}: current mark, method, applicable rights |
| `/portfolio close <position>` | Closes the record at exit or write off | When the position ends | Realised outcome, last mark before it, and the gap between them |

---

### `/portfolio open <position>`

Runs `ONBOARD`. It will not complete until three things exist: what reporting is
owed, in what form, and who personally owes it. This is the command whose
absence causes every chase problem later.

```bash
/portfolio open northwind --from capital.allocation.approved
/portfolio open northwind --reporting "monthly KPIs, quarterly accounts" \
  --rights "information rights per SHA clause 8" --contact "CFO"
```

**Returns** the position record and the first due date. Emits
`portfolio.position.opened`.

### `/portfolio collect [period]`

The chase list is the output, not a side effect.

```bash
/portfolio collect 2026-Q2
/portfolio collect 2026-Q2 --escalate
```

A position missing one period is chased and its mark labelled unsupported. A
position missing two consecutive periods is escalated to the owner and moves to
watch at minimum. Raw submissions are retained; the normalised series is derived
and can always be recomputed from them.

### `/portfolio mark <position>`

A mark is a method plus evidence plus a date. Supplying only a number is
rejected.

```bash
/portfolio mark northwind --method last-priced-round \
  --evidence "series A, 2026-05-12, term sheet on file" --value 4200000
/portfolio mark northwind --method cost \
  --evidence "no marking event since funding"
```

**Human approval gate.** The mark is not written to the book until the owner
approves it. `portfolio.mark.updated` is emitted on approval, never before. A
mark is never changed because the period reads badly: where a mark moves inside
a period, the prior mark, the new mark and the triggering evidence are shown
side by side.

### `/portfolio support <position>`

Support capacity is finite and is stated as a number before the period starts.
An unlogged favour is not portfolio support.

```bash
/portfolio capacity --set "20 hours per month, 4 warm intros per quarter"
/portfolio support northwind --request "intro to a VP Sales candidate" \
  --cost "2 hours, 1 intro"
```

**Human approval gate.** Any support with an external cost, in particular an
introduction that puts the owner's name behind a position, requires explicit
approval before delivery.

### `/portfolio followon <position>`

Refuses to run without the thesis checkpoint result from Investment Thesis {OS},
and never states an amount.

```bash
/portfolio followon northwind --checkpoint thesis.checkpoint.due:2026-Q2
```

**Returns** a recommendation (follow on, or stand down) with the checkpoint
result attached even when it is unfavourable, the current mark and its method,
and the reserve already held against the position. It emits
`portfolio.followon.recommended` and hands the pack to Capital {OS}. The amount
is Capital's decision, not this OS's.

### `/portfolio report <period>`

Builds the report. It does not send it.

```bash
/portfolio report 2026-Q2
/portfolio report 2026-Q2 --audience limited-partners
```

**Human approval gate.** Sending is a separate, explicit human act every time.
Stakeholder reporting may be a regulated communication depending on the
jurisdiction and the recipient class, and every mark in the report is a
management estimate rather than an audited figure, stated as such on the face of
the report. `portfolio.report.published` is emitted only after the send is
approved.

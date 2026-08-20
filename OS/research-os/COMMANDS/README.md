# Research {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install research-os` | Installs this OS into your environment | Once, first |
| `agentik configure research-os` | Collects the minimum context it needs | After install |
| `agentik run research-os` | Starts the OS | Every session |
| `agentik doctor research-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update research-os` | Updates to the latest version | When a release lands |
| `agentik eval research-os` | Runs its evaluation suite | Before trusting it |

## OS commands

### `/research <topic or question>`

The root command. Given a topic, it runs `FRAME` and hands back the question in
answerable form. Given an already-framed question, it runs the corpus check and
proposes a search plan.

**When to use it:** any time you need to know something and do not already own
the answer.
**Returns:** either two or three answerable versions of your question with their
different scopes, or the corpus-check result plus a draft search plan.

### `/question <topic>`

Frame only. Turns a topic into a question with a scope, a time window, a
population, the decision it feeds and an explicit out-of-scope list.

**When to use it:** when the ask is broad ("look into agentic frameworks") and
nobody has said what would count as an answer.
**Returns:** the framed question, its sub-questions, the decision context, and
what is deliberately excluded. Emits `research.question.framed`.

### `/plan [--budget <time|money>] [--depth quick|standard|deep]`

Build the search plan: sub-questions, the source types that could answer each,
where to look, the stopping rule, and the budget. Nothing is retrieved.

**When to use it:** before gathering, always. A plan is what stops research from
expanding to fill whatever budget it is given.
**Returns:** the plan, the stopping rule, the cost estimate, and the list of
steps that will need approval (paid sources, restricted access, contacting a
human, personal data).

### `/gather [--sub-question <id>] [--source-type <type>]`

Execute the plan and open the source ledger. Every retrieval attempt is logged
with its locator, the date accessed and the access outcome, including every
paywall, block, dead link and rate limit.

**When to use it:** once the plan is agreed and any required approvals are in.
**Returns:** the source ledger, live, with what was retrieved and what was
blocked and why. A block is reported as a negative result, never as silence.

### `/vet [<source-id> | --all]`

Vet sources on tier, authority, method, recency, funding interest and whether
the primary behind them is reachable. Retrieves the primary where a source is
summarising one.

**When to use it:** after gathering, before extracting anything. Skipping this
is how a vendor benchmark ends up quoted as an independent measurement.
**Returns:** per source, its tier, who produced it, how they know, when, who paid
for it, and its stated weakness. Sources that fail vetting stay in the ledger,
marked, so nobody rediscovers them next month.

### `/verify <claim-id | --all>`

Reopen the citation behind a claim and confirm the source actually says it.

**When to use it:** on every load-bearing claim before the memo goes out, and on
any claim you inherited from elsewhere.
**Returns:** per claim, verified with the locator, or failed with the reason. A
claim whose citation cannot be reopened is removed from the answer and dropped to
`unknown`. This command is the direct defence against a fabricated citation.

### `/contradictions`

Show every place the sources disagree, with both sides, their evidence, and a
characterisation of the disagreement: different populations, different dates,
different method, or different interest.

**When to use it:** before writing the memo, and any time an answer feels
suspiciously clean.
**Returns:** the contradiction record, each entry classed `contested`, plus what
evidence would settle it. Emits `research.claim.contested` for any contested
claim a decision rests on. Nothing here is averaged into a middle number.

### `/confidence [<claim-id>]`

State confidence for one claim or for the answer as a whole, with the reason
behind the level.

**When to use it:** before anyone acts on the memo, and whenever someone asks
"how sure are we".
**Returns:** high, medium or low, plus the checkable reasons: number of
independent sources, source tier, recency, method quality, size of the remaining
unknown, and the cheapest action that would raise the level.

### `/memo [--audience <who>] [--length short|full]`

Write the research memo: answer first, then the evidence, the contradictions,
the unknown register, the confidence with its reasons, and the full source
ledger including what was blocked.

**When to use it:** when the stopping rule has fired, or the budget is spent and
you need what is established so far.
**Returns:** the memo, in which every claim carries a source, a date and a class,
and every citation can be reopened by the reader without this OS. Emits
`research.evidence.compiled` and, once filed, `research.memo.published`.

### `/research-audit <artifact>`

Audit someone else's research: a memo, a deck, a report, a wiki page, a model
whose inputs came from somewhere.

**When to use it:** when you inherit confident claims and do not know where they
came from, or when a number has been circulating and nobody can name its origin.
**Returns:** claim by claim, classified as sourced (with the locator), unsourced,
circular (several citations tracing to one origin), stale (true on a date that
has passed), or unreopenable. It states what survives the audit and what the
artifact should stop asserting.

## Command summary

| Command | Does | Returns |
|---|---|---|
| `/research` | entry point: topic to question, or question to plan | answerable question, or corpus check plus draft plan |
| `/question` | frame a topic into an answerable question | scope, sub-questions, decision context, out-of-scope list |
| `/plan` | design the search before touching a source | sub-questions, source types, stopping rule, budget, approvals needed |
| `/gather` | execute the plan, log every attempt | source ledger, including everything blocked and why |
| `/vet` | judge each source before trusting it | tier, authority, method, recency, interest, stated weakness |
| `/verify` | reopen a citation and confirm it says this | verified with locator, or removed and dropped to unknown |
| `/contradictions` | show where the sources disagree | both sides, their evidence, what would settle it |
| `/confidence` | how sure, and why | level plus checkable reasons and the cheapest way to raise it |
| `/memo` | write the defensible answer | memo with sourced, dated, classed claims and the full ledger |
| `/research-audit` | check research you inherited | per claim: sourced, unsourced, circular, stale, unreopenable |

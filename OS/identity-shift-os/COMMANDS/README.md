# Identity Shift {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install identity-shift-os` | Installs this OS into your environment | Once, first |
| `agentik configure identity-shift-os` | Collects the minimum context it needs | After install |
| `agentik run identity-shift-os` | Starts the OS | Every session |
| `agentik doctor identity-shift-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update identity-shift-os` | Updates to the latest version | When a release lands |
| `agentik eval identity-shift-os` | Runs its evaluation suite | Before trusting it |

## OS commands

The five commands follow the life of one shift, in order: scope it, charter it,
feed it evidence, review it, close it.

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/identity-shift` | Opens the OS and routes to the right mode for where the shift currently stands | Any request about becoming someone you are not yet | The mode it selected and that mode's artifact, or a routing verdict sending you to another OS |
| `/shift-define` | Charters one shift: from, to, entry baseline, exit test, close-by date | Once per shift, at the start | The charter, with a shift id, or a refusal naming the missing field |
| `/shift-evidence` | Records one dated observation, classed confirming, disconfirming or ambiguous | Every time something happens that bears on the target identity | The ledger entry as written, and the running evidence balance |
| `/shift-review` | Runs the becoming review for the period | On the review cadence, or when you want to know where it stands | Evidence balance, drift check, one adjustment, and the continue, amend or close verdict |
| `/shift-close` | Closes the shift and hands the identity model to Mindset {OS} | When the exit test is met, the close-by date arrives, or you abandon it | The closing record with its verdict, and confirmation that Mindset {OS} took the model |

### `/identity-shift`

The entry point. It looks at whether a shift is open, and routes accordingly: no
shift and a vague request goes to `SCOPE`, no shift and a clear from and to goes
to `CHARTER`, an open shift goes to `EVIDENCE` or `REVIEW` depending on what you
brought it, and a shift past its close-by date goes to `CLOSE`.

The most useful thing it does is refuse. If what you described is a value
conflict, a goal, a hard call or a habit, it names the owning OS and sends you
there instead of opening a shift that will never close.

```
/identity-shift I am leaving my job in four months and I am not a founder yet
/identity-shift where does my shift stand
```

### `/shift-define`

Charters the shift. It reads the current identity model from Mindset {OS} and
quotes the starting identity into the charter, checks the target identity
against the value set from Alignment {OS}, then asks for the exit test and the
close-by date before anything else.

It refuses to charter when the starting identity is unavailable, when the exit
test cannot be checked by someone other than you, or when there is no close-by
date. Those refusals are the point: they are what stops a shift from becoming a
permanent second identity model competing with Mindset {OS}.

```
/shift-define from "senior engineer who executes well" to "founder who sells"
```

**When to use it:** once per shift.
**Returns:** the charter with a shift id, the entry baseline dated today, the
exit test, the review cadence, the close-by date, and the one behaviour that
carries the shift (handed to Habit Tracker {OS} as a contract tagged with the
shift id).

### `/shift-evidence`

Adds one entry to the evidence ledger. An entry is an observable action with a
date. It is classed confirming, disconfirming or ambiguous, and disconfirming
entries are recorded with exactly the same weight as confirming ones.

It rejects feelings and intentions. "I felt more like a founder this week" is
not an entry; "I ran the pricing call myself on the 12th and did not discount"
is.

```
/shift-evidence 12 Aug, ran the pricing call myself, held the price
/shift-evidence 14 Aug, handed the second call back to my co-founder
```

**When to use it:** the same day something happens, while the detail is still
accurate.
**Returns:** the entry as recorded, its class, and the running balance since the
last review.

### `/shift-review`

Runs the becoming review for the period. It computes the evidence balance since
the last review, checks the shift for drift against the charter, and makes
exactly one adjustment. It ends with an explicit verdict: continue, amend the
charter with a recorded reason, or move to close.

A period with no evidence in either direction is reported as empty, not as
progress. Two consecutive empty periods force a close decision.

```
/shift-review
```

**When to use it:** on the cadence set in the charter, weekly by default.
**Returns:** the dated review record, the evidence balance, the drift finding,
the one adjustment, and the verdict.

### `/shift-close`

Closes the shift with one of three verdicts: achieved (the exit test was met),
expired (the close-by date arrived and it was not), or abandoned (you stopped).
All three are legitimate closes and all three produce a record.

It writes what the evidence actually showed, converts the result into identity
statements written as behaviours under conditions, hands those to Mindset {OS},
closes every behaviour contract tagged with the shift id in Habit Tracker {OS},
and archives the charter. After it runs, this OS holds nothing about you.

```
/shift-close achieved
/shift-close abandoned
```

**When to use it:** the moment the exit test is met, or the close-by date
arrives, or you decide to stop. Not later.
**Returns:** the closing record, the identity statements handed over, and
confirmation that Mindset {OS} adopted them or stated why it declined.

## Command summary

| Command | Does |
|---|---|
| `/identity-shift` | routes to the right mode, or to the right OS |
| `/shift-define` | charters one shift: from, to, exit test, close-by date |
| `/shift-evidence` | one dated, classed observation into the ledger |
| `/shift-review` | evidence balance, drift, one adjustment, a verdict |
| `/shift-close` | closes it and hands the identity model to Mindset {OS} |

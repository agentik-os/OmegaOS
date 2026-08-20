# IP & Asset {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

None of these commands file, sign, send, pay or receive anything. The ones that
touch a legal or financial consequence stop and ask.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install ip-asset-os` | Installs this OS into your environment | Once, first |
| `agentik configure ip-asset-os` | Collects the minimum context it needs | After install |
| `agentik run ip-asset-os` | Starts the OS | Every session |
| `agentik doctor ip-asset-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update ip-asset-os` | Updates to the latest version | When a release lands |
| `agentik eval ip-asset-os` | Runs its evaluation suite | Before trusting it |

## OS commands

### `/ip register`

Add an asset to the register, or update one that is already there. Asks for the
type, the description, the creation facts, the holder of record and the
jurisdictions.

```bash
/ip register
/ip register --type trademark --name "Agentik"
/ip register --type dataset --path ./data/corpus
```

**What it does:** creates or updates one row in the asset register, attaches the
holder from the entities Ownership {OS} has confirmed, and immediately runs the
title check on the new asset.

**When to use it:** the moment an asset exists, not the moment somebody asks for
it. Registration after the fact is how contractor gaps survive for years.

**Returns:** the asset row, its title status (`proven`, `unproven` or
`disputed`), and the named next gap if there is one. Emits `ipasset.registered`.

### `/ip inventory`

Sweep for assets that exist but are not in the register.

```bash
/ip inventory
/ip inventory --scope brand
/ip inventory --scope code
```

**What it does:** walks the categories (marks, works, patents, secrets, domains,
brand assets, content, code, data, models, physical) and asks about each, using
what Brand {OS} and Context & Memory {OS} already know so it does not ask twice.

**When to use it:** first session, and once a quarter afterwards.

**Returns:** the assets found, the assets added, and a list of what it could not
resolve without you.

### `/ip title <asset>`

Establish or re-establish chain of title.

```bash
/ip title agentik-wordmark
/ip title --all --unproven-only
```

**What it does:** asks who made it, when, under what agreement, and looks for the
document that proves the answer. Flags contractors without assignment clauses,
employees outside invention terms, co-authors and commissioned work.

**When to use it:** before any grant, sale, funding round or diligence request,
and whenever a new contributor touches an asset.

**Returns:** the title status per asset, the document that proves it, or the
named missing document and who holds it. Never upgrades a status on your say-so.
Emits `ipasset.title.assigned` when a holder is confirmed and approved.

### `/ip protect <asset>`

Decide the protection posture for one asset.

```bash
/ip protect agentik-wordmark
/ip protect agentik-wordmark --jurisdictions FR,EU,US
```

**What it does:** lays out the options (register, hold as trade secret, accept
unregistered), what each buys and costs, per jurisdiction, and records the
decision with its reason.

**When to use it:** when an asset is worth more than the filing and attorney
cost of protecting it, or when an unproven asset has just become proven.

**Returns:** the decision per jurisdiction, and where registration is chosen, the
counsel brief to instruct. It does not file. Clearance and freedom-to-operate
are opinions a qualified attorney gives, and this command declines to give them.

### `/ip license`

Record a grant out or a grant in.

```bash
/ip license --out --to "Acme" --asset corpus-v2
/ip license --in --from "Vendor" --asset font-family
/ip license list
```

**What it does:** records counterparty, direction, exclusivity, territory, term,
field of use, royalty basis and revocation trigger, from the executed document.
Checks every existing grant on the same asset before recording an exclusive, and
refuses to record two exclusives that overlap.

**When to use it:** when terms are agreed, and again when the signed document
comes back.

**Returns:** the licence record, its status (`executed` or `unexecuted`), and any
conflict it found. A grant is committed only after Review & Governance {OS}
returns `change.approved`. Emits `ipasset.license.granted`.

### `/ip calendar`

Show and maintain the renewal and deadline calendar.

```bash
/ip calendar
/ip calendar --horizon 12m
/ip calendar --push
```

**What it does:** lists every dated obligation with its lead time and named human
owner, and with `--push` emits them to Execution {OS} as tasks.

**When to use it:** monthly, and before any period you will be unreachable.

**Returns:** the dated obligations, sorted, with unknown dates surfaced first and
treated as urgent. A missed renewal can extinguish a right permanently, so this
command states which obligations belong on a professional docket as well as
here. Emits `ipasset.renewal.due`.

### `/ip watch <item>`

Triage a suspected infringement.

```bash
/ip watch "https://example.com/copy-of-my-page"
/ip watch --list
```

**What it does:** records the evidence, matches it against the register, and
classifies the item as ignore, monitor, or escalate to counsel.

**When to use it:** when you find a copy, or someone reports one.

**Returns:** the classification, the reasoning, and where the answer is escalate,
the counsel brief. It does not draft or send a demand letter.

### `/ip value <asset>`

Record a valuation estimate.

```bash
/ip value corpus-v2 --method cost-to-recreate
/ip value agentik-wordmark --method relief-from-royalty
```

**What it does:** records the method, the inputs, the date and the figure.

**When to use it:** when Wealth {OS} needs a balance sheet number, or Exit &
Liquidity {OS} needs a starting position.

**Returns:** the valuation record, always labelled an estimate and never an
appraisal. A figure that will be relied on in a transaction or a tax position
needs a qualified valuer, and the record says so. Emits
`ipasset.valuation.recorded`.

### `/ip schedule`

Produce the IP schedule a buyer will diligence.

```bash
/ip schedule
/ip schedule --for exit
```

**What it does:** renders the register in diligence shape: every asset with title
status, protection posture, jurisdictions, encumbrances, licences out and open
source obligations.

**When to use it:** before any diligence, funding round or sale conversation.

**Returns:** the schedule, including every unproven asset, marked unproven. A
schedule that hides its gaps is the one that fails in diligence.

### `/ip brief <matter>`

Assemble a counsel brief.

```bash
/ip brief agentik-wordmark-eu-filing
```

**What it does:** packs the facts, the documents, the specific question and the
outcome the user wants, in the form a professional can act on.

**When to use it:** every time the answer is "ask a lawyer".

**Returns:** the brief, and a request for approval before anyone is instructed,
because instructing costs money.

## Command summary

| Command | Does |
|---|---|
| `/ip register` | add or update one asset in the register |
| `/ip inventory` | sweep for assets that are not in the register yet |
| `/ip title <asset>` | prove ownership, or name the missing document |
| `/ip protect <asset>` | decide register, trade secret, or unregistered |
| `/ip license` | record a grant out or in, and refuse overlapping exclusives |
| `/ip calendar` | dated obligations, lead times, owners, tasks |
| `/ip watch <item>` | triage a suspected infringement |
| `/ip value <asset>` | record a valuation estimate with method and date |
| `/ip schedule` | the IP schedule for diligence, gaps included |
| `/ip brief <matter>` | the pack an instructed professional works from |

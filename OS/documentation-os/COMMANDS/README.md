# Documentation {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install documentation-os` | Installs this OS into your environment | Once, first |
| `agentik configure documentation-os` | Collects the minimum context it needs | After install |
| `agentik run documentation-os` | Starts the OS | Every session |
| `agentik doctor documentation-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update documentation-os` | Updates to the latest version | When a release lands |
| `agentik eval documentation-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/doc-map` | Inventories the document set | the document store | what exists, where, who owns it, last verified, duplicates, orphans |
| `/doc-write` | Writes one document to the house shape | the question, the reader, the source | a document with answer first, owner, verified date and review date; refuses to publish without an owner |
| `/doc-find` | Answers a question from the set | the question | the answer plus the document and verified date it came from, or a plain statement that nothing answers it |
| `/doc-verify` | Checks a document against reality | the document, current facts | confirmed with a new verified date, corrected, or a drift report addressed to the owner |
| `/doc-merge` | Collapses duplicates | two or more documents answering one question | one surviving document plus redirects; requires both owners to agree |
| `/doc-retire` | Removes a document from circulation | the document, the reason | an archive entry, a removed entry point, and a pointer to what replaces it |
| `/doc-index` | Repairs findability | the document set | corrected titles, entry points, naming and search terms, with the questions that were failing |
| `/doc-stale` | Lists what has passed its review date | the index | stale documents by owner, oldest first, with the risk of each being wrong |

### When to reach for which

- Starting on an unfamiliar or neglected doc set: `/doc-map` first, always.
- Answering anyone: `/doc-find`, so the answer carries its source.
- Before writing anything new: `/doc-find`, then `/doc-write` only if nothing
  answers the question.
- On a cadence: `/doc-stale`, then `/doc-verify` on what it returns.
- When the same question keeps being asked: `/doc-index`, not another document.

## Command summary

| Command | Does |
|---|---|
| `/doc-map` | inventory with owners and verified dates |
| `/doc-write` | one document, answer first, four required fields |
| `/doc-find` | a cited answer, or an honest nothing |
| `/doc-verify` | confirm, correct, or report drift |
| `/doc-merge` | one question, one surviving document |
| `/doc-retire` | reversible archival with a redirect |
| `/doc-index` | fixes why something could not be found |
| `/doc-stale` | what is past its review date, by owner |

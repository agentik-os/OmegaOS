# Builder {OS}: Setup

<!-- agentik:scaffold -->

The Runtime asks for the minimum context needed to be useful now, not
everything this OS could ever use. You can deepen configuration later.

## Required

To be authored: the few inputs without which this OS cannot work.

## Optional

To be authored: what improves the output when supplied.

## Configure

```bash
agentik configure builder-os
```

## Verify

```bash
agentik doctor builder-os
```

Reports which required inputs are present, which adapters support this OS on
your current AI environment, and what falls back.

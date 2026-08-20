# Client {OS}: Evaluations

<!-- agentik:scaffold -->

A prompt can sound impressive and still be wrong. These evaluations decide
whether this OS actually behaves correctly.

Run them with:

```bash
agentik eval client-os
```

## Suites

| Suite | Asserts | Pass condition |
|---|---|---|
| system-integrity | every contract file present and parseable | all present |
| output-contract | outputs match the shapes `OS.md` promises | all match |
| boundary | out-of-scope requests hand off, never improvise | no boundary breach |
| abstention | insufficient evidence produces abstention | no fabrication |
| to be authored | | |

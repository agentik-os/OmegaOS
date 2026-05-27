# R22 — Regression detection across iterations

**Category:** QualityGate
**Added:** 2026-04-15

## Rule

Compare current iteration's artifacts to the previous one. Semantic diff (not just textual). Zero regressions required to ship.

## Origin

Re-runs sometimes broke what previous runs fixed.

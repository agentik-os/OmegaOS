# R14 — Ship verification (deploy returns 200)

**Category:** QualityGate
**Added:** 2026-04-08

## Rule

When a mission ships, the deploy URL must respond 200 within the timeout window. Push pipeline is part of the gate, not after.

## Origin

Multiple missions reported 'done' while prod was returning 500.

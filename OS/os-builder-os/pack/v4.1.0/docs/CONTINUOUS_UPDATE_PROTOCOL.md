# Continuous update protocol

## Why updates are part of the OS

An ultimate OS can become weak when its evidence, assumptions, tools, regulations or user context change.

Updateability is therefore a first-class capability.

## Update triggers

- new edition of a retained book;
- newly influential book or school;
- new systematic evidence;
- new primary evidence that challenges a material claim;
- official standard or regulatory change;
- recurring user failure pattern;
- new field postmortem;
- command or workflow regression;
- neighboring OS contract change;
- scheduled freshness review.

## Update workflow

```text
DISCOVER CHANGE
→ CLASSIFY IMPACT
→ UPDATE SOURCE LEDGER
→ DIFF CLAIMS
→ RE-SYNTHESIZE AFFECTED CLUSTERS
→ RECOMPILE RULES
→ MIGRATE WORKFLOWS AND STATE
→ RUN TARGETED AND REGRESSION EVALS
→ UPDATE DOCS
→ RELEASE NEW VERSION
```

## Required diffs

Every update produces:

- source diff;
- claim diff;
- confidence diff;
- principle and rule diff;
- command and workflow diff;
- schema and migration diff;
- eval diff;
- documentation diff.

## Versioning

- Patch: corrections that do not alter user-facing contracts.
- Minor: backward-compatible capabilities, commands or evidence updates.
- Major: changed core logic, incompatible schemas or command behavior.

## Deprecation

Deprecated claims and commands remain traceable with:

- reason;
- replacement;
- effective version;
- migration instructions;
- evidence that triggered the change.

# Knowledge synthesis protocol

## Purpose

Synthesis is the transformation from many source-specific models into one coherent, conditional and traceable operating model.

It is not concatenation.

## Step 1: Normalize concepts

Create canonical concept IDs and map synonyms, near-synonyms and overloaded terms.

Example:

```text
self-efficacy
confidence
agency belief
perceived capability
```

These may overlap without being identical. Preserve distinctions before merging.

## Step 2: Normalize claims

Convert each claim into one clear proposition with:

- subject;
- relationship or mechanism;
- outcome;
- population or context;
- conditions;
- time horizon;
- source support.

## Step 3: Cluster by mechanism

Group claims that describe the same underlying process, even when authors use different language.

## Step 4: Classify agreement

Use these synthesis statuses:

- **Robust:** supported across diverse credible sources and contexts.
- **Promising:** supported but limited by evidence, scope or replication.
- **Conditional:** useful only under explicit conditions.
- **Disputed:** credible sources disagree and the conflict is unresolved.
- **Outdated:** once useful but materially superseded.
- **Rejected:** contradicted, unsafe or unsupported beyond acceptable limits.
- **Design choice:** selected by Builder for usability or coherence, not claimed as empirical truth.

## Step 5: Resolve contradictions conditionally

Prefer rules such as:

```text
IF context A and constraints B
THEN use method X
ELSE IF context C
THEN use method Y
ELSE collect more information
```

Do not create false universal principles.

## Step 6: Compile mechanisms into principles

A principle must include:

- mechanism;
- expected benefit;
- conditions;
- limits;
- observable signal;
- operational implication.

## Step 7: Compile principles into decision rules

Every rule must have:

- trigger;
- required inputs;
- condition;
- action;
- expected output;
- confidence;
- exception;
- escalation;
- evidence links.

## Step 8: Compile rules into system behavior

Rules are organized into:

- diagnostics;
- prioritization;
- planning;
- execution;
- monitoring;
- review;
- adaptation;
- recovery;
- escalation.

## Step 9: Preserve epistemic labels

The OS interface should distinguish:

- established guidance;
- conditional heuristic;
- experimental practice;
- user preference;
- unresolved question.

## Step 10: Red-team the synthesis

Ask:

- Which popular idea survived only because it appeared often?
- Which academic idea is correct but unusable?
- Which practitioner idea works only for a narrow population?
- Which contradiction was hidden by vague wording?
- Which rule could cause harm when over-applied?
- Which key concept lacks a measurable signal?
- Which design choice is being presented as evidence?

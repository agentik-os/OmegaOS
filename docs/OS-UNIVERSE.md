# Agentik OS Universe: Master OS Map v1

> **Doctrine.** This is the canonical taxonomy of the Agentik OS universe: which
> operative systems exist, which stack each belongs to, and the LOGIC each one
> runs. It is the reference the Forge and AGK build against.
>
> | | |
> |---|---|
> | Status | Doctrine, v1 |
> | Adopted | 2026-08-18 |
> | Authority | Operator directive. This map supersedes any ad-hoc stack list. |
> | Scope | The whole universe (~12 stacks). The shipped registry is a subset, see Appendix A. |
> | Implementation SSOT | `OS/_tools/suite.py` (the 73-unit registry). Never hand-edit its generated files. |
> | Integration standard | `docs/OS-SUITE.md` (what an integrated OS must expose). |
> | Build program | `docs/OS-BUILD-STEPPER.md` (the method and order that turn this map into shipped units). |
> | Installed copy | `~/.omega/docs/OS-UNIVERSE.md` |

## The fundamental rule

**Each OS works alone. The connections between OS are optional and activated per
mission.**

A workflow can therefore use 1 OS, several OS in sequence, several OS in
parallel, or form a controlled loop. No OS ever REQUIRES another to be useful.
`/mindset` runs on its own. So does every other unit.

This is the whole architectural bet: the OS are the units of intelligence, the
capabilities are their shared language, the workflows combine them, the bundles
organize them, the Orchestrator picks the path, the Registry knows what exists,
and the user keeps control.

---

## 00. Core Agentik Stack

The systems that let every other OS function intelligently.

| OS | Logique principale |
|----|--------------------|
| Agentik Runtime {OS} | Exécuter et coordonner les OS |
| OS Builder / Forge {OS} | Rechercher → définir → architecturer → build → eval → package |
| Context {OS} | Retrieve → filter → rank → compress → inject → verify |
| Memory {OS} | Observe → qualify → store → retrieve → update → forget |
| Knowledge {OS} | Sources → extract → structure → connect → retrieve |
| Prompt {OS} | Intent → context → contract → prompt → eval → improve |
| Harness {OS} | Model + context + tools + skills + memory + policies + evals |
| Agent {OS} | Mission → role → capabilities → action → verification |
| Orchestration {OS} | Mission → capability graph → route → coordinate → synthesize |
| AI Logic {OS} | State → rules → reasoning → routing → action |
| Quality & Evaluation {OS} | Output → tests → graders → failure analysis → repair |
| Audit {OS} | System → adversarial inspection → weaknesses → remediation |
| Automation {OS} | Trigger → conditions → action → verification → event |
| Review & Governance {OS} | Observe system → review → govern → update |
| Documentation {OS} | System state → structured knowledge → documentation → update |

### Core loop

```
CONTEXT
   ↓
KNOWLEDGE
   ↓
PROMPT / SKILL / AGENT
   ↓
ORCHESTRATION
   ↓
EXECUTION
   ↓
EVALUATION
   ↓
MEMORY
   ↓
OBSERVABILITY
   ↓
GOVERNANCE
   ↓
UPDATE
```

---

## 01. Personal Evolution Stack

The personal transformation system.

1. Mindset {OS}
2. Identity Shift {OS}
3. Alignment {OS}
4. Goal & Life Strategy {OS}
5. Decision {OS}
6. Execution {OS}
7. Routine {OS}
8. Habit Tracker {OS}
9. Health & Energy {OS}
10. Journal {OS}
11. Intuitive {OS}
12. Mentor {OS}

### Grande logique

```
WHY
 ↓
MINDSET              How am I interpreting reality?
 ↓
ALIGNMENT            Is this what I actually want?
 ↓
IDENTITY             Who must I become?
 ↓
GOALS                What outcomes matter?
 ↓
DECISION             What should I choose?
 ↓
EXECUTION            What should I do?
 ↓
ROUTINE              How should my days operate?
 ↓
HABITS               What needs to become automatic?
 ↓
HEALTH & ENERGY      Can I sustain it?
 ↓
JOURNAL              What actually happened?
 ↓
EVIDENCE
 ↓
MINDSET UPDATE
```

### Boucle

```
Mindset → Identity → Action → Evidence → Journal → Reflection → Mindset
```

The loop is **never mandatory**. `/mindset` can run alone.

---

## 02. Social / Human Stack

Communication, relations, network and attraction.

* Social Intelligence {OS}
* Communication {OS}
* Conversation {OS}
* Attraction / Seductive {OS}
* Relationship {OS}
* Relationship & Network {OS}

### Logique

```
NOTICE
 ↓
READ CONTEXT
 ↓
OPEN
 ↓
LISTEN
 ↓
THREAD
 ↓
CONNECT
 ↓
CALIBRATE
 ↓
EXPRESS INTENT
 ↓
RECIPROCITY
 ↓
ESCALATE / DE-ESCALATE
 ↓
REFLECT
 ↓
LEARN
```

### Routing by situation

```
                    SOCIAL INTELLIGENCE
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
       COMMUNICATION    ATTRACTION    NETWORK
              │            │            │
              ▼            ▼            ▼
         CONVERSATION   ROMANTIC     RELATIONSHIP
                        LEGIBILITY
```

So there is no confusion between **being socially excellent** and **expressing a
romantic intention**.

---

## 03. Learn / Knowledge Stack

* Books {OS}
* Librarian {OS}
* Research {OS}
* Knowledge {OS}
* Interest Media {OS}
* Context & Memory {OS}

### Logique

```
QUESTION / CURIOSITY
 ↓
DISCOVER
 ↓
SOURCE
 ↓
READ / RESEARCH
 ↓
EXTRACT
 ↓
VERIFY
 ↓
SYNTHESIZE
 ↓
CONNECT
 ↓
STORE
 ↓
RETRIEVE
 ↓
APPLY
```

An interesting chain becomes:

```
Books + Research + Librarian
 ↓
Knowledge
 ↓
Context
 ↓
Any other OS
```

Knowledge therefore does not stay in a book summary. It becomes **capability
context**.

---

## 04. Discover & Strategy Stack

Before building anything.

* Research {OS}
* Market Research {OS}
* Trend & Opportunity {OS}
* Customer Discovery {OS}
* Brainstorm {OS}
* Validation {OS}
* Strategy & Portfolio {OS}
* Business Model {OS}
* Decision {OS}

### Logique

```
QUESTION
 ↓
RESEARCH
 ↓
MARKET
 ↓
CUSTOMER
 ↓
OPPORTUNITY
 ↓
IDEAS
 ↓
HYPOTHESES
 ↓
VALIDATION
 ↓
STRATEGY
 ↓
DECISION
 ↓
Blueprint {OS}
```

---

## 05. Build Stack

One of the most important chains.

* Research {OS}
* Blueprint {OS}
* Design {OS}
* Prototype {OS}
* Stepper {OS}
* Builder {OS}
* Quality & Evaluation {OS}
* Security {OS}
* Release {OS}
* Documentation {OS}

### Full build logic

```
IDEA
 ↓
RESEARCH
 ↓
BLUEPRINT
 ↓
DESIGN
 ↓
PROTOTYPE
 ↓
STEPPER
 ↓
BUILDER
 ↓
TEST
 ↓
QUALITY / EVAL
 ↓
SECURITY
 ↓
GAUNTLET
 ↓
RELEASE
 ↓
DOCUMENT
 ↓
OBSERVE
 ↓
ITERATE
```

But the orchestrator may decide:

```
Need research?               NO → skip
Need visual/product design?  NO → skip
Prototype useful?            NO → skip
```

So `Blueprint → Stepper → Builder → Eval` is perfectly valid.

---

## 06. Content / Media Stack

* AGK-Market {OS}
* Positioning {OS}
* Brand {OS}
* Storyteller {OS}
* Content {OS}
* Viral {OS}
* Interest Media {OS}

### Logic

```
AUDIENCE
 ↓
POSITIONING
 ↓
BRAND
 ↓
IDEA / KNOWLEDGE / EXPERIENCE
 ↓
STORY
 ↓
CONTENT
 ↓
FORMAT
 ↓
HOOK
 ↓
DISTRIBUTION
 ↓
ATTENTION
 ↓
DATA
 ↓
LEARNING
 ↓
NEXT CONTENT
```

The Knowledge Stack can feed Content directly:

```
Books · Research · Journal · Experiences · Knowledge
 ↓
Storyteller
 ↓
Content
 ↓
Viral
```

Very interesting for public building.

---

## 07. Commercial / Growth Stack

* Positioning {OS}
* Offer {OS}
* Pricing {OS}
* Sales {OS}
* Affiliate {OS}
* Growth {OS}
* Revenue {OS}
* Delivery & Customer Success {OS}
* Relationship & Network {OS}

### Logic

```
MARKET
 ↓
POSITIONING
 ↓
OFFER
 ↓
PRICING
 ↓
ACQUISITION
 ↓
SALES
 ↓
CUSTOMER
 ↓
DELIVERY
 ↓
SUCCESS
 ↓
RETENTION
 ↓
REFERRAL
 ↓
AFFILIATE
 ↓
REVENUE
 ↓
REINVEST
```

With feedback:

```
Customer Success → Customer Insight → Offer → Sales
```

---

## 08. Operator / Business Stack

* Operator {OS}
* Execution {OS}
* Project {OS}
* Client {OS}
* Meeting {OS}
* Documentation {OS}
* Operations & Automation {OS}
* Process & SOP {OS}
* Team & Delegation {OS}
* KPI & Analytics {OS}
* Review & Governance {OS}

### Logic

```
OBJECTIVE
 ↓
PROJECT
 ↓
PLAN
 ↓
EXECUTION
 ↓
TEAM / AGENTS
 ↓
MEETINGS
 ↓
OPERATIONS
 ↓
AUTOMATION
 ↓
KPI
 ↓
REVIEW
 ↓
DECISION
 ↓
IMPROVEMENT
```

Operator {OS} is essentially the **business orchestrator** of this stack.

---

## 09. Wealth / Ownership Stack

* Money {OS}
* Wealth {OS}
* Ownership {OS}
* IP & Asset {OS}
* Revenue {OS}
* Business Strategy {OS}
* Exit & Liquidity {OS}
* 0TO100M {OS}

### Logic

```
INCOME
 ↓
CASH FLOW
 ↓
SURPLUS
 ↓
CAPITAL
 ↓
OWNERSHIP
 ↓
ASSETS
 ↓
LEVERAGE
 ↓
COMPOUNDING
 ↓
LIQUIDITY
 ↓
REINVESTMENT
```

### 0TO100M sits above

```
                  0TO100M {OS}
                       │
       ┌───────────────┼───────────────┐
       ▼               ▼               ▼
    BUSINESS          WEALTH         CAPITAL
       │               │               │
       └───────────────┼───────────────┘
                       ▼
                    OWNERSHIP
                       ↓
                    LEVERAGE
```

It is a **meta-strategy OS**, not simply a Money OS.

---

## 10. Capital / Investment Stack

* Capital {OS}
* Investment Thesis {OS}
* Deal Flow {OS}
* Due Diligence {OS}
* Acquisition {OS}
* Deal Structuring {OS}
* Portfolio Management {OS}
* Board {OS}

### Logic

```
CAPITAL
 ↓
THESIS
 ↓
SEARCH
 ↓
DEAL FLOW
 ↓
SCREEN
 ↓
DUE DILIGENCE
 ↓
VALUATION
 ↓
STRUCTURE
 ↓
DECISION
 ↓
ACQUIRE / INVEST
 ↓
PORTFOLIO
 ↓
VALUE CREATION
 ↓
BOARD / GOVERNANCE
 ↓
EXIT
 ↓
CAPITAL
```

A complete capital loop.

---

## 11. CAIO Stack

The specialized Professional Suite.

### Diagnose

* CAIO Role {OS}
* Company Context {OS}
* Client Qualification {OS}
* Organization Intelligence {OS}
* AI Maturity {OS}
* Process Discovery {OS}
* Data & Knowledge {OS}

### Strategy

* AI Opportunity Mapping {OS}
* AI Portfolio {OS}
* Business Case & ROI {OS}
* AI Strategy {OS}
* CAIO Roadmap {OS}

### Build

* AI Architecture {OS}
* Security & AI Governance {OS}
* Proof of Value {OS}
* AI Implementation {OS}
* AI Evaluation {OS}
* Production Readiness {OS}

### Adoption

* AI Adoption {OS}
* AI Training & Capability {OS}
* AI Change Management {OS}

### Operate

* AI KPI & Value Realization {OS}
* AI Governance Board {OS}
* AI Risk & Incident {OS}
* AI Operations {OS}
* AI Portfolio Review {OS}
* Executive & Board Communication {OS}

### Commercial

* CAIO Positioning {OS}
* CAIO Offer {OS}
* CAIO Sales {OS}
* CAIO Client Delivery {OS}
* CAIO Case Study {OS}

### Full CAIO logic

```
QUALIFY
 ↓
UNDERSTAND COMPANY
 ↓
AI MATURITY
 ↓
PROCESS DISCOVERY
 ↓
OPPORTUNITY MAP
 ↓
PRIORITIZE
 ↓
BUSINESS CASE
 ↓
AI STRATEGY
 ↓
ROADMAP
 ↓
ARCHITECTURE
 ↓
POV
 ↓
IMPLEMENT
 ↓
EVALUATE
 ↓
DEPLOY
 ↓
ADOPT
 ↓
MEASURE VALUE
 ↓
GOVERN
 ↓
IMPROVE
```

---

## The real global model

Something far more powerful than a prompt library:

```
                         USER
                           │
                           ▼
                    INTENT / MISSION
                           │
                           ▼
                 AGENTIK ORCHESTRATOR
                           │
                    CAPABILITY MAP
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
       ▼                   ▼                   ▼
   PERSONAL             BUILD              BUSINESS
       │                   │                   │
       ▼                   ▼                   ▼
   KNOWLEDGE            CONTENT              GROWTH
       │                   │                   │
       └──────────────┬────┴────┬──────────────┘
                      │         │
                      ▼         ▼
                   WEALTH    CAPITAL
                      │         │
                      └────┬────┘
                           ▼
                        RESULT
                           │
                           ▼
                        MEMORY
                           │
                           ▼
                         LEARN
```

## And each OS itself follows this logic

```
INPUT
 ↓
DIAGNOSE
 ↓
DECIDE
 ↓
EXECUTE
 ↓
VERIFY
 ↓
OUTPUT
 ↓
HANDOFF?
├── NO → COMPLETE
└── YES
     ↓
CAPABILITY RESOLUTION
     ↓
OTHER OS
     ↓
RESULT
```

## The key point of the whole architecture

> The OS are the units of intelligence.
> The capabilities are their common language.
> The workflows combine them.
> The bundles organize them.
> The Orchestrator chooses the path.
> The Registry knows what exists.
> And the user keeps control.

This taxonomy is the **Master OS Map v1** for the Forge and AGK.

---

## Appendix A: coverage against the shipped registry

Mechanical diff, generated 2026-08-18 against `OS/_registry.json` (73 units).
This appendix is a STATE report, not doctrine: the map above is the target, the
registry is what currently ships. Regenerate it after every `suite.py` run.

### 00. Core Agentik

| OS | Registry slug | Status |
|----|---------------|--------|
| Agentik Runtime {OS} | `agentik-runtime` | shipped |
| OS Builder / Forge {OS} | `os-builder-os` | shipped |
| Context {OS} | . | **net new** |
| Memory {OS} | . | **net new** |
| Knowledge {OS} | `knowledge-os` | shipped |
| Prompt {OS} | . | **net new** |
| Harness {OS} | . | **net new** |
| Agent {OS} | `agent-os` | shipped |
| Orchestration {OS} | `orchestration-os` | shipped |
| AI Logic {OS} | `ai-logic-os` | shipped |
| Quality & Evaluation {OS} | `quality-evaluation-os` | shipped |
| Audit {OS} | . | **net new** |
| Automation {OS} | `automation-os` | shipped |
| Review & Governance {OS} | `review-governance-os` | shipped |
| Documentation {OS} | `documentation-os` | shipped |

### 01. Personal Evolution

| OS | Registry slug | Status |
|----|---------------|--------|
| Mindset {OS} | `mindset-os` | shipped |
| Identity Shift {OS} | `identity-shift-os` | shipped |
| Alignment {OS} | `alignment-os` | shipped |
| Goal & Life Strategy {OS} | `goal-life-strategy-os` | shipped |
| Decision {OS} | `decision-os` | shipped |
| Execution {OS} | `execution-os` | shipped |
| Routine {OS} | . | **net new** |
| Habit Tracker {OS} | `habit-tracker-os` | shipped |
| Health & Energy {OS} | `health-energy-os` | shipped |
| Journal {OS} | `journal-os` | shipped |
| Intuitive {OS} | `intuitive-os` | shipped |
| Mentor {OS} | . | **net new** |

### 02. Social / Human

| OS | Registry slug | Status |
|----|---------------|--------|
| Social Intelligence {OS} | `social-intelligence-os` | shipped |
| Communication {OS} | . | **net new** |
| Conversation {OS} | . | **net new** |
| Attraction / Seductive {OS} | . | **net new** |
| Relationship {OS} | . | **net new** |
| Relationship & Network {OS} | `network-os` | shipped |

### 03. Learn / Knowledge

| OS | Registry slug | Status |
|----|---------------|--------|
| Books {OS} | . | **net new** |
| Librarian {OS} | `librarian-os` | shipped |
| Research {OS} | `research-os` | shipped |
| Knowledge {OS} | `knowledge-os` | shipped |
| Interest Media {OS} | . | **net new** |
| Context & Memory {OS} | `context-memory-os` | shipped |

### 04. Discover & Strategy

| OS | Registry slug | Status |
|----|---------------|--------|
| Research {OS} | `research-os` | shipped |
| Market Research {OS} | `market-research-os` | shipped |
| Trend & Opportunity {OS} | `trend-opportunity-os` | shipped |
| Customer Discovery {OS} | `customer-discovery-os` | shipped |
| Brainstorm {OS} | `brainstorm-os` | shipped |
| Validation {OS} | `validation-os` | shipped |
| Strategy & Portfolio {OS} | `strategy-portfolio-os` | shipped |
| Business Model {OS} | `business-model-os` | shipped |
| Decision {OS} | `decision-os` | shipped |

### 05. Build

| OS | Registry slug | Status |
|----|---------------|--------|
| Research {OS} | `research-os` | shipped |
| Blueprint {OS} | `blueprint-os` | shipped |
| Design {OS} | `design-os` | shipped |
| Prototype {OS} | `prototype-os` | shipped |
| Stepper {OS} | `stepper-os` | shipped |
| Builder {OS} | `builder-os` | shipped |
| Quality & Evaluation {OS} | `quality-evaluation-os` | shipped |
| Security {OS} | `security-os` | shipped |
| Release {OS} | `release-os` | shipped |
| Documentation {OS} | `documentation-os` | shipped |

### 06. Content / Media

| OS | Registry slug | Status |
|----|---------------|--------|
| AGK-Market {OS} | . | **net new** |
| Positioning {OS} | `positioning-os` | shipped |
| Brand {OS} | `brand-os` | shipped |
| Storyteller {OS} | `storyteller-os` | shipped |
| Content {OS} | `content-os` | shipped |
| Viral {OS} | . | **net new** |
| Interest Media {OS} | . | **net new** |

### 07. Commercial / Growth

| OS | Registry slug | Status |
|----|---------------|--------|
| Positioning {OS} | `positioning-os` | shipped |
| Offer {OS} | `offer-os` | shipped |
| Pricing {OS} | `pricing-os` | shipped |
| Sales {OS} | `sales-os` | shipped |
| Affiliate {OS} | `affiliate-os` | shipped |
| Growth {OS} | `growth-os` | shipped |
| Revenue {OS} | `revenue-os` | shipped |
| Delivery & Customer Success {OS} | `delivery-cs-os` | shipped |
| Relationship & Network {OS} | `network-os` | shipped |

### 08. Operator / Business

| OS | Registry slug | Status |
|----|---------------|--------|
| Operator {OS} | . | **net new** |
| Execution {OS} | `execution-os` | shipped |
| Project {OS} | `project-os` | shipped |
| Client {OS} | `client-os` | shipped |
| Meeting {OS} | `meeting-os` | shipped |
| Documentation {OS} | `documentation-os` | shipped |
| Operations & Automation {OS} | `operations-os` | shipped |
| Process & SOP {OS} | `process-sop-os` | shipped |
| Team & Delegation {OS} | `team-delegation-os` | shipped |
| KPI & Analytics {OS} | `kpi-analytics-os` | shipped |
| Review & Governance {OS} | `review-governance-os` | shipped |

### 09. Wealth / Ownership

| OS | Registry slug | Status |
|----|---------------|--------|
| Money {OS} | `money-os` | shipped |
| Wealth {OS} | `wealth-os` | shipped |
| Ownership {OS} | `ownership-os` | shipped |
| IP & Asset {OS} | `ip-asset-os` | shipped |
| Revenue {OS} | `revenue-os` | shipped |
| Business Strategy {OS} | `business-strategy-os` | shipped |
| Exit & Liquidity {OS} | `exit-liquidity-os` | shipped |
| 0TO100M {OS} | . | **net new** |

### 10. Capital / Investment

| OS | Registry slug | Status |
|----|---------------|--------|
| Capital {OS} | `capital-os` | shipped |
| Investment Thesis {OS} | `investment-thesis-os` | shipped |
| Deal Flow {OS} | `deal-flow-os` | shipped |
| Due Diligence {OS} | `due-diligence-os` | shipped |
| Acquisition {OS} | `acquisition-os` | shipped |
| Deal Structuring {OS} | `deal-structuring-os` | shipped |
| Portfolio Management {OS} | `portfolio-management-os` | shipped |
| Board {OS} | `board-os` | shipped |

### 11. CAIO

| OS | Registry slug | Status |
|----|---------------|--------|
| CAIO Role {OS} | . | **net new** |
| Company Context {OS} | . | **net new** |
| Client Qualification {OS} | . | **net new** |
| Organization Intelligence {OS} | . | **net new** |
| AI Maturity {OS} | . | **net new** |
| Process Discovery {OS} | . | **net new** |
| Data & Knowledge {OS} | . | **net new** |
| AI Opportunity Mapping {OS} | . | **net new** |
| AI Portfolio {OS} | . | **net new** |
| Business Case & ROI {OS} | . | **net new** |
| AI Strategy {OS} | . | **net new** |
| CAIO Roadmap {OS} | . | **net new** |
| AI Architecture {OS} | . | **net new** |
| Security & AI Governance {OS} | . | **net new** |
| Proof of Value {OS} | . | **net new** |
| AI Implementation {OS} | . | **net new** |
| AI Evaluation {OS} | . | **net new** |
| Production Readiness {OS} | . | **net new** |
| AI Adoption {OS} | . | **net new** |
| AI Training & Capability {OS} | . | **net new** |
| AI Change Management {OS} | . | **net new** |
| AI KPI & Value Realization {OS} | . | **net new** |
| AI Governance Board {OS} | . | **net new** |
| AI Risk & Incident {OS} | . | **net new** |
| AI Operations {OS} | . | **net new** |
| AI Portfolio Review {OS} | . | **net new** |
| Executive & Board Communication {OS} | . | **net new** |
| CAIO Positioning {OS} | . | **net new** |
| CAIO Offer {OS} | . | **net new** |
| CAIO Sales {OS} | . | **net new** |
| CAIO Client Delivery {OS} | . | **net new** |
| CAIO Case Study {OS} | . | **net new** |

### In the registry, not named in the map

| Registry slug | Name | Group |
|---|---|---|
| `evaluation-os` | Evaluation {OS} | systems |
| `tool-integration-os` | Tool & Integration {OS} | systems |

**Totals.** 133 map entries (120 distinct), 83 resolve to a shipped OS, 50 are net new. Registry: 73 units, 71 matched by the map, 2 not named in it.

**Note on stack 11.** The CAIO units currently ship as the `caio-*` SKILL suite (`/caio-master`, `/caio-discovery-interview`, `/caio-ai-readiness-assessment`, `/caio-enterprise-workflow-architect`, `/caio-implementation-runbook`, `/caio-run-and-optimize`, `/caio-enablement-and-transfer`), not as registry OS units. Promoting them to OS is a `suite.py` decision, not a doc edit.


# Brainstorm {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install brainstorm-os` | Installs this OS into your environment | Once, first |
| `agentik configure brainstorm-os` | Collects the minimum context it needs | After install |
| `agentik run brainstorm-os` | Starts the OS | Every session |
| `agentik doctor brainstorm-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update brainstorm-os` | Updates to the latest version | When a release lands |
| `agentik eval brainstorm-os` | Runs its evaluation suite | Before trusting it |

## OS commands

The root command is `/brainstorm`. Everything below is a conversational
sub-command of the same session: they share one ledger, one lineage and one set
of stable IDs, and any of them can be issued mid session without losing state.

---

### Entry

#### `/brainstorm <idea>`

Start a new Council session, or recover the existing one for this project. Runs
the default `COUNCIL` mode: frame the idea, spawn three independent cells,
freeze their positions, cross-examine, synthesise.

**When to use it:** the moment you have an intuition worth attacking.
**Returns:** the frame (`BS-FRM`), the three independent positions, the
cross-examination record (`BS-ARG`), the tension map (`BS-TEN`), and the
decision options. Status line: `BRAINSTORM IN PROGRESS`.

#### `/brainstorm --deep <idea>`

Run the full multi-cycle `DEEP` protocol: repeated challenge cycles with method
switching between them, a mandatory red-team pass, an experiment queue, and a
complete handoff packet.

**When to use it:** when the commitment behind the idea is expensive, novel or
hard to reverse.
**Returns:** everything `COUNCIL` returns, plus the per-cycle deltas, the
experiment queue (`BS-EXP`), and the handoff packet.

#### `/continue`

Resume from the exact ledger checkpoint: current round, completed artifacts,
next exact action, remaining challenges, founder decisions outstanding.

**When to use it:** at the start of every session after the first, and after any
context loss.
**Returns:** the checkpoint restated, then execution of the next exact action.

---

### Divergence

#### `/ideate --wild <seed>`

Run the Imagination Chamber with no premature feasibility filtering. The
feasibility cells stay out of this pass by design.

**When to use it:** when the candidates on the table are all variants of the
same assumption.
**Returns:** structurally distant candidates (`BS-IDEA`), each with the frame it
came from, unranked and explicitly unfiltered.

#### `/frame-fission`

Split the problem statement into structurally different frames: another problem,
another actor, another scale, another time horizon, another ownership model,
another scarcity, another worldview, and the no-interface frame.

**When to use it:** before generating solution volume, and any time the session
keeps producing the same shape of answer.
**Returns:** the frame set (`BS-FRM`), each with what it makes obvious and what
it hides.

#### `/signature`

Extract Founder DNA: obsessions, beliefs, taste markers, anti-patterns, unfair
insights, energy preferences and the signature tension.

**When to use it:** at the first session of a project, and whenever the council
starts producing correct concepts the founder will never actually build.
**Returns:** the DNA record, each item marked unconfirmed, partially confirmed
or confirmed. Nothing inferred is presented as stated.

#### `/reframe`

Change the problem statement, actor, scale or time horizon of the current
leading concept, keeping its lineage.

**When to use it:** when the concept is sound but is answering a question that
matters less than the one next to it.
**Returns:** the reframed concept with its parentage recorded.

#### `/expand`

Reopen divergence without discarding locked constraints or locked decisions.

**When to use it:** after a convergence that feels premature.
**Returns:** new candidates that respect every existing lock, with the locks
restated so the widening is honest.

#### `/surface`

Decide the embodiment: mobile, web, desktop, multi-surface, chat, API or agent,
ambient or wearable, physical or spatial, human service, or no interface at all.

**When to use it:** whenever the interface materially changes the concept, and
before any handoff to Blueprint {OS}.
**Returns:** the surface decision (`BS-SRF`) with the rejected surfaces and
their reasons.

#### `/surface --compare mobile,web,desktop,multi`

Run the full Surface Lab matrix over the named candidates, and produce the
prototype plan that would discriminate between them.

**When to use it:** when two surfaces both look defensible.
**Returns:** the scored matrix, the distinct role each surface would play in a
multi-surface concept, the canonical state owner, and the cheapest experiment
that separates the candidates.

---

### Evolution

#### `/genome`

Encode a serious direction as an idea genome: its loci, the mechanism at each
locus, and which loci are load-bearing.

**When to use it:** once a direction is worth developing rather than listing.
**Returns:** the genome (`BS-GEN`), with the loci that can be mutated without
destroying the concept marked separately from those that cannot.

#### `/evolve --generations N`

Mutate, cross, select and record parentage across N bounded generations. Each
generation states its selection pressure and what went extinct.

**When to use it:** when a direction is promising but its current form is
probably not its best form.
**Returns:** the generation record, the survivors, the extinctions, and a
genetic-diversity warning when the population has converged too early.

#### `/mutate`

Generate structural mutations of the leading mechanism, not adjacent ideas.
Changing what the thing does, not what it is called.

**When to use it:** when variants keep arriving that differ only cosmetically.
**Returns:** mutated candidates with the locus changed named explicitly.

#### `/collision <domains>`

Transfer a deep mechanism from a distant domain into the concept: not the
metaphor, the mechanism.

**When to use it:** when the concept is competent and unsurprising.
**Returns:** the transferred mechanism, what it changes structurally, and
whether the transfer survives contact with the constraints.

#### `/worlds`

Test the concept inside counterfactual worlds: a world where the main constraint
vanished, where it doubled, where the incentive flipped.

**When to use it:** to find which assumptions the concept is actually resting
on.
**Returns:** per world, what survives and what collapses, and the assumption
each collapse exposed.

#### `/anomaly`

Preserve and interrogate the strangest coherent survivor rather than letting the
convergence smooth it away.

**When to use it:** just before `/converge`, every time.
**Returns:** the anomaly, why it is coherent, and what it would take to be
right.

#### `/incubate`

Park a concept with an evidence-based resurrection trigger instead of killing
it.

**When to use it:** when a concept is good and the moment is wrong.
**Returns:** the incubation record (`BS-INC`) with the specific external event
or evidence that would wake it.

#### `/portfolio`

Test whether several live concepts compound, coexist, conflict or merely
distract.

**When to use it:** when more than one concept survived and all of them look
attractive.
**Returns:** the coherence thesis, the shared primitives, and the conflicts,
stated as conflicts rather than resolved by preference.

---

### Adversarial

#### `/challenge`

Attack the current leading concept with a lens it has not yet faced.

**When to use it:** after any convergence that arrived too easily.
**Returns:** the new attack, the concept's response, and the cycle delta: what
changed because of this round.

#### `/redteam`

Run the `RED TEAM` mode: premortem, incentive and abuse analysis, second-order
effects, and explicit kill criteria.

**When to use it:** before freezing, always. Before any spend, always.
**Returns:** the failure modes ranked, each with a repair, an experiment or a
kill recommendation. A bare objection is not an accepted output here.

#### `/council <specialists>`

Request a domain-specific specialist bench beyond the three core cells, capped
at two additional mandates.

**When to use it:** only when a specialist attacks a material uncertainty, never
for coverage theatre.
**Returns:** the specialist positions, held to the same contract as the core
cells: assumptions, strongest proposal, strongest objection, falsifier,
confidence.

#### `/audit`

Run `AUDIT` mode over the current session, or over a brainstorm produced
elsewhere. Scores diversity, dissent, evidence discipline, traceability,
decision quality and handoff readiness.

**When to use it:** before declaring convergence, and whenever you inherit a
brainstorm you did not run.
**Returns:** the gate scores with every defect named against the record it
appears in, false convergence included.

---

### Convergence

#### `/research`

Separate what is answerable from fact from what needs an experiment, and route
each out.

**When to use it:** the moment the council starts asserting things about the
market, competitors, prices, laws or users.
**Returns:** the split list, with the fact questions routed to Research {OS} or
Market Research {OS} and the rest turned into hypotheses (`BS-HYP`) with
falsifiers.

#### `/compare <ideas>`

Compare named candidates against declared criteria, with a sensitivity check on
the weights.

**When to use it:** when the choice is between specific candidates rather than
the whole population.
**Returns:** the scored comparison, and which criterion the ranking is actually
sensitive to.

#### `/experiment`

Turn the decisive unknowns into the cheapest tests that could settle them.

**When to use it:** whenever a decision is waiting on something knowable.
**Returns:** the experiment queue (`BS-EXP`), each entry with what it would
settle, what it costs, and which decision it unblocks. Designing the instrument
and signing a threshold belongs to Validation {OS}.

#### `/converge`

Rank the survivors on declared weighted criteria, valuable surprise, founder
signature and surface fit, then classify every decision as locked, provisional,
experiment-first, deferred, incubated or rejected.

**When to use it:** when a choice is due and the gates pass.
**Returns:** one selected concept with a clear recommendation, the dissent
recorded beside it, every rejection with its reason, and only the materially
branching questions left for the founder.

---

### State

#### `/freeze`

Lock the accepted decisions and version the selected concept.

**When to use it:** once, at the end of a converged session.
**Returns:** the versioned frozen concept and the lineage that connects it to
its parents. Requires human approval, and refuses while a critical question is
open or a quality gate is red unless that refusal is explicitly overridden.

#### `/handoff blueprint | research | brief`

Package only the fields the named downstream unit needs, and nothing else.

**When to use it:** after `/freeze`, when the concept is leaving this OS.
**Returns:** the handoff packet and the emitted event. `blueprint` requires
human approval, and states plainly when Market Research {OS} and Validation
{OS} are being skipped.

---

## Command summary

| Command | Group | Does | Returns |
|---|---|---|---|
| `/brainstorm <idea>` | entry | start or recover a Council session | frame, three positions, tensions, decision options |
| `/brainstorm --deep <idea>` | entry | full multi-cycle protocol | cycle deltas, red team, experiment queue, handoff packet |
| `/continue` | entry | resume from the ledger checkpoint | checkpoint restated, next action executed |
| `/ideate --wild <seed>` | divergence | Imagination Chamber, no feasibility filter | structurally distant unranked candidates |
| `/frame-fission` | divergence | structurally different problem frames | the frame set, each with what it hides |
| `/signature` | divergence | extract Founder DNA | DNA record, confirmed versus inferred |
| `/reframe` | divergence | change problem, actor, scale or horizon | the reframed concept with lineage |
| `/expand` | divergence | reopen divergence inside the locks | new candidates, locks restated |
| `/surface` | divergence | choose the embodiment | surface decision plus rejected surfaces |
| `/surface --compare` | divergence | full Surface Lab matrix | scored matrix and the discriminating prototype plan |
| `/genome` | evolution | encode concept loci | the genome, load-bearing loci marked |
| `/evolve --generations N` | evolution | mutate, cross, select over N generations | survivors, extinctions, diversity warning |
| `/mutate` | evolution | structural mutations of the mechanism | mutants with the changed locus named |
| `/collision <domains>` | evolution | transfer a distant mechanism | the transfer and whether it survives the constraints |
| `/worlds` | evolution | counterfactual world tests | what collapses and the assumption it exposed |
| `/anomaly` | evolution | preserve the strangest coherent survivor | the anomaly and what would make it right |
| `/incubate` | evolution | park with a resurrection trigger | incubation record with its wake condition |
| `/portfolio` | evolution | compound, coexist, conflict or distract | coherence thesis, shared primitives, conflicts |
| `/challenge` | adversarial | attack with an unused lens | the attack, the response, the cycle delta |
| `/redteam` | adversarial | premortem, abuse, second-order, kill criteria | ranked failure modes, each with a repair or a kill |
| `/council <specialists>` | adversarial | specialist bench, capped at two | specialist positions under the cell contract |
| `/audit` | adversarial | score the brainstorm itself | gate scores, defects named against the record |
| `/research` | convergence | split facts from experiments | routed fact questions, hypotheses with falsifiers |
| `/compare <ideas>` | convergence | compare named candidates | scored comparison plus weight sensitivity |
| `/experiment` | convergence | cheapest decisive tests | the experiment queue with unblocked decisions |
| `/converge` | convergence | rank survivors and decide | one selected concept, dissent recorded |
| `/freeze` | state | lock decisions, version the concept | frozen version and lineage, approval required |
| `/handoff <target>` | state | package the downstream input | handoff packet and the emitted event |

---

## Deterministic session engine

When a filesystem is available, `scripts/brainstorm_os.py` owns the durable
session state. It is the structural half of this OS: it does not decide
anything, it prevents the ledger from drifting, losing lineage or recycling an
ID. Semantic judgment stays with the Council.

| Subcommand | Does |
|---|---|
| `init <output> --title T [--domain D] [--project-id P] [--depth spark\|imagination\|council\|deep\|red-team\|converge\|audit]` | create a new v3 session file |
| `migrate <session> [--output O]` | migrate a v1 or v2 session to v3 without losing lineage |
| `frame <session> [--idea] [--desired-change] [--central-tension] [--highest-impact-unknown] [--actor] [--constraint] [--non-goal] [--success-signal] [--locked-core]` | update the session frame |
| `dna <session> [--obsession] [--belief] [--taste-marker] [--anti-pattern] [--unfair-insight] [--energy-preference] [--signature-tension] [--confirmation-status]` | update Founder DNA and the signature tension |
| `add <session> <collection> --statement S --status ST [--confidence] [--rationale] [--provenance] [--parent-id] [--relates-to] [--falsifier] [--threshold] [--revisit-trigger] [--resurrection-trigger] [--generation]` | add one typed ledger item and mint its stable ID |
| `surface <session> --type T --statement S [--status] [--confidence]` | record or select a surface candidate |
| `surface-config <session> [--applicability] [--role TYPE=ROLE] [--canonical-state-owner] [--multi-surface-rationale] [--next-surface-trigger]` | configure Surface Lab applicability and per-surface roles |
| `evolve <session> --name N --selection-pressure P --delta D [--operator] [--parent] [--survivor] [--extinct] [--genetic-diversity-warning]` | record one evolutionary generation |
| `portfolio <session> [--active-idea] [--coherence-thesis] [--shared-primitive] [--conflict]` | record the live population and its coherence |
| `checkpoint <session> --name N --delta D [--lens] [--revision] [--stage] [--status] [--non-material]` | record a challenge-cycle delta, including a cycle that changed nothing |
| `audit <session> [--require-pass]` | compute the structural quality gates, and exit non-zero when required to pass |
| `freeze <session> [--level patch\|minor\|major] [--note] [--converged] [--allow-open-critical] [--allow-quality-fail]` | version and freeze the selected concept |
| `export <session> <output>` | export a readable Markdown session |
| `handoff <session> <research\|blueprint\|decision\|creative> <output> [--force]` | export the structured downstream handoff packet |
| `summary <session>` | print the compact session summary and checksum |
| `validate <session>` | validate the session against `assets/session.schema.json` |

The collections `add` writes to are the stable ID families: `sources`
(`BS-SRC`), `frames` (`BS-FRM`), `genomes` (`BS-GEN`), `ideas` (`BS-IDEA`),
`surfaces` (`BS-SRF`), `incubations` (`BS-INC`), `hypotheses` (`BS-HYP`),
`arguments` (`BS-ARG`), `tensions` (`BS-TEN`), `decisions` (`BS-DEC`),
`experiments` (`BS-EXP`), `questions` (`BS-QUE`).

`scripts/install_omega_os.py` runs only on an explicit Omega OS installation
request. `scripts/test_brainstorm_os.py` is the engine's own test suite and is
what `agentik eval brainstorm-os` exercises.

//! The AgentikOS operative-systems suite — registry + status for the OS tab
//! (TUI). 24 operative systems along the value chain, in four groups: PERSONAL
//! (Mindset, Health & Energy, Habit Tracker, Alignment — the BE / ENERGY
//! layer), the BUILD CHAIN (01 Strategy & Portfolio → 02 Brainstorm →
//! 03 Market Research → 04 Blueprint → 05 Design → 06 Stepper → 07 Builder →
//! 08 Quality/Evaluation/Release), GROWTH (Storyteller, Revenue, Delivery &
//! Customer Success, Relationship & Network, Wealth & Capital — the
//! COMMUNICATE → SELL → DELIVER → CAPITAL layer), and SYSTEMS (Execution,
//! Operations & Automation, Review & Governance, Context & Memory, AI Logic,
//! Content, Books — the AUTOMATE / LEARN / meta layer). Each lives under
//! `OS/<slug>/` in the repo (installed to `~/.omega/os/`); payloads arrive as
//! zips via the Deposit box and are unpacked in place. This module answers,
//! cheaply and with NO network: which OSes exist, where they live on THIS
//! machine, and whether their payload has been integrated yet.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Which group of the suite an OS belongs to — the TUI renders one section
/// per group, in declaration order: Personal, Build chain, Growth, Systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsGroup {
    /// The personal BE / ENERGY layer (Mindset, Health & Energy, Habit
    /// Tracker, Alignment).
    Personal,
    /// The product pipeline, in chain order:
    /// 01 Strategy & Portfolio → 02 Brainstorm → 03 Market Research →
    /// 04 Blueprint → 05 Design → 06 Stepper → 07 Builder →
    /// 08 Quality / Evaluation / Release.
    BuildChain,
    /// The go-to-market layer, COMMUNICATE → SELL → DELIVER → CAPITAL
    /// (Storyteller, Revenue, Delivery & Customer Success, Relationship &
    /// Network, Wealth & Capital).
    Growth,
    /// Systems / meta OSes that operate ON a system rather than a product or a
    /// person (Execution, Operations & Automation, Review & Governance,
    /// Context & Memory, AI Logic, Content, Books).
    Systems,
}

/// One operative system of the suite — the static half (identity). The single
/// source of truth: the TUI tab, `OS/README.md` and install parity all derive
/// from `OsProduct::all()`; add or reorder an OS HERE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsProduct {
    /// Directory name under `OS/` — also the id used everywhere.
    pub slug: &'static str,
    /// Display name.
    pub name: &'static str,
    /// One-line focus shown in the detail pane.
    pub tagline: &'static str,
    /// The suite group this OS renders under.
    pub group: OsGroup,
    /// What you can do with this OS — the command surface shown in the OS-tab
    /// detail pane (integrated OSes only). One line per entry; empty for a
    /// pre-integration OS (its detail shows the integration pipeline instead).
    pub commands: &'static [&'static str],
}

impl OsProduct {
    /// The whole suite, grouped and contiguous: PERSONAL, then the BUILD CHAIN
    /// in pipeline order (01→08), then GROWTH, then SYSTEMS. Chain position =
    /// index within the BuildChain group + 1 (derived, never hand-numbered).
    pub fn all() -> &'static [OsProduct] {
        &[
            // ── PERSONAL — the BE / ENERGY layer ───────────────────────────
            OsProduct {
                slug: "mindset-os",
                name: "Mindset OS",
                tagline: "Jim Rohn identity/wellbeing/wealth OS: evidence-labeled coaching, philosophy compiler, 90-day program.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex:",
                    "  /mindset-os   Jim Rohn identity/wellbeing/wealth coach",
                    "  labels every claim E1(established) · E2(promising) · S(spiritual)",
                    "                     P(personal) · C(clinical → a professional)",
                    "omega-mindset CLI (workspaces):",
                    "  new --name --output      the weekly workspace (19 files)",
                    "  score <scorecard.json>   validate + summarize a weekly scorecard",
                    "  challenge --output       the 6-month identity challenge",
                    "                           (180 daily + 26 weekly + 6 monthly + state)",
                    "  coach <dir>              one AI growth pass now (Telegram card)",
                    "  coach <dir> --arm        daily 07:00 auto-coaching (disarmed by default)",
                    "  coach <dir> --disarm     stop the loop",
                ],
            },
            OsProduct {
                slug: "health-energy-os",
                name: "Health & Energy OS",
                tagline: "Build and protect physical and cognitive capacity: sleep, movement, training, nutrition, recovery and stress.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /health · /health-energy-os ):",
                    "  build and protect physical and cognitive capacity",
                    "  sleep · movement · training · nutrition · recovery",
                    "  stress regulation · environment design · energy audits",
                    "  knows when to escalate to a real professional",
                    "  12 specialist agents · 18 skills · 8 protocols · 6 schemas",
                    "→ upstream capacity provider for Habit, Execution and Strategy OS",
                ],
            },
            OsProduct {
                slug: "habit-tracker-os",
                name: "Habit Tracker OS",
                tagline: "Habit Tracker {OS}: conversation-first habit system, deterministic state, adaptive reviews and seasons.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /habits · /habit-tracker-os ):",
                    "  conversation-first habit tracker (chat is the interface)",
                    "  build good habits · reduce unwanted ones",
                    "  morning/evening check-ins · handle urges and lapses",
                    "  adaptive weekly/monthly reviews · recovery seasons",
                    "  behavior experiments · visual progress reports",
                    "→ integrates with Mindset {OS} and Context & Memory {OS}",
                ],
            },
            OsProduct {
                slug: "alignment-os",
                name: "Alignment OS",
                tagline: "Alignment Coach {OS}: a wisdom + decision second brain — clarity, values, right effort, next action.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /coach · /align · /alignment-os ):",
                    "  a wisdom + decision inner counsel (Stoic · Daoist · Rohn synthesis)",
                    "  labels every claim E1..E5 (never metaphysics as science)",
                    "protocols / skills (say the word or the /command):",
                    "  /morning /evening /weekly   the daily/weekly protocols",
                    "  /decision      a structured decision protocol",
                    "  /true_north    reconnect to chosen values",
                    "  /virtue_check  virtue-before-outcome check",
                    "  /dichotomy_control   separate control from non-control",
                    "  /wu_wei        right effort vs forcing",
                    "  /reframe       reframe a situation   ·   /shadow   shadow work",
                    "  /belief_audit  audit a limiting belief   ·   /fear   fear work",
                    "  /meaning       meaning work   ·   /manifestation   grounded",
                    "  /quantum_truth accurate quantum guardrails",
                    "  /personal_philosophy   compile your philosophy",
                    "  /anti_dependency   hand agency back   ·   /reset   3-min reset",
                    "council: stoic·daoist·rohn·scientist·quantum·manifestation·shadow",
                    "         ·compassion·challenger·action·meaning·integrator (routed)",
                    "omega-align   open the coach master agent in a session",
                ],
            },
            // ── BUILD CHAIN — CHOOSE → BUILD → CERTIFY (01→08) ──────────────
            OsProduct {
                slug: "strategy-portfolio-os",
                name: "Strategy & Portfolio OS",
                tagline: "Turn ambition, evidence and constraints into explicit choices, a ranked portfolio of bets and disciplined allocation.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /strategy · /strategy-portfolio-os ):",
                    "  turn ambition + evidence into explicit choices and a ranked portfolio",
                    "  diagnose · set the strategy kernel · rank bets",
                    "  fund / pause / kill each bet",
                    "  allocate time, attention, people and capital before execution",
                    "  12 specialist agents · 20 skills · 6 protocols · 7 schemas",
                    "→ feeds Blueprint (a product bet) and Execution (personal outcomes)",
                ],
            },
            OsProduct {
                slug: "brainstorm-os",
                name: "Brainstorm OS",
                tagline: "Brainstorm {OS}: multi-agent imagination and decision council, lineage and a frozen concept handoff.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /brainstorm · /brainstorm-os ):",
                    "  run the multi-agent imagination + decision council",
                    "  depths: spark · imagination · council · deep",
                    "          red-team · converge · audit",
                    "  modes: challenge · continue · evolve · go-deeper (keeps lineage)",
                    "  independent chambers · Founder DNA · idea genomes · premortems",
                    "→ emits a frozen concept handoff for Market Research / Blueprint",
                ],
            },
            OsProduct {
                slug: "market-research-os",
                name: "Market Research OS",
                tagline: "Market Research {OS}: evidence, validation and a bounded decision before Blueprint.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /market-research · /market-research-os ):",
                    "  compile an idea into a decision-grade research + validation pack",
                    "  verbs: scan · validate · diligence · deep",
                    "         audit · delta · continue · score · handoff",
                    "  depths: SIGNAL(desk) · VALIDATION(primary) · INVESTMENT_GRADE",
                    "  decisions: GO · PIVOT · HOLD · NO-GO · INSUFFICIENT (always bounded)",
                    "→ never validated on desk research alone; emits a Blueprint input manifest",
                ],
            },
            OsProduct {
                slug: "blueprint-os",
                name: "Blueprint OS",
                tagline: "The product-definition compiler: idea to a traceable, gated definition pack.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /blueprint <idea> · /blueprint-os   the product-definition compiler",
                    "  modes: NEW(compile) · RECOVER(rebuild truth) · EXTEND(add module)",
                    "         REVISE(supersede) · AUDIT(gaps/gates) · DELTA(diff+impact)",
                    "  continue(resume) · status · export <view>",
                    "omega-blueprint CLI (canonical state):",
                    "  init         create the state file (--project-id --project-name --request)",
                    "  validate     schema + 20 gates + handoff + checksum (exit 1 on critical)",
                    "  status       progress, revision, next section, checksum",
                    "  checkpoint   advance revision + save the continuation pointer",
                    "  demo         a valid minimal state to read",
                    "→ stops at BLUEPRINT COMPLETE — STEPPER READY, frozen handoff downstream",
                ],
            },
            OsProduct {
                slug: "design-os",
                name: "Design OS (UX/UI)",
                tagline: "Design {OS}: challenge the blueprint into flows, screens, states and a validated Design Handoff.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /design-os ):",
                    "  compile an approved Blueprint into a challenged UX/UI definition",
                    "  it does: challenge the flow · information architecture",
                    "           screen/state contracts · visual system · responsive + a11y",
                    "  maps to shadcn/ui + STAX (the OmegaOS stack)",
                    "omega-designer CLI (contract validators):",
                    "  intake <file>     validate the Blueprint intake schema",
                    "  handoff <file>    validate the Design Handoff (flows/surfaces/evals/seeds)",
                    "  self-test         run the validator self-test",
                    "→ consumes the Blueprint handoff; emits the Design Handoff for Stepper",
                ],
            },
            OsProduct {
                slug: "stepper-os",
                name: "Stepper OS",
                tagline: "Step-by-step execution: a blueprint walked one verified step at a time.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /stepper-os   execute a blueprint step by step (verifier-gated DONE)",
                    "omega-stepper CLI (execution DAG):",
                    "  init         scaffold the project (manifest + sources + first step)",
                    "  validate     schema + DAG + audit Blueprint/Design references resolve",
                    "  status       weighted + raw progress, per-status counts",
                    "  plan         ranked READY candidates + the safe execution wave",
                    "  show / agent-brief   one step's spec / the self-contained agent brief",
                    "  start        claim a READY step (prints the brief: Blueprint + Design refs)",
                    "  verify       run the deterministic checks (no state change)",
                    "  done         verify then close — DONE only if every check passes",
                    "  fail / block / unblock   record a failure / block / lift a block",
                    "  review       record a review verdict (role gate)",
                    "  resume       reconcile interrupted attempts after a restart",
                    "  release-check   PASS only when every target-priority step is DONE",
                    "  report / events   status report / the append-only event log",
                    "→ references BOTH the Blueprint and the Design docs per step",
                ],
            },
            OsProduct {
                slug: "builder-os",
                name: "Builder OS",
                tagline: "The implementation runtime: the Stepper roadmap executed into release-ready code.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /build · /builder-os   the autonomous implementation runtime",
                    "  /build preflight · status · plan · run · step <id> · test · verify",
                    "         repair <id> · audit · resume · pause · release-check · report",
                    "omega-builder CLI (evidence state):",
                    "  init         init from the frozen Blueprint handoff + BUILD READY graph",
                    "  validate / status   structure / evidence-backed status",
                    "  sync-step    mirror one Stepper step (spec-hash, required-for-release)",
                    "  claim        claim a READY step (locks, worktree, branch)",
                    "  transition   apply a valid attempt transition",
                    "  record-check   record deterministic check evidence",
                    "  mark-step    mirror a Stepper status after the Verifier decision",
                    "  gate         evaluate a release gate BG01–BG20",
                    "  checkpoint   a recovery checkpoint",
                    "  set-release / finalize   set candidate / final handoff when all gates pass",
                    "  release-check   evaluate terminal release readiness   ·   demo (self-test)",
                    "→ executes the Stepper roadmap; never a competing TODO list",
                ],
            },
            OsProduct {
                slug: "quality-evaluation-release-os",
                name: "Quality, Evaluation & Release OS",
                tagline: "Independent certification between Builder and production: conformance, observability, recovery and a go/no-go.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /quality · /quality-evaluation-release-os ):",
                    "  independent certification between Builder and production",
                    "  contract conformance · risk management",
                    "  observability · recovery validation",
                    "  controlled release readiness · a bounded go/no-go decision",
                    "  16 specialist agents · 26 skills · 7 protocols · 8 schemas",
                    "→ sits after Builder OS, before production",
                ],
            },
            // ── GROWTH — COMMUNICATE → SELL → DELIVER → CAPITAL ─────────────
            OsProduct {
                slug: "storyteller-os",
                name: "Storyteller OS",
                tagline: "Storyteller {OS}: coach, mine, verify and shape truthful stories without erasing your voice.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /story · /storyteller-os ):",
                    "  /story        find + build a story from a real moment",
                    "  /mine         mine lived material for story signal",
                    "  /interview    a story interview (coach, don't ghost-write)",
                    "  /deepen       make a story deeper (meaning + tension)",
                    "  /shape        shape it into a structure (story model)",
                    "  /write        write it, only when authorized, in YOUR voice",
                    "  /adapt        adapt to a channel (reel/carousel/thread/keynote)",
                    "  /truthcheck   verify claims against the evidence standard",
                    "  /score        score the story   ·   /rehearse   perform it",
                    "  /storybank    manage your bank of stories",
                    "→ coach, do not ghost-write; never erase your voice",
                ],
            },
            OsProduct {
                slug: "revenue-os",
                name: "Revenue OS",
                tagline: "The conversational revenue brain: offers, pricing, pipeline, invoicing, collections, cash flow and forecasting.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /revenue · /revenue-os ):",
                    "  the conversational revenue brain + governed business ledger",
                    "  offers · pricing · leads · pipeline · sales calls · proposals",
                    "  invoicing · collections · cash flow · forecast · monthly close",
                    "  renewal + expansion",
                    "  24 agents · 40 skills · 10 protocols · 14 schemas",
                    "→ owns the BUSINESS ledger only; never personal money",
                ],
            },
            OsProduct {
                slug: "delivery-customer-success-os",
                name: "Delivery & Customer Success OS",
                tagline: "The customer journey after the sale: onboarding, delivery, adoption, value proof, renewal and expansion.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /delivery · /delivery-customer-success-os ):",
                    "  manage the customer journey after commercial commitment",
                    "  handoff · onboarding · discovery · success plan",
                    "  delivery · scope · communication · acceptance",
                    "  adoption · value proof · renewal · expansion · offboarding",
                    "  19 agents · 30 skills · 9 protocols · 9 schemas",
                    "→ receives signed scope; returns value + health evidence to Revenue",
                ],
            },
            OsProduct {
                slug: "relationship-network-os",
                name: "Relationship & Network OS",
                tagline: "Build, protect and deepen valuable relationships: attention, memory, generous relevance, follow-through, introductions.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /network · /relationship-network-os ):",
                    "  build, protect and deepen valuable human relationships",
                    "  attention · relationship memory · generous relevance",
                    "  follow-through · boundaries · communication",
                    "  thoughtful introductions · network stewardship",
                    "  12 agents · 18 skills · 7 protocols · 6 schemas",
                ],
            },
            OsProduct {
                slug: "wealth-capital-os",
                name: "Wealth & Capital OS",
                tagline: "The personal financial brain: cash flow, savings, emergency resilience, debt, investment policy and allocation.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /wealth · /wealth-capital-os ):",
                    "  the personal financial brain (personal money only)",
                    "  cash flow · savings · emergency resilience · debt · goals",
                    "  investment policy · risk tolerance",
                    "  life-aligned capital allocation",
                    "  12 agents · 20 skills · 7 protocols · 7 schemas",
                    "→ receives only verified owner distributions from Revenue OS",
                ],
            },
            // ── SYSTEMS — AUTOMATE / LEARN / meta ──────────────────────────
            OsProduct {
                slug: "execution-os",
                name: "Execution OS",
                tagline: "LLM-first personal execution: ambitions into focused commitments, protected work and shipped evidence.",
                group: OsGroup::Systems,
                commands: &[
                    "/execution-os · /execute   personal execution as a closed control loop",
                    "  Capture → Clarify → Select → Commit → Focus → Prove → Review → Adapt",
                    "omega-execution CLI:",
                    "  init --owner        create your execution state",
                    "  boot                open the day (capacity GREEN/AMBER/RED + must-win)",
                    "  capture             inbox a raw outcome/commitment/idea",
                    "  add-outcome         a measurable outcome to pursue",
                    "  add-commitment      a commitment with one physical next action",
                    "  start / complete    complete requires evidence + acceptance",
                    "  focus / focus-end   protect a 15/25/50/90-min block",
                    "  block / unblock     adaptive recovery, each with a next action",
                    "  defer / cancel / delegate   move or hand off a commitment",
                    "  add-promise         a stakeholder promise (notice-by + consequence)",
                    "  halt                close the day (proof + tomorrow's first action)",
                    "  reset               weekly reset (truth + next-week win + experiment)",
                    "  audit               monthly system audit (system change)",
                ],
            },
            OsProduct {
                slug: "operations-automation-os",
                name: "Operations & Automation OS",
                tagline: "See how the work really runs, cut waste, then standardize, delegate or automate with monitoring and recovery.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /operations · /operations-automation-os ):",
                    "  see how the work really runs, then remove waste and control gaps",
                    "  interview and observe · reveal waste and control gaps",
                    "  triage: remove · simplify · standardize · delegate · automate",
                    "  production-ready automation blueprints with monitoring + recovery",
                    "  24 agents · 39 skills · 9 protocols · 9 schemas",
                ],
            },
            OsProduct {
                slug: "review-governance-os",
                name: "Review & Governance OS",
                tagline: "Turn actions, incidents, metrics and decisions into honest learning, controlled change and explicit policy.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /review · /review-governance-os ):",
                    "  turn actions, incidents, metrics and decisions into honest learning",
                    "  incident review · postmortem · retrospective",
                    "  policy change · continuously improved systems",
                    "  governs consequential change across every other OS (approval authority)",
                    "  13 agents · 20 skills · 7 protocols · 7 schemas",
                    "→ closes the learning loop with Context & Memory OS",
                ],
            },
            OsProduct {
                slug: "context-memory-os",
                name: "Context & Memory OS",
                tagline: "One trustworthy, inspectable, permissioned memory layer: fact vs inference, no cross-project or identity bleed.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /memory · /context-memory-os ):",
                    "  one trustworthy, inspectable, permissioned memory layer for every OS",
                    "  separates fact from inference and temporary state",
                    "  no cross-project or identity bleed",
                    "  permissioned knowledge retrieval",
                    "  14 agents · 20 skills · 7 protocols · 8 schemas",
                    "→ the canonical shared context layer the other OSes recover from",
                ],
            },
            OsProduct {
                slug: "ai-logic-os",
                name: "AI Logic OS",
                tagline: "Workflow optimizer + agentic-system challenger: code-vs-AI arbitration, default bias NO, finds the logic gaps and specs the fix.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /ai-logic · /ailogic · /ai-logic-os ):",
                    "  a technical adviser whose default bias is NO",
                    "  ~80% deterministic code / ~20% AI judgment — a model call must justify itself",
                    "two jobs:",
                    "  1. optimize a workflow — map · instrument · triage · design · spec · measure · loop",
                    "     triage bins: Codifier · Augmenter · Garder humain · Supprimer",
                    "  2. challenge an agentic system (OmegaOS / an agent / a pipeline / a tool):",
                    "     - where does a LLM do an `if`'s job?",
                    "     - where is a consequential output unverifiable?",
                    "     - where does an irreversible action lack a human gate?",
                    "     - where is the feedback loop missing?",
                    "     - what primitive is absent and should exist?",
                    "  every finding cites proof (file:line / a rule / a log)",
                    "  output always ends with what it does NOT recommend, and why",
                    "omega-ailogic   open the AI Logic master agent in a session",
                ],
            },
            OsProduct {
                slug: "content-os",
                name: "Content OS",
                tagline: "The full content lifecycle: capture, story mining, writing, production, platform-native packaging, publishing and learning.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /content · /content-os ):",
                    "  the full content lifecycle, positioning through performance",
                    "  daily capture · story mining · research · writing",
                    "  visual/audio/video briefs · platform-native packaging",
                    "  publishing · community engagement · performance learning",
                    "  38 agents · 44 skills · 12 protocols · 10 schemas",
                    "→ packages narrative; Storyteller OS owns the story craft",
                ],
            },
            OsProduct {
                slug: "books-os",
                name: "Books OS",
                tagline: "Your library as an operating system: reading, retention and living knowledge.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /books-os = /alexandria — the librarian ):",
                    "  /setup       calibrate on how YOU learn   ·   /language   reply language",
                    "  /book        full X-Ray of a book   ·   /espresso   90-second version",
                    "  /chapter     a book chapter by chapter   ·   /idea   atlas across many books",
                    "  /compare (/vs)   authors in combat   ·   /apply   to a real business",
                    "  /challenge   10-round sparring on your idea   ·   /decision   decision lab",
                    "  /council     3-5 perspectives   ·   /teach   Feynman triple explanation",
                    "  /quiz /drill   adaptive recall   ·   /cards   flashcards   ·   /review   spaced rep",
                    "  /map (/visual)   diagram   ·   /memory   memory forge   ·   /focus   5-min session",
                    "  /best [topic]   the 50 best books + 50 tips   ·   /bestsellers [niche]   top 100",
                    "  /gem   an underrated idea   ·   /capture /save /applylog   feed the ledger",
                    "omega-books CLI:   open the librarian master agent in a terminal session",
                ],
            },
        ]
    }

    /// "01"…"08" for build-chain OSes (their pipeline position), None for
    /// every other group. Derived from registry order — never hand-numbered.
    pub fn chain_position(&self) -> Option<usize> {
        if self.group != OsGroup::BuildChain {
            return None;
        }
        Self::all()
            .iter()
            .filter(|p| p.group == OsGroup::BuildChain)
            .position(|p| p.slug == self.slug)
            .map(|i| i + 1)
    }
}

/// The dynamic half: has this OS's payload been integrated on this machine?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsStatus {
    /// Directory absent or only the placeholder README — the zip has not
    /// landed yet.
    AwaitingDrop,
    /// The directory carries a payload beyond the placeholder.
    Integrated,
}

/// One row for the OS tab: identity + where it lives here + integration state.
#[derive(Debug, Clone)]
pub struct OsEntry {
    pub product: OsProduct,
    pub status: OsStatus,
    /// `<os_root>/<slug>` when a root was found (the dir itself may not exist
    /// yet for an OS added to the registry before its folder).
    pub path: Option<PathBuf>,
    /// A dedicated Telegram bot is wired for this OS (`os-<slug>` entry in
    /// `~/.omega/agent-bots.json`, linked via `omega-os-bot`).
    pub bot_linked: bool,
}

/// Locate the `OS/` suite root. Order: `OMEGA_OS_ROOT` env override, then the
/// repo relative to the running exe (a dev box running `target/…/omega`), then
/// a walk up from the current dir, then well-known checkouts, then the
/// INSTALLED copy `~/.omega/os` — last, so a checkout always wins and an
/// operator editing the suite sees it immediately. Same resolution grammar as
/// `marketing::capabilities_toml_path`.
pub fn os_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("OMEGA_OS_ROOT") {
        let p = p.trim();
        if !p.is_empty() {
            let pb = PathBuf::from(p);
            if pb.is_dir() {
                return Some(pb);
            }
        }
    }

    let is_suite_root = |d: &Path| -> Option<PathBuf> {
        let cand = d.join("OS");
        // Require a known slug inside, so a random `OS/` dir on the walk-up
        // path can't hijack the suite.
        if cand.is_dir() && cand.join("mindset-os").is_dir() {
            return Some(cand);
        }
        None
    };

    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        while let Some(d) = dir {
            if let Some(found) = is_suite_root(&d) {
                return Some(found);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = Some(cwd);
        while let Some(d) = dir {
            if let Some(found) = is_suite_root(&d) {
                return Some(found);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    for base in [
        home.join("Station").join("SideBusiness").join("OmegaOS"),
        home.join("OmegaOS"),
    ] {
        if let Some(found) = is_suite_root(&base) {
            return Some(found);
        }
    }

    let omega_dir = std::env::var("OMEGA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".omega"));
    let installed = omega_dir.join("os");
    if installed.is_dir() {
        return Some(installed);
    }
    None
}

/// Integrated = the OS dir contains a real payload beyond its scaffold.
/// Scaffold files every OS carries from day one — the placeholder README,
/// the MASTER.md master-agent prompt, the ledger/ working dir a linked
/// Telegram bot accumulates, dotfiles — do NOT count as integration.
/// Fast + local — safe on tab entry / F5.
fn dir_status(dir: &Path) -> OsStatus {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return OsStatus::AwaitingDrop;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "README.md" || name == "MASTER.md" || name == "ledger" || name.starts_with('.') {
            continue;
        }
        return OsStatus::Integrated;
    }
    OsStatus::AwaitingDrop
}

/// Bot keys present in `~/.omega/agent-bots.json` (one read for the list).
fn linked_bot_keys() -> std::collections::HashSet<String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    let omega_dir = std::env::var("OMEGA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".omega"));
    let Ok(raw) = std::fs::read_to_string(omega_dir.join("agent-bots.json")) else {
        return Default::default();
    };
    serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|v| {
            v.as_object()
                .map(|map| map.keys().cloned().collect())
        })
        .unwrap_or_default()
}

/// The whole suite with per-machine status, in product order.
pub fn list_os_entries() -> Vec<OsEntry> {
    let root = os_root();
    let bots = linked_bot_keys();
    OsProduct::all()
        .iter()
        .map(|p| {
            let path = root.as_ref().map(|r| r.join(p.slug));
            let status = path
                .as_deref()
                .filter(|d| d.is_dir())
                .map(dir_status)
                .unwrap_or(OsStatus::AwaitingDrop);
            OsEntry {
                product: *p,
                status,
                path,
                bot_linked: bots.contains(&format!("os-{}", p.slug)),
            }
        })
        .collect()
}

impl OsEntry {
    /// Status glyph for the list: 🟢 integrated / ⚪ awaiting its drop.
    pub fn glyph(&self) -> &'static str {
        match self.status {
            OsStatus::Integrated => "🟢",
            OsStatus::AwaitingDrop => "⚪",
        }
    }

    pub fn status_label(&self) -> &'static str {
        match self.status {
            OsStatus::Integrated => "integrated",
            OsStatus::AwaitingDrop => "awaiting drop (zip via Deposit)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The value-chain suite, in registry order, grouped and contiguous:
    /// PERSONAL, then the BUILD CHAIN (01→08), then GROWTH, then SYSTEMS.
    /// 24 = 23 value-chain OSes + books-os (a working OS with no value-chain
    /// twin, kept in Systems rather than dropped).
    const EXPECTED: &[(&str, OsGroup)] = &[
        ("mindset-os", OsGroup::Personal),
        ("health-energy-os", OsGroup::Personal),
        ("habit-tracker-os", OsGroup::Personal),
        ("alignment-os", OsGroup::Personal),
        ("strategy-portfolio-os", OsGroup::BuildChain),
        ("brainstorm-os", OsGroup::BuildChain),
        ("market-research-os", OsGroup::BuildChain),
        ("blueprint-os", OsGroup::BuildChain),
        ("design-os", OsGroup::BuildChain),
        ("stepper-os", OsGroup::BuildChain),
        ("builder-os", OsGroup::BuildChain),
        ("quality-evaluation-release-os", OsGroup::BuildChain),
        ("storyteller-os", OsGroup::Growth),
        ("revenue-os", OsGroup::Growth),
        ("delivery-customer-success-os", OsGroup::Growth),
        ("relationship-network-os", OsGroup::Growth),
        ("wealth-capital-os", OsGroup::Growth),
        ("execution-os", OsGroup::Systems),
        ("operations-automation-os", OsGroup::Systems),
        ("review-governance-os", OsGroup::Systems),
        ("context-memory-os", OsGroup::Systems),
        ("ai-logic-os", OsGroup::Systems),
        ("content-os", OsGroup::Systems),
        ("books-os", OsGroup::Systems),
    ];

    #[test]
    fn suite_is_the_full_value_chain_in_order() {
        let got: Vec<(&str, OsGroup)> =
            OsProduct::all().iter().map(|p| (p.slug, p.group)).collect();
        assert_eq!(got.len(), 24, "24 = 23 value-chain OSes + books-os");
        assert_eq!(got, EXPECTED);
    }

    #[test]
    fn groups_are_contiguous_in_declaration_order() {
        // Each group appears exactly once as a contiguous run, in the order
        // Personal → BuildChain → Growth → Systems (what the TUI renders).
        let order = [
            OsGroup::Personal,
            OsGroup::BuildChain,
            OsGroup::Growth,
            OsGroup::Systems,
        ];
        let mut seen: Vec<OsGroup> = Vec::new();
        let mut last: Option<OsGroup> = None;
        for p in OsProduct::all() {
            if last != Some(p.group) {
                assert!(
                    !seen.contains(&p.group),
                    "group {:?} is not contiguous",
                    p.group
                );
                seen.push(p.group);
                last = Some(p.group);
            }
        }
        assert_eq!(seen, order, "groups render in value-chain order");
    }

    #[test]
    fn build_chain_positions_are_01_to_08_unique() {
        let positions: Vec<usize> = OsProduct::all()
            .iter()
            .filter(|p| p.group == OsGroup::BuildChain)
            .map(|p| p.chain_position().expect("build-chain OS has a position"))
            .collect();
        assert_eq!(positions, (1..=8).collect::<Vec<_>>());
        // Every non-BuildChain OS returns None.
        for p in OsProduct::all() {
            if p.group != OsGroup::BuildChain {
                assert_eq!(p.chain_position(), None, "{} must have no position", p.slug);
            }
        }
        // Named anchors of the chain.
        let pos = |slug: &str| {
            OsProduct::all()
                .iter()
                .find(|p| p.slug == slug)
                .unwrap()
                .chain_position()
        };
        assert_eq!(pos("strategy-portfolio-os"), Some(1));
        assert_eq!(pos("blueprint-os"), Some(4));
        assert_eq!(pos("builder-os"), Some(7));
        assert_eq!(pos("quality-evaluation-release-os"), Some(8));
        assert_eq!(pos("books-os"), None);
        assert_eq!(pos("mindset-os"), None);
    }

    #[test]
    fn every_slug_resolves_to_an_os_dir_when_a_root_is_present() {
        // With OMEGA_OS_ROOT unset the suite root may not resolve on a bare
        // CI box; only assert dir presence when a root actually resolves.
        if let Some(root) = os_root() {
            for p in OsProduct::all() {
                let dir = root.join(p.slug);
                assert!(
                    dir.is_dir(),
                    "OS payload dir missing for {}: {}",
                    p.slug,
                    dir.display()
                );
            }
        }
    }

    #[test]
    fn placeholder_only_dir_is_awaiting_and_payload_is_integrated() {
        let tmp = std::env::temp_dir().join(format!("os-products-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("README.md"), "# placeholder").unwrap();
        std::fs::write(tmp.join("MASTER.md"), "# master agent").unwrap();
        std::fs::create_dir_all(tmp.join("ledger")).unwrap();
        assert_eq!(dir_status(&tmp), OsStatus::AwaitingDrop);
        std::fs::write(tmp.join("app.py"), "payload").unwrap();
        assert_eq!(dir_status(&tmp), OsStatus::Integrated);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn entries_cover_the_whole_suite() {
        let entries = list_os_entries();
        assert_eq!(entries.len(), OsProduct::all().len());
    }
}

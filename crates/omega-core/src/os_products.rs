//! The AGENTIK {OS} operative-systems suite — registry + status for the
//! OS tab (TUI). 73 operative systems in 9 groups along the
//! value chain:
//!
//!   00 · RUNTIME              2 units — Build and run the entire Agentik ecosystem.
//!   01 · PERSONAL            10 units — Operate yourself.
//!   02 · DISCOVER & DECIDE    9 units — Find what is worth building.
//!   03 · BUILD                8 units — Turn evidence into products.
//!   04 · GROW                12 units — Turn value into distribution and revenue.
//!   05 · OPERATE             10 units — Turn chaos into repeatable execution.
//!   06 · OWN                  6 units — Turn work into assets.
//!   07 · CAPITAL              8 units — Turn assets into capital allocation.
//!   08 · AI & SYSTEMS         8 units — Intelligence infrastructure for everything.
//!
//! Each lives under `OS/<slug>/` in the repo (installed to `~/.omega/os/`).
//! This module answers, cheaply and with NO network: which OSes exist,
//! where they live on THIS machine, and which concrete readiness surfaces
//! are present. Static presence is never reported as runtime verification.
//!
//! GENERATED from `OS/_registry.json` by `OS/_tools/gen_os_products.py`.
//! Add or reorder an OS in `OS/_tools/suite.py`, then re-run the generator.
//! Everything below `OsReadinessLevel` is hand-written and spliced through.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Which group of the suite an OS belongs to — the TUI renders one section
/// per group, in declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsGroup {
    /// 00 · RUNTIME — Build and run the entire Agentik ecosystem.
    Runtime,
    /// 01 · PERSONAL — Operate yourself.
    Personal,
    /// 02 · DISCOVER & DECIDE — Find what is worth building.
    Discover,
    /// 03 · BUILD — Turn evidence into products.
    Build,
    /// 04 · GROW — Turn value into distribution and revenue.
    Grow,
    /// 05 · OPERATE — Turn chaos into repeatable execution.
    Operate,
    /// 06 · OWN — Turn work into assets.
    Own,
    /// 07 · CAPITAL — Turn assets into capital allocation.
    Capital,
    /// 08 · AI & SYSTEMS — Intelligence infrastructure for everything.
    Systems,
}

impl OsGroup {
    /// Every group, in render order. Callers iterate this instead of
    /// hand-listing variants, so adding a group never breaks a renderer.
    pub fn all() -> &'static [OsGroup] {
        &[
            OsGroup::Runtime,
            OsGroup::Personal,
            OsGroup::Discover,
            OsGroup::Build,
            OsGroup::Grow,
            OsGroup::Operate,
            OsGroup::Own,
            OsGroup::Capital,
            OsGroup::Systems,
        ]
    }

    /// Short section heading shown in the TUI and the gateway.
    pub fn label(&self) -> &'static str {
        match self {
            OsGroup::Runtime => "00 · RUNTIME",
            OsGroup::Personal => "01 · PERSONAL",
            OsGroup::Discover => "02 · DISCOVER & DECIDE",
            OsGroup::Build => "03 · BUILD",
            OsGroup::Grow => "04 · GROW",
            OsGroup::Operate => "05 · OPERATE",
            OsGroup::Own => "06 · OWN",
            OsGroup::Capital => "07 · CAPITAL",
            OsGroup::Systems => "08 · AI & SYSTEMS",
        }
    }

    /// What this group is for, one line.
    pub fn purpose(&self) -> &'static str {
        match self {
            OsGroup::Runtime => "Build and run the entire Agentik ecosystem",
            OsGroup::Personal => "Operate yourself",
            OsGroup::Discover => "Find what is worth building",
            OsGroup::Build => "Turn evidence into products",
            OsGroup::Grow => "Turn value into distribution and revenue",
            OsGroup::Operate => "Turn chaos into repeatable execution",
            OsGroup::Own => "Turn work into assets",
            OsGroup::Capital => "Turn assets into capital allocation",
            OsGroup::Systems => "Intelligence infrastructure for everything",
        }
    }
}

/// One operative system of the suite — the static half (identity). The single
/// source of truth: the TUI tab, `OS/README.md` and install parity all derive
/// from `OsProduct::all()`; add an OS in `OS/_tools/suite.py`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsProduct {
    /// Canonical suite number (00..71) in the AGENTIK {OS} tree.
    pub num: u8,
    /// Directory name under `OS/` — also the id used everywhere.
    pub slug: &'static str,
    /// Display name.
    pub name: &'static str,
    /// One-line focus shown in the detail pane.
    pub tagline: &'static str,
    /// The suite group this OS renders under.
    pub group: OsGroup,
    /// What you can do with this OS — the command surface shown in the OS-tab
    /// detail pane as declared capabilities. One line per entry; empty for a
    /// pre-integration OS (its detail shows the integration pipeline instead).
    pub commands: &'static [&'static str],
}

impl OsProduct {
    /// The whole suite: 73 units, grouped and contiguous, in registry order.
    pub fn all() -> &'static [OsProduct] {
        &[
            // ── 00 · RUNTIME — Build and run the entire Agentik ecosystem ────
            OsProduct {
                num: 0,
                slug: "os-builder-os",
                name: "OS Builder {OS}",
                tagline: "Build an operative system itself: intake, spec, research, build, red team, score, release.",
                group: OsGroup::Runtime,
                commands: &[],
            },
            OsProduct {
                num: 1,
                slug: "agentik-runtime",
                name: "Agentik Runtime {OS}",
                tagline: "Install, configure, run, compose, update and evaluate every Agentik OS.",
                group: OsGroup::Runtime,
                commands: &[],
            },
            // ── 01 · PERSONAL — Operate yourself ─────────────
            OsProduct {
                num: 2,
                slug: "mindset-os",
                name: "Mindset {OS}",
                tagline: "Identity and beliefs: the behavioural compiler under Alignment.",
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
                num: 3,
                slug: "identity-shift-os",
                name: "Identity Shift {OS}",
                tagline: "Deliberate identity change: who you must become before what you must do.",
                group: OsGroup::Personal,
                commands: &[],
            },
            OsProduct {
                num: 4,
                slug: "alignment-os",
                name: "Alignment {OS}",
                tagline: "Meaning, values and inner alignment: the BE authority of the suite.",
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
            OsProduct {
                num: 5,
                slug: "goal-life-strategy-os",
                name: "Goal & Life Strategy {OS}",
                tagline: "Life-level goals and the strategy that makes them reachable.",
                group: OsGroup::Personal,
                commands: &[],
            },
            OsProduct {
                num: 6,
                slug: "habit-tracker-os",
                name: "Habit Tracker {OS}",
                tagline: "Recurring behaviour contracts and the evidence they actually happened.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /habits · /habits-os · /habit-tracker-os ):",
                    "  conversation-first habit coach (the chat is the interface)",
                    "  modes it routes: SETUP · TODAY · CHECK_IN · URGE · LAPSE",
                    "                   REVIEW · RECOVER · ADAPT · VISUALIZE",
                    "  build good habits · reduce unwanted ones (friction + replacement)",
                    "  labels every record explicit · observed · inferred · proposed",
                    "  seasons: build · maintain · recover · travel · crisis",
                    "  safety-and-boundaries.md routes clinical risk to a professional",
                    "omega-habits CLI (deterministic state, per-user SQLite):",
                    "  init                     create/update the user profile",
                    "  add · update · list      accept, version, list habit contracts",
                    "  log · correct            append then supersede explicit/observed evidence",
                    "  today                    rank today's primary habits (seven maximum)",
                    "  review                   compute an evidence-bounded review",
                    "  chart                    render a Mermaid progress diagram",
                    "  context · export         compact LLM context · export user-owned state",
                    "  season · experiment      change season · create a bounded experiment",
                    "  delete · doctor          delete user-owned state · validate integrity",
                    "→ integrates with Mindset {OS} and Context & Memory {OS}",
                ],
            },
            OsProduct {
                num: 7,
                slug: "health-energy-os",
                name: "Health & Energy {OS}",
                tagline: "Physical and cognitive capacity: sleep, movement, fuel, recovery, stress.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /health · /health-energy-os ):",
                    "  /health             open Health & Energy OS (daily check-in)",
                    "  /readiness          assess today's capacity",
                    "  /health-audit       build a capacity baseline",
                    "  /sleep              audit sleep and circadian constraints",
                    "  /training           build or revise training",
                    "  /nutrition          review fuel and adherence",
                    "  /recovery           respond to fatigue or overload",
                    "  /travel-health      design a travel and jet-lag protocol",
                    "  /health-experiment  create an N-of-1 experiment",
                    "  /wearable           interpret device trends conservatively",
                    "  12 agents · 18 skills · 8 protocols · 6 schemas (say the word or the /command)",
                    "reference runtime (stdlib Python, provider-neutral):",
                    "  runtime/os_runtime.py info | route /health | event | validate",
                    "→ upstream capacity provider for Habit, Execution and Strategy OS",
                ],
            },
            OsProduct {
                num: 8,
                slug: "intuitive-os",
                name: "Intuitive {OS}",
                tagline: "Train and calibrate intuition as a usable, falsifiable signal.",
                group: OsGroup::Personal,
                commands: &[],
            },
            OsProduct {
                num: 9,
                slug: "decision-os",
                name: "Decision {OS}",
                tagline: "Make hard calls well: framing, reversibility, evidence, the decision record.",
                group: OsGroup::Personal,
                commands: &[],
            },
            OsProduct {
                num: 10,
                slug: "journal-os",
                name: "Journal {OS}",
                tagline: "Reflection that compounds: capture, revisit, extract the pattern.",
                group: OsGroup::Personal,
                commands: &[],
            },
            OsProduct {
                num: 11,
                slug: "social-intelligence-os",
                name: "Social Intelligence {OS}",
                tagline: "Read rooms and people accurately, and act with integrity.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /seduction · /charisma · /seductive-os ):",
                    "  the personal magnetism coach (presence, conversation, inner game)",
                    "  builds a more compelling PERSON, never scripts, lines or routines",
                    "  labels every claim E1(replicated) · E2(thin) · E3(craft)",
                    "                     P(personal taste) · C(clinical → a professional)",
                    "modes (say the word or the /command):",
                    "  /presence      embodiment, voice, stillness, attention",
                    "  /conversation  curiosity, stories, humour, depth   ·   /style",
                    "  /innergame     self-worth under the skill   ·   /rejection",
                    "  /calibrate     read real interest vs your own inference",
                    "  /flirt         consent-gated   ·   /date   design a real one",
                    "  /apps          the digital half   ·   /desire   long-term",
                    "  /audit /practice /debrief /reset",
                    "ethics: consent is the product, not the constraint — refusals.md",
                    "        names the manipulative playbook it declines, and why each",
                    "        one also fails on YOUR terms. Ethics guardian holds a veto.",
                    "omega-seduction   open the magnetism master agent in a session",
                    "→ pairs with Mindset (identity), Health & Energy (the substrate),",
                    "  Alignment (values) and Relationship & Network (the platonic half)",
                ],
            },
            // ── 02 · DISCOVER & DECIDE — Find what is worth building ─────────
            OsProduct {
                num: 12,
                slug: "librarian-os",
                name: "Librarian {OS}",
                tagline: "Your reading and source corpus turned into retrievable understanding.",
                group: OsGroup::Discover,
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
            OsProduct {
                num: 13,
                slug: "research-os",
                name: "Research {OS}",
                tagline: "General-purpose evidence gathering with sources you can defend.",
                group: OsGroup::Discover,
                commands: &[],
            },
            OsProduct {
                num: 14,
                slug: "trend-opportunity-os",
                name: "Trend & Opportunity {OS}",
                tagline: "Spot movement early and turn it into a named opportunity.",
                group: OsGroup::Discover,
                commands: &[],
            },
            OsProduct {
                num: 15,
                slug: "brainstorm-os",
                name: "Brainstorm {OS}",
                tagline: "Generate and evolve ideas before research or definition.",
                group: OsGroup::Discover,
                commands: &[
                    "Claude / Codex ( /brainstorm · /brainstorm-os ):",
                    "  run the multi-agent imagination + decision council",
                    "  modes: spark · imagination · council · deep · red-team · converge · audit",
                    "  keeps lineage across: challenge · continue · evolve · go-deeper",
                    "  /ideate --wild   diverge without premature feasibility filtering",
                    "  /frame-fission   structurally different problem + worldview frames",
                    "  /signature       extract confirmed + inferred Founder DNA",
                    "  /genome · /evolve --generations N   encode + mutate concept loci",
                    "  /collision · /worlds · /anomaly     distant transfer, counterfactuals, survivor",
                    "  /surface --compare mobile,web,desktop,multi   embodiment lab matrix",
                    "  /challenge · /reframe · /mutate · /redteam    attack the leading concept",
                    "  /council <specialists> · /research · /compare · /converge · /experiment",
                    "  /audit · /freeze · /incubate · /portfolio · /continue",
                    "  /handoff blueprint | research | brief   package the frozen concept",
                    "brainstorm_os.py session engine (scripts/, when a filesystem is present):",
                    "  init · migrate · record frames/genomes/generations · compare surfaces",
                    "  incubate · audit · freeze · export · handoff · validate (session.schema.json)",
                    "→ emits a frozen concept handoff for Market Research / Blueprint",
                ],
            },
            OsProduct {
                num: 16,
                slug: "strategy-portfolio-os",
                name: "Strategy & Portfolio {OS}",
                tagline: "Choose the bets: goals, projects and resource allocation.",
                group: OsGroup::Discover,
                commands: &[
                    "Claude / Codex ( /strategy · /strategy-portfolio-os ):",
                    "  turn ambition + evidence into explicit choices and a ranked portfolio",
                    "  diagnose the critical challenge · set the strategy kernel · rank bets",
                    "router modes (say the word or the /command):",
                    "  /strategy            open strategic design (the strategy kernel)",
                    "  /diagnosis           define the critical challenge",
                    "  /portfolio           review all projects and bets",
                    "  /prioritize          rank competing initiatives",
                    "  /scenario            build future scenarios and signposts",
                    "  /strategic-decision  structure a consequential choice",
                    "  /quarter-plan        create the quarterly strategy",
                    "  /kill-review         decide continue / pivot / pause / kill",
                    "  /one-page-strategy   a concise strategy memo",
                    "  /not-doing           define the exclusions (the not-doing list)",
                    "  fund / pause / kill each bet · allocate time, attention, people, capital",
                    "  labels claims E1(evidence) · E2 · E3 · E4 · E5(preference)",
                    "  12 specialist agents · 20 skills · 6 protocols · 7 schemas",
                    "→ feeds Blueprint (a product bet) and Execution (personal outcomes)",
                ],
            },
            OsProduct {
                num: 17,
                slug: "market-research-os",
                name: "Market Research {OS}",
                tagline: "Market and customer evidence, and the validation that follows.",
                group: OsGroup::Discover,
                commands: &[
                    "Claude / Codex ( /market-research · /market-research-os ):",
                    "  compile an idea into a decision-grade research + validation pack",
                    "  modes: NEW · RECOVER · RAPID_SCAN · FULL_VALIDATION · DILIGENCE",
                    "         DEEP_DIVE · MONITOR · AUDIT · DELTA (inferred per run)",
                    "  depths: SIGNAL(desk) · VALIDATION(primary+behavioral) · INVESTMENT_GRADE",
                    "  decisions: GO · PIVOT · HOLD · NO-GO · INSUFFICIENT (always bounded)",
                    "  never validated on desk research alone; ethical-scraping preflight first",
                    "market_research_os.py CLI (deterministic workspace):",
                    "  init <ws> --project-id --project-name --decision   create the state file",
                    "  validate <ws> [--strict]   schema + quality gates (exit 1 on critical)",
                    "  status <ws> · score <ws>   progress, gate + hypothesis diagnostics",
                    "  checkpoint <ws> --current --next   restart-safe continuation pointer",
                    "  allocate <ws> <prefix>   monotonic stable IDs   ·   export <ws> --output",
                    "  demo <ws>   a valid minimal workspace to read",
                    "→ emits a Blueprint Input Manifest on GO / PIVOT; install_omega_os.py ports the pack",
                ],
            },
            OsProduct {
                num: 18,
                slug: "customer-discovery-os",
                name: "Customer Discovery {OS}",
                tagline: "Talk to real users and extract what they actually need.",
                group: OsGroup::Discover,
                commands: &[],
            },
            OsProduct {
                num: 19,
                slug: "validation-os",
                name: "Validation {OS}",
                tagline: "Kill or confirm an idea with the cheapest sufficient test.",
                group: OsGroup::Discover,
                commands: &[],
            },
            OsProduct {
                num: 20,
                slug: "business-model-os",
                name: "Business Model {OS}",
                tagline: "How value is created, delivered and captured, made explicit.",
                group: OsGroup::Discover,
                commands: &[],
            },
            // ── 03 · BUILD — Turn evidence into products ─────────
            OsProduct {
                num: 21,
                slug: "blueprint-os",
                name: "Blueprint {OS}",
                tagline: "The product-definition compiler: a complete, traceable definition pack.",
                group: OsGroup::Build,
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
                num: 22,
                slug: "design-os",
                name: "Design {OS}",
                tagline: "UX, interaction and visual design compiled into a machine-readable handoff.",
                group: OsGroup::Build,
                commands: &[
                    "Claude / Codex ( /design-os ):",
                    "  compile an approved Blueprint into a challenged UX/UI definition + a validated Design Handoff",
                    "  challenge the flow · information architecture · screen/state contracts",
                    "  interaction + visual system · responsive + accessibility + trust states",
                    "  maps to shadcn/ui + STAX (the OmegaOS stack)",
                    "  modes: FULL(default) · AUDIT(challenge an existing design) · FLOW(selected journeys)",
                    "         AI_APP(chat/agents/artifacts) · STAX_FIT(panel-model decision) · REVISION(impacted IDs)",
                    "omega-designer CLI (contract validators, stdlib Python):",
                    "  intake <blueprint-intake.json>   validate the Blueprint intake schema",
                    "  handoff <design-handoff.json>    validate the Design Handoff (flows/surfaces/states/evals/seeds)",
                    "  self-test                        run the validator self-test",
                    "→ consumes the Blueprint handoff; emits STEPPER_READY design-handoff.json for Stepper",
                ],
            },
            OsProduct {
                num: 23,
                slug: "prototype-os",
                name: "Prototype {OS}",
                tagline: "The cheapest artifact that answers the riskiest open question.",
                group: OsGroup::Build,
                commands: &[],
            },
            OsProduct {
                num: 24,
                slug: "stepper-os",
                name: "Stepper {OS}",
                tagline: "The dependency-aware step graph and its deterministic verification gate.",
                group: OsGroup::Build,
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
                num: 25,
                slug: "builder-os",
                name: "Builder {OS}",
                tagline: "The implementation runtime: steps executed into release-ready code.",
                group: OsGroup::Build,
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
                num: 26,
                slug: "quality-evaluation-os",
                name: "Quality & Evaluation {OS}",
                tagline: "Independent certification of what was built, before it ships.",
                group: OsGroup::Build,
                commands: &[
                    "Claude / Codex ( /quality · /quality-evaluation-release-os ):",
                    "  independent quality, evaluation and release authority",
                    "  positioned between Builder OS and production (certify on evidence)",
                    "  16 specialist agents · 26 skills · 7 protocols · 8 schemas",
                    "modes (say the word or the /command):",
                    "  /quality             open the quality authority (intake contracts + scope)",
                    "  /test-plan           build a risk-based test plan",
                    "  /traceability        map requirements to evidence (bidirectional matrix)",
                    "  /qa                  run functional and exploratory QA",
                    "  /ai-eval             design and run AI / agent evaluations",
                    "  /security-review     apply OWASP / threat-model security standards",
                    "  /accessibility       audit WCAG 2.2 / mobile accessibility",
                    "  /release-candidate   assemble the release-candidate evidence pack",
                    "  /release-gate        issue the go / no-go release decision",
                    "  /deploy              execute a controlled (canary / progressive) release",
                    "  /rollback            trigger or prepare a rollback",
                    "reference runtime (stdlib Python, integrity + routing):",
                    "  os_runtime.py info                 name / version / slug / purpose",
                    "  os_runtime.py route \"/quality\"      resolve a command to its mode",
                    "  os_runtime.py validate             sha256-check every packaged file",
                    "  os_runtime.py event <kind> <json>  append an event record",
                    "→ Builder OS builds; Quality certifies, gates and authorizes release",
                ],
            },
            OsProduct {
                num: 27,
                slug: "security-os",
                name: "Security {OS}",
                tagline: "Threat modelling, hardening and the security gate on a release.",
                group: OsGroup::Build,
                commands: &[],
            },
            OsProduct {
                num: 28,
                slug: "release-os",
                name: "Release {OS}",
                tagline: "Ship it: release boundaries, rollout, rollback and the incident path.",
                group: OsGroup::Build,
                commands: &[],
            },
            // ── 04 · GROW — Turn value into distribution and revenue ─────
            OsProduct {
                num: 29,
                slug: "positioning-os",
                name: "Positioning {OS}",
                tagline: "The category you compete in and the claim you own inside it.",
                group: OsGroup::Grow,
                commands: &[],
            },
            OsProduct {
                num: 30,
                slug: "brand-os",
                name: "Brand {OS}",
                tagline: "Identity, voice and the visual system that carries them.",
                group: OsGroup::Grow,
                commands: &[],
            },
            OsProduct {
                num: 31,
                slug: "storyteller-os",
                name: "Storyteller {OS}",
                tagline: "Narrative truth, structure, voice and consent.",
                group: OsGroup::Grow,
                commands: &[
                    "Claude / Codex ( /story · /storyteller-os ):",
                    "  default to COACH, never ghost-write; never erase your voice",
                    "  /story        find + build a story from a real moment",
                    "  /mine · /interview · /moment    mine lived material for story signal",
                    "  /deepen · /shape · /hook · /scene · /arc    meaning + tension + structure",
                    "  /cowrite · /write · /rewrite · /voice    create + edit, only when authorized",
                    "  /adapt · /content · /keynote · /pitch    adapt to a channel, keep story DNA",
                    "  /brandstory · /customerstory · /datastory    business + evidence stories",
                    "  /truthcheck   verify claims against the evidence standard (a hard gate)",
                    "  /score · /rehearse · /feedback   score, perform, learn from response",
                    "  /storybank · /repurpose · /story-review   run your bank of stories",
                    "omega-story CLI (local story bank, SQLite, no network):",
                    "  init · capture · list · show · update    the canonical Story Objects",
                    "  add-claim · add-consent    claim ledger + consent records",
                    "  validate · score · doctor    structural completeness + bank health",
                    "  export --format jsonl|json|markdown    portable export",
                    "  default bank ~/.omega/os/storytelling-os/ledger/story-bank.db",
                    "→ coach, do not ghost-write; truth + consent are gates, not bonus points",
                ],
            },
            OsProduct {
                num: 32,
                slug: "offer-os",
                name: "Offer {OS}",
                tagline: "The thing you sell, shaped so the value is obvious.",
                group: OsGroup::Grow,
                commands: &[],
            },
            OsProduct {
                num: 33,
                slug: "pricing-os",
                name: "Pricing {OS}",
                tagline: "What to charge, how to package it and when to change it.",
                group: OsGroup::Grow,
                commands: &[],
            },
            OsProduct {
                num: 34,
                slug: "content-os",
                name: "Content {OS}",
                tagline: "Editorial strategy, packaging, publishing and content analytics.",
                group: OsGroup::Grow,
                commands: &[
                    "Claude / Codex ( /content ):",
                    "  /content           open Content OS (strategy mode)",
                    "  /content-gps       define positioning and content system",
                    "  /capture-day       ingest the day as source material",
                    "  /story-mine        find stories, insights and proof",
                    "  /pillar            create a pillar asset",
                    "  /cascade           build a multi-platform waterfall",
                    "  /instagram         Instagram-native package",
                    "  /tiktok            TikTok-native package",
                    "  /youtube           YouTube package",
                    "  /linkedin          LinkedIn package",
                    "  /x                 X package",
                    "  /newsletter        newsletter edition",
                    "  /article           create an article",
                    "  /visual-brief      image/design direction",
                    "  /video-brief       script, shots and edit plan",
                    "  /sound-brief       sound/music/voice direction",
                    "  /content-calendar  build the editorial calendar",
                    "  /content-review    run the performance council",
                    "38 specialist agents · 44 skills · 12 protocols · 10 schemas",
                    "reference runtime (stdlib Python, provider-neutral, not an LLM adapter):",
                    "  runtime/os_runtime.py validate   integrity-check the pack (MANIFEST sha256)",
                    "  runtime/os_runtime.py route /content   resolve a command to its mode",
                    "  runtime/os_runtime.py event <kind> <json>   append a provenance event",
                    "→ Communication Stack: turns life, expertise, products and proof into native multi-platform content",
                ],
            },
            OsProduct {
                num: 35,
                slug: "sales-os",
                name: "Sales {OS}",
                tagline: "Pipeline, conversations and the close, without manipulation.",
                group: OsGroup::Grow,
                commands: &[],
            },
            OsProduct {
                num: 36,
                slug: "affiliate-os",
                name: "Affiliate {OS}",
                tagline: "Learn distribution by selling someone else's real product.",
                group: OsGroup::Grow,
                commands: &[],
            },
            OsProduct {
                num: 37,
                slug: "network-os",
                name: "Network {OS}",
                tagline: "Trusted relationship memory and network stewardship.",
                group: OsGroup::Grow,
                commands: &[
                    "Claude / Codex ( /network · /relationship-network-os ):",
                    "  ethical relationship steward, connector and gathering architect",
                    "  /network                open relationship overview (audit mode)",
                    "  /person                 prepare a person brief",
                    "  /meeting-prep           prepare for a meeting",
                    "  /interaction            capture an interaction and its commitments",
                    "  /follow-up              draft a relevant, loop-closing follow-up",
                    "  /intro                  create a consent-based introduction",
                    "  /nurture                design a relationship rhythm (cadence)",
                    "  /difficult-conversation prepare a truthful conversation",
                    "  /boundary               set or reinforce a boundary",
                    "  /gathering              design a meaningful gathering",
                    "  12 agents · 18 skills · 7 protocols · 6 schemas",
                    "reference runtime (stdlib Python, not a production DB/LLM):",
                    "  python runtime/os_runtime.py info|validate|route|event",
                ],
            },
            OsProduct {
                num: 38,
                slug: "growth-os",
                name: "Growth {OS}",
                tagline: "Loops, experiments and the channels that compound.",
                group: OsGroup::Grow,
                commands: &[],
            },
            OsProduct {
                num: 39,
                slug: "revenue-os",
                name: "Revenue {OS}",
                tagline: "Business cash flow, CRM, billing and receivables.",
                group: OsGroup::Grow,
                commands: &[
                    "Claude / Codex ( /revenue · /revenue-os ):",
                    "  conversational revenue brain + governed CRM/finance database",
                    "  /revenue        dashboard: integrated revenue + cash overview",
                    "  /offer          create or audit an offer",
                    "  /positioning    define category and differentiation",
                    "  /pricing        build a pricing architecture",
                    "  /pipeline       review CRM and forecast",
                    "  /lead           create or analyze a lead",
                    "  /sales-call     prepare or debrief a call",
                    "  /proposal       build proposal + commercial logic",
                    "  /invoice        create or inspect an invoice",
                    "  /collections    manage overdue receivables",
                    "  /business-cashflow   analyze business cash flow",
                    "  /receipt-business    stage a business receipt/photo",
                    "  /contract       stage contract data",
                    "  /revenue-close  run the monthly commercial/financial close",
                    "  /revenue-scenario    model revenue/cash scenarios",
                    "  /renewal        plan a renewal or expansion",
                    "  24 specialist agents · 40 skills · 10 protocols · 14 schemas",
                    "reference runtime (stdlib python, self-describing):",
                    "  runtime/os_runtime.py info · route \"/revenue\" · validate",
                    "  runtime/bootstrap_revenue_db.py   seed the reference records",
                    "→ handoffs: Market Research (WTP evidence), Delivery & Customer",
                    "            Success (signed scope), Wealth & Capital (only verified",
                    "            owner distributions cross the business/personal line)",
                ],
            },
            OsProduct {
                num: 40,
                slug: "delivery-cs-os",
                name: "Delivery & Customer Success {OS}",
                tagline: "Fulfil the promise, drive adoption, earn the renewal.",
                group: OsGroup::Grow,
                commands: &[
                    "Claude / Codex ( /delivery ):",
                    "  /delivery         open the delivery portfolio (review)",
                    "  /handoff-client   run the sales-to-delivery transfer",
                    "  /onboard-client   create the onboarding plan (access, roles, kickoff)",
                    "  /success-plan     define outcomes and measures with the customer",
                    "  /client-plan      milestones and governance (RACI)",
                    "  /client-update    draft a transparent status update",
                    "  /scope-change     process a change request (protect scope + margin)",
                    "  /client-risk      create an issue/escalation plan",
                    "  /adoption         build an adoption intervention",
                    "  /value-proof      compile outcome evidence (attribution honest)",
                    "  /qbr              prepare the business / value review",
                    "  /renew-client     prepare renewal or expansion",
                    "  /case-study       request and build a consented case study",
                    "  /offboard         close the engagement responsibly",
                    "  post-payment gate: contract.signed -> payment.reconciled -> handoff.accepted",
                    "  19 specialist agents · 30 skills · 9 protocols · 9 schemas",
                    "reference runtime (stdlib Python, provider-neutral, no LLM):",
                    "  runtime/os_runtime.py info | validate | route \"/delivery\" | event <kind> <json>",
                ],
            },
            // ── 05 · OPERATE — Turn chaos into repeatable execution ──────
            OsProduct {
                num: 41,
                slug: "execution-os",
                name: "Execution {OS}",
                tagline: "Time-bound personal commitments and proof of output.",
                group: OsGroup::Operate,
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
                num: 42,
                slug: "project-os",
                name: "Project {OS}",
                tagline: "Scope, plan and land a project without losing the thread.",
                group: OsGroup::Operate,
                commands: &[],
            },
            OsProduct {
                num: 43,
                slug: "meeting-os",
                name: "Meeting {OS}",
                tagline: "Meetings that produce decisions and owners, or do not happen.",
                group: OsGroup::Operate,
                commands: &[],
            },
            OsProduct {
                num: 44,
                slug: "documentation-os",
                name: "Documentation {OS}",
                tagline: "Write it once, find it later, keep it true.",
                group: OsGroup::Operate,
                commands: &[],
            },
            OsProduct {
                num: 45,
                slug: "client-os",
                name: "Client {OS}",
                tagline: "The client relationship: expectations, comms and boundaries.",
                group: OsGroup::Operate,
                commands: &[],
            },
            OsProduct {
                num: 46,
                slug: "operations-os",
                name: "Operations {OS}",
                tagline: "Process diagnosis and work simplification before automation.",
                group: OsGroup::Operate,
                commands: &[
                    "Claude / Codex ( /operations · /operations-automation-os ):",
                    "  challenge the system before adding tools, automate only suitable work",
                    "  /operations           open the operations diagnostic (default mode)",
                    "  /process-interview    interview process owners and users with examples",
                    "  /process-map          map the current state",
                    "  /value-stream         analyze flow and waste",
                    "  /simplify             remove and simplify work before any tool",
                    "  /automation-audit     find and score automation candidates",
                    "  /automate             create a production-ready automation blueprint",
                    "  /agent-automation     assess an AI-agent workflow (suitability + contract)",
                    "  /future-state         design the target operating model",
                    "  /runbook              create the operating runbook",
                    "  /automation-review    audit live automations and operational health",
                    "  /automation-incident  contain and recover a failed automation",
                    "  every automation carries inputs, exceptions, approvals, evidence,",
                    "  observability and a human recovery path",
                    "  24 specialist agents · 39 skills · 9 protocols · 9 schemas",
                    "runtime/os_runtime.py (reference CLI, stdlib): info · validate · route · event",
                    "→ Builder builds, Quality gates, Review & Governance authorizes risk/policy change",
                ],
            },
            OsProduct {
                num: 47,
                slug: "process-sop-os",
                name: "Process & SOP {OS}",
                tagline: "Turn a thing you do well into a thing anyone can do.",
                group: OsGroup::Operate,
                commands: &[],
            },
            OsProduct {
                num: 48,
                slug: "team-delegation-os",
                name: "Team & Delegation {OS}",
                tagline: "Hand work off so it comes back right the first time.",
                group: OsGroup::Operate,
                commands: &[],
            },
            OsProduct {
                num: 49,
                slug: "kpi-analytics-os",
                name: "KPI & Analytics {OS}",
                tagline: "Measure the few numbers that actually change decisions.",
                group: OsGroup::Operate,
                commands: &[],
            },
            OsProduct {
                num: 50,
                slug: "review-governance-os",
                name: "Review & Governance {OS}",
                tagline: "Cross-OS learning and approval of consequential change.",
                group: OsGroup::Operate,
                commands: &[
                    "Claude / Codex ( /review · /review-governance-os ):",
                    "  /review            open review (the default command)",
                    "  /daily-review      run daily reflection",
                    "  /weekly-review     run weekly operating review",
                    "  /monthly-review    run monthly metrics review",
                    "  /quarterly-review  run strategic governance",
                    "  /postmortem        analyze an incident or failure (blameless)",
                    "  /policy            create or audit a policy",
                    "  /change-request    submit a consequential change (approval authority)",
                    "  /risk-register     review risks, controls and residual exposure",
                    "  /ai-governance     apply AI risk governance (Govern/Map/Measure/Manage)",
                    "  13 agents · 20 skills · 7 protocols · 7 schemas",
                    "reference runtime (provider-neutral, stdlib only):",
                    "  python runtime/os_runtime.py info       name, version, purpose",
                    "  python runtime/os_runtime.py validate   sha256 integrity check of the pack",
                    "  python runtime/os_runtime.py route /review   resolve a command to its mode",
                    "  python runtime/os_runtime.py event <kind> <json>   append-only event log",
                    "→ governs consequential change across every OS · closes the learning loop with Context & Memory OS",
                ],
            },
            // ── 06 · OWN — Turn work into assets ───────────
            OsProduct {
                num: 51,
                slug: "money-os",
                name: "Money {OS}",
                tagline: "Personal cash flow: what comes in, what goes out, what is left.",
                group: OsGroup::Own,
                commands: &[],
            },
            OsProduct {
                num: 52,
                slug: "wealth-os",
                name: "Wealth {OS}",
                tagline: "Personal net worth, reserves and long-horizon goals.",
                group: OsGroup::Own,
                commands: &[
                    "Claude / Codex ( /wealth · /wealth-capital-os ):",
                    "  a conversational personal CFO, behavior coach and capital planner",
                    "  /wealth          open the personal CFO dashboard",
                    "  /cashflow        analyze personal cash flow (inflows, outflows, margin)",
                    "  /money-close     reconcile and close the month",
                    "  /saving          create or revise a savings plan",
                    "  /emergency-fund  size and fund the resilience reserve",
                    "  /debt            choose a debt payoff strategy",
                    "  /invest-policy   write an investment policy statement",
                    "  /purchase        evaluate a major purchase",
                    "  /money-scenario  model a what-if financial scenario",
                    "  /receipt         stage a document or receipt (verified before it posts)",
                    "  labels every claim E1..E5 · owns personal money, Revenue OS owns the business",
                    "  12 agents · 20 skills · 7 protocols · 7 schemas",
                    "reference runtime (stdlib Python, self-describe + integrity only):",
                    "  python runtime/os_runtime.py   info · route · event · validate",
                ],
            },
            OsProduct {
                num: 53,
                slug: "ownership-os",
                name: "Ownership {OS}",
                tagline: "What you own, through which entity, and on what terms.",
                group: OsGroup::Own,
                commands: &[],
            },
            OsProduct {
                num: 54,
                slug: "ip-asset-os",
                name: "IP & Asset {OS}",
                tagline: "Intellectual property and durable assets: create, protect, license.",
                group: OsGroup::Own,
                commands: &[],
            },
            OsProduct {
                num: 55,
                slug: "business-strategy-os",
                name: "Business Strategy {OS}",
                tagline: "The strategy of the business as an asset, not as a job.",
                group: OsGroup::Own,
                commands: &[],
            },
            OsProduct {
                num: 56,
                slug: "exit-liquidity-os",
                name: "Exit & Liquidity {OS}",
                tagline: "Prepare, time and run a liquidity event.",
                group: OsGroup::Own,
                commands: &[],
            },
            // ── 07 · CAPITAL — Turn assets into capital allocation ───────
            OsProduct {
                num: 57,
                slug: "capital-os",
                name: "Capital {OS}",
                tagline: "Allocate capital deliberately across a portfolio of bets.",
                group: OsGroup::Capital,
                commands: &[],
            },
            OsProduct {
                num: 58,
                slug: "investment-thesis-os",
                name: "Investment Thesis {OS}",
                tagline: "Write the thesis before the cheque, and test it after.",
                group: OsGroup::Capital,
                commands: &[],
            },
            OsProduct {
                num: 59,
                slug: "deal-flow-os",
                name: "Deal Flow {OS}",
                tagline: "Source, filter and track opportunities at the top of the funnel.",
                group: OsGroup::Capital,
                commands: &[],
            },
            OsProduct {
                num: 60,
                slug: "due-diligence-os",
                name: "Due Diligence {OS}",
                tagline: "Verify the story before you are committed to it.",
                group: OsGroup::Capital,
                commands: &[],
            },
            OsProduct {
                num: 61,
                slug: "acquisition-os",
                name: "Acquisition {OS}",
                tagline: "Buy a business: search, approach, negotiate, close.",
                group: OsGroup::Capital,
                commands: &[],
            },
            OsProduct {
                num: 62,
                slug: "deal-structuring-os",
                name: "Deal Structuring {OS}",
                tagline: "Terms, instruments and incentives that survive contact with reality.",
                group: OsGroup::Capital,
                commands: &[],
            },
            OsProduct {
                num: 63,
                slug: "portfolio-management-os",
                name: "Portfolio Management {OS}",
                tagline: "Run the portfolio after the deal: reporting, support, reallocation.",
                group: OsGroup::Capital,
                commands: &[],
            },
            OsProduct {
                num: 64,
                slug: "board-os",
                name: "Board {OS}",
                tagline: "Governance at the board level: papers, cadence, real oversight.",
                group: OsGroup::Capital,
                commands: &[],
            },
            // ── 08 · AI & SYSTEMS — Intelligence infrastructure for everything ────
            OsProduct {
                num: 65,
                slug: "ai-logic-os",
                name: "AI Logic {OS}",
                tagline: "When to use deterministic code and when to use model judgment.",
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
                num: 66,
                slug: "context-memory-os",
                name: "Context & Memory {OS}",
                tagline: "The canonical shared context and persistence layer for every OS.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /memory · /context-memory-os ):",
                    "  one trustworthy, inspectable, permissioned memory layer for every OS",
                    "  separates fact from inference and temporary state, no cross-project bleed",
                    "modes (say the word or the /command):",
                    "  /memory          search or inspect authorized memory",
                    "  /remember        propose a memory write (staged with provenance)",
                    "  /ingest          ingest a file or event (screened for injection)",
                    "  /context         compile a purpose-specific context pack",
                    "  /snapshot        create a versioned project/person snapshot",
                    "  /decision-log    record a decision and its rationale",
                    "  /contradiction   resolve conflicting records",
                    "  /memory-audit    audit provenance and access",
                    "  /forget          delete or archive authorized memory",
                    "  /export-memory   create a user-readable export",
                    "  14 agents · 20 skills · 7 protocols · 8 schemas",
                    "reference runtime (python runtime/os_runtime.py, stdlib only):",
                    "  info · route · event · validate   self-describing, integrity-checkable",
                    "→ the canonical shared context layer the other OSes recover from",
                ],
            },
            OsProduct {
                num: 67,
                slug: "agent-os",
                name: "Agent {OS}",
                tagline: "Design, brief and supervise agents that do real work.",
                group: OsGroup::Systems,
                commands: &[],
            },
            OsProduct {
                num: 68,
                slug: "automation-os",
                name: "Automation {OS}",
                tagline: "Governed automation of a process that was simplified first.",
                group: OsGroup::Systems,
                commands: &[],
            },
            OsProduct {
                num: 69,
                slug: "knowledge-os",
                name: "Knowledge {OS}",
                tagline: "Turn scattered information into a retrievable knowledge base.",
                group: OsGroup::Systems,
                commands: &[],
            },
            OsProduct {
                num: 70,
                slug: "evaluation-os",
                name: "Evaluation {OS}",
                tagline: "Measure AI output quality with rubrics, not vibes.",
                group: OsGroup::Systems,
                commands: &[],
            },
            OsProduct {
                num: 71,
                slug: "tool-integration-os",
                name: "Tool & Integration {OS}",
                tagline: "Connect external tools safely, with typed contracts.",
                group: OsGroup::Systems,
                commands: &[],
            },
            OsProduct {
                num: 72,
                slug: "orchestration-os",
                name: "Orchestration {OS}",
                tagline: "Compose many agents and systems into one reliable mission.",
                group: OsGroup::Systems,
                commands: &[],
            },
        ]
    }

    /// "01"..."08" for BUILD OSes (their pipeline position), None for
    /// every other group. Derived from registry order, never hand-numbered.
    pub fn chain_position(&self) -> Option<usize> {
        if self.group != OsGroup::Build {
            return None;
        }
        Self::all()
            .iter()
            .filter(|p| p.group == OsGroup::Build)
            .position(|p| p.slug == self.slug)
            .map(|i| i + 1)
    }
}
/// Coarse stage derived from concrete local surfaces. The strongest stage is
/// deliberately `Testable`, not `Verified`: finding test files does not prove
/// that anybody ran them successfully against this revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsReadinessLevel {
    /// Directory missing or empty scaffold.
    Scaffold,
    /// Prompts/docs/packaging exist, but no local runtime entrypoint was found.
    Reference,
    /// A runtime surface exists, but no test surface was found.
    Runnable,
    /// Runtime and test surfaces both exist; tests have not been executed here.
    Testable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsManifestStatus {
    Missing,
    Invalid,
    Valid,
}

/// Evidence behind the readiness label. These fields let the TUI state what it
/// actually observed instead of compressing any arbitrary extra file into a
/// green "integrated" badge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OsReadiness {
    pub level: OsReadinessLevel,
    pub directory_present: bool,
    pub master_present: bool,
    pub payload_present: bool,
    pub manifest: OsManifestStatus,
    pub runtime_present: bool,
    pub tests_present: bool,
    /// Raw `events.schema_status` from MANIFEST.json when present. Values such
    /// as `stub` remain visible and never imply runtime verification.
    pub event_schema_status: Option<String>,
}

impl OsReadiness {
    fn missing() -> Self {
        Self {
            level: OsReadinessLevel::Scaffold,
            directory_present: false,
            master_present: false,
            payload_present: false,
            manifest: OsManifestStatus::Missing,
            runtime_present: false,
            tests_present: false,
            event_schema_status: None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.level {
            OsReadinessLevel::Scaffold if !self.directory_present => "directory missing",
            OsReadinessLevel::Scaffold => "scaffold only",
            OsReadinessLevel::Reference => "reference surface only",
            OsReadinessLevel::Runnable => "runtime present (tests not found)",
            OsReadinessLevel::Testable => "runtime + tests present (not executed)",
        }
    }

    pub fn manifest_label(&self) -> &'static str {
        match self.manifest {
            OsManifestStatus::Missing => "missing",
            OsManifestStatus::Invalid => "invalid JSON",
            OsManifestStatus::Valid => "valid JSON",
        }
    }
}

/// One row for the OS tab: identity + where it lives here + integration state.
#[derive(Debug, Clone)]
pub struct OsEntry {
    pub product: OsProduct,
    pub readiness: OsReadiness,
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

/// Inspect bounded local filesystem evidence. Symlinked directories are not
/// followed, and the walk is capped, so entering the OS tab cannot recurse
/// forever through a malformed payload.
fn dir_readiness(dir: &Path) -> OsReadiness {
    if !dir.is_dir() {
        return OsReadiness::missing();
    }

    let manifest_path = dir.join("MANIFEST.json");
    let (manifest, event_schema_status) = if manifest_path.is_file() {
        match std::fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        {
            Some(value) => (
                OsManifestStatus::Valid,
                value
                    .pointer("/events/schema_status")
                    .and_then(|value| value.as_str())
                    .map(str::to_string),
            ),
            None => (OsManifestStatus::Invalid, None),
        }
    } else {
        (OsManifestStatus::Missing, None)
    };

    let master_present = dir.join("MASTER.md").is_file();
    let mut payload_present = false;
    let mut runtime_present = false;
    let mut tests_present = false;
    let mut pending = vec![(dir.to_path_buf(), 0usize)];
    let mut visited = 0usize;

    while let Some((current, depth)) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            visited += 1;
            if visited > 4096 {
                break;
            }
            let path = entry.path();
            let Ok(relative) = path.strip_prefix(dir) else {
                continue;
            };
            let components: Vec<String> = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy().to_lowercase())
                .collect();
            let Some(name) = components.last().map(String::as_str) else {
                continue;
            };
            let top = components.first().map(String::as_str).unwrap_or(name);
            let is_scaffold = depth == 0 && matches!(name, "readme.md" | "master.md" | "ledger")
                || name.starts_with('.');
            if !is_scaffold {
                payload_present = true;
            }

            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };
            let in_tests = components
                .iter()
                .any(|part| part == "tests" || part == "test");
            let test_name = name.starts_with("test_")
                || name.contains(".test.")
                || name.contains("_test.")
                || name.ends_with("_spec.rs");
            if in_tests || test_name {
                tests_present = true;
            }

            let runtime_dir = matches!(top, "bin" | "scripts" | "src" | "runtime" | "app" | "cli");
            let root_entrypoint = depth == 0
                && matches!(
                    name,
                    "main.rs"
                        | "main.py"
                        | "app.py"
                        | "index.ts"
                        | "index.js"
                        | "package.json"
                        | "cargo.toml"
                );
            if runtime_dir || root_entrypoint {
                runtime_present = true;
            }

            if file_type.is_dir() && depth < 4 {
                pending.push((path, depth + 1));
            }
        }
        if visited > 4096 {
            break;
        }
    }

    let level = if runtime_present && tests_present {
        OsReadinessLevel::Testable
    } else if runtime_present {
        OsReadinessLevel::Runnable
    } else if master_present || payload_present || dir.join("README.md").is_file() {
        OsReadinessLevel::Reference
    } else {
        OsReadinessLevel::Scaffold
    };

    OsReadiness {
        level,
        directory_present: true,
        master_present,
        payload_present,
        manifest,
        runtime_present,
        tests_present,
        event_schema_status,
    }
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
        .and_then(|v| v.as_object().map(|map| map.keys().cloned().collect()))
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
            let readiness = path
                .as_deref()
                .map(dir_readiness)
                .unwrap_or_else(OsReadiness::missing);
            OsEntry {
                product: *p,
                readiness,
                path,
                bot_linked: bots.contains(&format!("os-{}", p.slug)),
            }
        })
        .collect()
}

impl OsEntry {
    /// Readiness glyph: absence/scaffold, reference, runnable, then testable.
    pub fn glyph(&self) -> &'static str {
        match self.readiness.level {
            OsReadinessLevel::Scaffold => "⚪",
            OsReadinessLevel::Reference => "🔵",
            OsReadinessLevel::Runnable => "🟡",
            OsReadinessLevel::Testable => "🧪",
        }
    }

    pub fn status_label(&self) -> &'static str {
        self.readiness.label()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The AGENTIK {OS} suite size. The roster is GENERATED from
    /// `OS/_registry.json`, so these tests assert the invariants that must hold
    /// for ANY registry rather than a hand-copied list of slugs, which is what
    /// used to drift the moment an OS was added.
    const SUITE_SIZE: usize = 73;

    #[test]
    fn suite_has_every_unit_with_a_unique_slug_and_contiguous_numbering() {
        let all = OsProduct::all();
        assert_eq!(all.len(), SUITE_SIZE, "the suite is {SUITE_SIZE} units");

        let mut slugs: Vec<&str> = all.iter().map(|p| p.slug).collect();
        slugs.sort_unstable();
        let before = slugs.len();
        slugs.dedup();
        assert_eq!(before, slugs.len(), "every slug is unique");

        // `num` is the operator's canonical tree number: 0..71, in order.
        let nums: Vec<u8> = all.iter().map(|p| p.num).collect();
        let want: Vec<u8> = (0..SUITE_SIZE as u8).collect();
        assert_eq!(nums, want, "numbers are contiguous 0..{} in order", SUITE_SIZE - 1);
    }

    #[test]
    fn every_unit_carries_a_name_and_a_one_line_tagline() {
        for p in OsProduct::all() {
            assert!(!p.name.is_empty(), "{} has no name", p.slug);
            assert!(!p.tagline.is_empty(), "{} has no tagline", p.slug);
            assert!(
                p.tagline.ends_with('.'),
                "{} tagline must be one sentence ending in a period",
                p.slug
            );
            // R-NODASH: no em or en dash anywhere in the product surface.
            for bad in ['\u{2014}', '\u{2013}'] {
                assert!(!p.name.contains(bad), "{} name has a long dash", p.slug);
                assert!(!p.tagline.contains(bad), "{} tagline has a long dash", p.slug);
            }
        }
    }

    #[test]
    fn every_group_is_populated_and_labelled() {
        for g in OsGroup::all() {
            assert!(!g.label().is_empty(), "{g:?} has no label");
            assert!(!g.purpose().is_empty(), "{g:?} has no purpose");
            let n = OsProduct::all().iter().filter(|p| p.group == *g).count();
            assert!(n > 0, "group {g:?} has no members");
        }
    }

    #[test]
    fn groups_are_contiguous_in_declaration_order() {
        // Each group appears exactly once as a contiguous run, in the order
        // `OsGroup::all()` declares (what the TUI renders, top to bottom).
        let order: Vec<OsGroup> = OsGroup::all().to_vec();
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
    fn build_chain_positions_are_contiguous_and_unique() {
        let chain: Vec<&OsProduct> = OsProduct::all()
            .iter()
            .filter(|p| p.group == OsGroup::Build)
            .collect();
        let positions: Vec<usize> = chain
            .iter()
            .map(|p| p.chain_position().expect("a BUILD OS has a position"))
            .collect();
        assert_eq!(
            positions,
            (1..=chain.len()).collect::<Vec<_>>(),
            "BUILD positions are 1..={} with no gaps",
            chain.len()
        );
        // Every OS outside the BUILD group returns None.
        for p in OsProduct::all() {
            if p.group != OsGroup::Build {
                assert_eq!(p.chain_position(), None, "{} must have no position", p.slug);
            }
        }
        // Named anchors of the chain, per the AGENTIK {OS} tree (20..27).
        let pos = |slug: &str| {
            OsProduct::all()
                .iter()
                .find(|p| p.slug == slug)
                .unwrap_or_else(|| panic!("{slug} is in the suite"))
                .chain_position()
        };
        assert_eq!(pos("blueprint-os"), Some(1), "Blueprint opens the BUILD chain");
        assert_eq!(pos("builder-os"), Some(5));
        assert_eq!(pos("release-os"), Some(8), "Release closes the BUILD chain");
        assert_eq!(pos("librarian-os"), None);
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
    fn readiness_distinguishes_reference_runtime_and_test_surfaces() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        let empty = dir_readiness(dir);
        assert_eq!(empty.level, OsReadinessLevel::Scaffold);
        assert!(!empty.payload_present);

        std::fs::write(dir.join("README.md"), "# reference").unwrap();
        std::fs::write(dir.join("MASTER.md"), "# master prompt").unwrap();
        let reference = dir_readiness(dir);
        assert_eq!(reference.level, OsReadinessLevel::Reference);
        assert!(reference.master_present);

        // An arbitrary extra document is payload, but it is not a runtime.
        std::fs::write(dir.join("notes.txt"), "not executable").unwrap();
        let documented = dir_readiness(dir);
        assert_eq!(documented.level, OsReadinessLevel::Reference);
        assert!(documented.payload_present);
        assert!(!documented.runtime_present);

        std::fs::create_dir_all(dir.join("scripts")).unwrap();
        std::fs::write(dir.join("scripts/run.py"), "print('run')").unwrap();
        let runnable = dir_readiness(dir);
        assert_eq!(runnable.level, OsReadinessLevel::Runnable);
        assert!(runnable.runtime_present);
        assert!(!runnable.tests_present);

        std::fs::write(dir.join("scripts/test_os.py"), "assert True").unwrap();
        let testable = dir_readiness(dir);
        assert_eq!(testable.level, OsReadinessLevel::Testable);
        assert!(testable.tests_present);
        assert!(testable.label().contains("not executed"));
    }

    #[test]
    fn readiness_reports_manifest_validity_and_stub_schema_without_promoting_it() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("MASTER.md"), "# prompt").unwrap();
        std::fs::write(dir.join("MANIFEST.json"), "not json").unwrap();
        let invalid = dir_readiness(dir);
        assert_eq!(invalid.manifest, OsManifestStatus::Invalid);
        assert_eq!(invalid.level, OsReadinessLevel::Reference);

        std::fs::write(
            dir.join("MANIFEST.json"),
            r#"{"events":{"schema_status":"stub"}}"#,
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("runtime")).unwrap();
        std::fs::write(dir.join("runtime/main.rs"), "fn main() {}").unwrap();
        let stub = dir_readiness(dir);
        assert_eq!(stub.manifest, OsManifestStatus::Valid);
        assert_eq!(stub.event_schema_status.as_deref(), Some("stub"));
        assert_eq!(stub.level, OsReadinessLevel::Runnable);
        assert!(!stub.tests_present);
    }

    #[test]
    fn missing_directory_is_not_reported_as_an_awaiting_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = dir_readiness(&tmp.path().join("not-created"));
        assert_eq!(missing.level, OsReadinessLevel::Scaffold);
        assert!(!missing.directory_present);
        assert_eq!(missing.label(), "directory missing");
    }

    #[test]
    fn entries_cover_the_whole_suite() {
        let entries = list_os_entries();
        assert_eq!(entries.len(), OsProduct::all().len());
    }
}

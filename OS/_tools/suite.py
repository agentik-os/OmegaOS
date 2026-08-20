#!/usr/bin/env python3
"""AGENTIK {OS} — the canonical 73-unit suite registry.

THE SINGLE SOURCE OF TRUTH for the whole suite. Everything derives from the
SUITE tuple below and nothing is hand-maintained twice:

    OS/_registry.json              machine-readable registry  (emit)
    crates/omega-core/src/os_products.rs   the TUI OS-menu roster (emit)
    OS/README.md                   the human index             (emit)
    OS/<slug>/                     the per-OS directory tree   (scaffold)

Add or reorder an OS HERE, then re-run. Never edit a generated file by hand.

Columns of SUITE:
  num      canonical number in the operator's tree (0..72)
  slug     directory name under OS/ and the id used everywhere
  name     display name (rendered as "<name> {OS}")
  group    category key, see GROUPS
  tagline  one line: what this OS is for
  maps     existing directory this OS inherits from, or None if net new

Usage:
    python3 suite.py check       validate the registry, print the summary
    python3 suite.py registry    write OS/_registry.json
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OS_DIR = os.path.dirname(HERE)
REPO = os.path.dirname(OS_DIR)

# ── the nine groups, in render order ──────────────────────────────────────
GROUPS = [
    ("runtime",  "00 · RUNTIME",           "Build and run the entire Agentik ecosystem",    "Runtime"),
    ("personal", "01 · PERSONAL",          "Operate yourself",                              "Personal"),
    ("discover", "02 · DISCOVER & DECIDE", "Find what is worth building",                   "Discover"),
    ("build",    "03 · BUILD",             "Turn evidence into products",                   "Build"),
    ("grow",     "04 · GROW",              "Turn value into distribution and revenue",      "Grow"),
    ("operate",  "05 · OPERATE",           "Turn chaos into repeatable execution",          "Operate"),
    ("own",      "06 · OWN",               "Turn work into assets",                         "Own"),
    ("capital",  "07 · CAPITAL",           "Turn assets into capital allocation",           "Capital"),
    ("systems",  "08 · AI & SYSTEMS",      "Intelligence infrastructure for everything",    "Systems"),
]

# ── the 73 operative systems ──────────────────────────────────────────────
# (num, slug, name, group, tagline, maps_from)
SUITE = [
    (0,  "os-builder-os",          "OS Builder",                "runtime",  "Build an operative system itself: intake, spec, research, build, red team, score, release.", None),
    (1,  "agentik-runtime",        "Agentik Runtime",           "runtime",  "Install, configure, run, compose, update and evaluate every Agentik OS.", None),

    (2,  "mindset-os",             "Mindset",                   "personal", "Identity and beliefs: the behavioural compiler under Alignment.", "mindset-os"),
    (3,  "identity-shift-os",      "Identity Shift",            "personal", "Deliberate identity change: who you must become before what you must do.", "identity-shift-os"),
    (4,  "alignment-os",           "Alignment",                 "personal", "Meaning, values and inner alignment: the BE authority of the suite.", "alignment-os"),
    (5,  "goal-life-strategy-os",  "Goal & Life Strategy",      "personal", "Life-level goals and the strategy that makes them reachable.", None),
    (6,  "habit-tracker-os",       "Habit Tracker",             "personal", "Recurring behaviour contracts and the evidence they actually happened.", "habit-tracker-os"),
    (7,  "health-energy-os",       "Health & Energy",           "personal", "Physical and cognitive capacity: sleep, movement, fuel, recovery, stress.", "health-energy-os"),
    (8,  "intuitive-os",           "Intuitive",                 "personal", "Train and calibrate intuition as a usable, falsifiable signal.", "intuitive-os"),
    (9,  "decision-os",            "Decision",                  "personal", "Make hard calls well: framing, reversibility, evidence, the decision record.", None),
    (10, "journal-os",             "Journal",                   "personal", "Reflection that compounds: capture, revisit, extract the pattern.", "journal-os"),
    (11, "social-intelligence-os", "Social Intelligence",       "personal", "Read rooms and people accurately, and act with integrity.", "seductive-os"),

    (12, "librarian-os",           "Librarian",                 "discover", "Your reading and source corpus turned into retrievable understanding.", "books-os"),
    (13, "research-os",            "Research",                  "discover", "General-purpose evidence gathering with sources you can defend.", "researcher-os"),
    (14, "trend-opportunity-os",   "Trend & Opportunity",       "discover", "Spot movement early and turn it into a named opportunity.", None),
    (15, "brainstorm-os",          "Brainstorm",                "discover", "Generate and evolve ideas before research or definition.", "brainstorm-os"),
    (16, "strategy-portfolio-os",  "Strategy & Portfolio",      "discover", "Choose the bets: goals, projects and resource allocation.", "strategy-portfolio-os"),
    (17, "market-research-os",     "Market Research",           "discover", "Market and customer evidence, and the validation that follows.", "market-research-os"),
    (18, "customer-discovery-os",  "Customer Discovery",        "discover", "Talk to real users and extract what they actually need.", None),
    (19, "validation-os",          "Validation",                "discover", "Kill or confirm an idea with the cheapest sufficient test.", None),
    (20, "business-model-os",      "Business Model",            "discover", "How value is created, delivered and captured, made explicit.", None),

    (21, "blueprint-os",           "Blueprint",                 "build",    "The product-definition compiler: a complete, traceable definition pack.", "blueprint-os"),
    (22, "design-os",              "Design",                    "build",    "UX, interaction and visual design compiled into a machine-readable handoff.", "design-os"),
    (23, "prototype-os",           "Prototype",                 "build",    "The cheapest artifact that answers the riskiest open question.", None),
    (24, "stepper-os",             "Stepper",                   "build",    "The dependency-aware step graph and its deterministic verification gate.", "stepper-os"),
    (25, "builder-os",             "Builder",                   "build",    "The implementation runtime: steps executed into release-ready code.", "builder-os"),
    (26, "quality-evaluation-os",  "Quality & Evaluation",      "build",    "Independent certification of what was built, before it ships.", "quality-evaluation-release-os"),
    (27, "security-os",            "Security",                  "build",    "Threat modelling, hardening and the security gate on a release.", None),
    (28, "release-os",             "Release",                   "build",    "Ship it: release boundaries, rollout, rollback and the incident path.", "quality-evaluation-release-os"),

    (29, "positioning-os",         "Positioning",               "grow",     "The category you compete in and the claim you own inside it.", None),
    (30, "brand-os",               "Brand",                     "grow",     "Identity, voice and the visual system that carries them.", None),
    (31, "storyteller-os",         "Storyteller",               "grow",     "Narrative truth, structure, voice and consent.", "storyteller-os"),
    (32, "offer-os",               "Offer",                     "grow",     "The thing you sell, shaped so the value is obvious.", None),
    (33, "pricing-os",             "Pricing",                   "grow",     "What to charge, how to package it and when to change it.", None),
    (34, "content-os",             "Content",                   "grow",     "Editorial strategy, packaging, publishing and content analytics.", "content-os"),
    (35, "sales-os",               "Sales",                     "grow",     "Pipeline, conversations and the close, without manipulation.", None),
    (36, "affiliate-os",           "Affiliate",                 "grow",     "Learn distribution by selling someone else's real product.", None),
    (37, "network-os",             "Network",                   "grow",     "Trusted relationship memory and network stewardship.", "relationship-network-os"),
    (38, "growth-os",              "Growth",                    "grow",     "Loops, experiments and the channels that compound.", None),
    (39, "revenue-os",             "Revenue",                   "grow",     "Business cash flow, CRM, billing and receivables.", "revenue-os"),
    (40, "delivery-cs-os",         "Delivery & Customer Success","grow",    "Fulfil the promise, drive adoption, earn the renewal.", "delivery-customer-success-os"),

    (41, "execution-os",           "Execution",                 "operate",  "Time-bound personal commitments and proof of output.", "execution-os"),
    (42, "project-os",             "Project",                   "operate",  "Scope, plan and land a project without losing the thread.", None),
    (43, "meeting-os",             "Meeting",                   "operate",  "Meetings that produce decisions and owners, or do not happen.", None),
    (44, "documentation-os",       "Documentation",             "operate",  "Write it once, find it later, keep it true.", None),
    (45, "client-os",              "Client",                    "operate",  "The client relationship: expectations, comms and boundaries.", None),
    (46, "operations-os",          "Operations",                "operate",  "Process diagnosis and work simplification before automation.", "operations-automation-os"),
    (47, "process-sop-os",         "Process & SOP",             "operate",  "Turn a thing you do well into a thing anyone can do.", None),
    (48, "team-delegation-os",     "Team & Delegation",         "operate",  "Hand work off so it comes back right the first time.", None),
    (49, "kpi-analytics-os",       "KPI & Analytics",           "operate",  "Measure the few numbers that actually change decisions.", None),
    (50, "review-governance-os",   "Review & Governance",       "operate",  "Cross-OS learning and approval of consequential change.", "review-governance-os"),

    (51, "money-os",               "Money",                     "own",      "Personal cash flow: what comes in, what goes out, what is left.", None),
    (52, "wealth-os",              "Wealth",                    "own",      "Personal net worth, reserves and long-horizon goals.", "wealth-capital-os"),
    (53, "ownership-os",           "Ownership",                 "own",      "What you own, through which entity, and on what terms.", None),
    (54, "ip-asset-os",            "IP & Asset",                "own",      "Intellectual property and durable assets: create, protect, license.", None),
    (55, "business-strategy-os",   "Business Strategy",         "own",      "The strategy of the business as an asset, not as a job.", None),
    (56, "exit-liquidity-os",      "Exit & Liquidity",          "own",      "Prepare, time and run a liquidity event.", None),

    (57, "capital-os",             "Capital",                   "capital",  "Allocate capital deliberately across a portfolio of bets.", "wealth-capital-os"),
    (58, "investment-thesis-os",   "Investment Thesis",         "capital",  "Write the thesis before the cheque, and test it after.", None),
    (59, "deal-flow-os",           "Deal Flow",                 "capital",  "Source, filter and track opportunities at the top of the funnel.", None),
    (60, "due-diligence-os",       "Due Diligence",             "capital",  "Verify the story before you are committed to it.", None),
    (61, "acquisition-os",         "Acquisition",               "capital",  "Buy a business: search, approach, negotiate, close.", None),
    (62, "deal-structuring-os",    "Deal Structuring",          "capital",  "Terms, instruments and incentives that survive contact with reality.", None),
    (63, "portfolio-management-os","Portfolio Management",      "capital",  "Run the portfolio after the deal: reporting, support, reallocation.", None),
    (64, "board-os",               "Board",                     "capital",  "Governance at the board level: papers, cadence, real oversight.", None),

    (65, "ai-logic-os",            "AI Logic",                  "systems",  "When to use deterministic code and when to use model judgment.", "ai-logic-os"),
    (66, "context-memory-os",      "Context & Memory",          "systems",  "The canonical shared context and persistence layer for every OS.", "context-memory-os"),
    (67, "agent-os",               "Agent",                     "systems",  "Design, brief and supervise agents that do real work.", None),
    (68, "automation-os",          "Automation",                "systems",  "Governed automation of a process that was simplified first.", "operations-automation-os"),
    (69, "knowledge-os",           "Knowledge",                 "systems",  "Turn scattered information into a retrievable knowledge base.", None),
    (70, "evaluation-os",          "Evaluation",                "systems",  "Measure AI output quality with rubrics, not vibes.", None),
    (71, "tool-integration-os",    "Tool & Integration",        "systems",  "Connect external tools safely, with typed contracts.", None),
    (72, "orchestration-os",       "Orchestration",             "systems",  "Compose many agents and systems into one reliable mission.", None),
]

# Directories that exist under OS/ but are duplicate aliases of a canonical
# unit. Recorded so the cleanup is explicit and never silently re-created.
ALIASES = {
    "designer-os":     "design-os",
    "habits-os":       "habit-tracker-os",
    "storytelling-os": "storyteller-os",
    "ideation-os":     "brainstorm-os",
    "researcher-os":   "research-os",
}

# The per-OS directory contract the operator specified.
FILES = ["README.md", "OS.md", "SYSTEM.md", "SKILL.md", "SETUP.md",
         "manifest.json", "CHANGELOG.md"]
DIRS = ["WORKFLOWS", "COMMANDS", "PROMPTS", "REFERENCES", "MEMORY",
        "TOOLS", "EVALS", "EXAMPLES", "INTERFACES", "ADAPTERS"]
INTERFACES = ["chat.md", "artifact.md", "dashboard.md", "generative-ui.md"]
ADAPTERS = ["chatgpt.md", "claude.md", "gemini.md", "codex.md"]

GROUP_KEYS = {g[0] for g in GROUPS}


def validate():
    """Fail loudly on any registry defect. Returns list of problems."""
    problems = []
    if len(SUITE) != 73:
        problems.append(f"SUITE has {len(SUITE)} units, expected 73")
    nums = [u[0] for u in SUITE]
    if nums != list(range(73)):
        problems.append("numbers are not a contiguous 0..71 range in order")
    slugs = [u[1] for u in SUITE]
    dupes = {s for s in slugs if slugs.count(s) > 1}
    if dupes:
        problems.append(f"duplicate slugs: {sorted(dupes)}")
    for num, slug, name, group, tagline, _maps in SUITE:
        if group not in GROUP_KEYS:
            problems.append(f"{slug}: unknown group {group!r}")
        if not tagline or not tagline.endswith("."):
            problems.append(f"{slug}: tagline must be one sentence ending in a period")
        for ch in ("—", "–"):          # R-NODASH
            if ch in tagline or ch in name:
                problems.append(f"{slug}: contains a long dash")
    # groups must be contiguous blocks, in GROUPS order
    order = [g[0] for g in GROUPS]
    seen, last = [], None
    for _n, _s, _nm, group, _t, _m in SUITE:
        if group != last:
            seen.append(group)
            last = group
    if seen != order:
        problems.append(f"groups are not contiguous in declaration order: {seen}")
    return problems


def registry():
    """The machine-readable registry every other surface derives from."""
    by_group = {g[0]: [] for g in GROUPS}
    for num, slug, name, group, tagline, maps in SUITE:
        by_group[group].append(slug)
    return {
        "schema_version": "2.0.0",
        "suite": "AGENTIK {OS}",
        "total": len(SUITE),
        "groups": [
            {"key": k, "label": label, "purpose": purpose,
             "rust_variant": variant, "members": by_group[k]}
            for k, label, purpose, variant in GROUPS
        ],
        "contract": {
            "files": FILES, "dirs": DIRS,
            "INTERFACES": INTERFACES, "ADAPTERS": ADAPTERS,
        },
        "aliases": ALIASES,
        "os": [
            {
                "num": num, "slug": slug, "name": name,
                "display": f"{name} {{OS}}", "group": group,
                "tagline": tagline,
                "inherits_from": maps,
                "status": "inherit" if maps else "new",
            }
            for num, slug, name, group, tagline, maps in SUITE
        ],
    }


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    problems = validate()
    if problems:
        print("REGISTRY INVALID:")
        for p in problems:
            print("  -", p)
        return 1

    if cmd == "check":
        reg = registry()
        new = [o for o in reg["os"] if o["status"] == "new"]
        inh = [o for o in reg["os"] if o["status"] == "inherit"]
        print(f"registry OK: {reg['total']} units in {len(reg['groups'])} groups")
        for g in reg["groups"]:
            print(f"  {g['label']:<24} {len(g['members']):>2} units")
        print(f"\ninherit from an existing dir : {len(inh)}")
        print(f"net new to author            : {len(new)}")
        srcs = sorted({o['inherits_from'] for o in inh})
        splits = [s for s in srcs
                  if len([o for o in inh if o['inherits_from'] == s]) > 1]
        print(f"existing dirs reused         : {len(srcs)}")
        print(f"dirs that SPLIT into several : {splits}")
        print(f"duplicate aliases to retire  : {sorted(ALIASES)}")
        return 0

    if cmd == "registry":
        out = os.path.join(OS_DIR, "_registry.json")
        with open(out, "w", encoding="utf-8") as fh:
            json.dump(registry(), fh, indent=2, ensure_ascii=False)
            fh.write("\n")
        print(f"wrote {out}")
        return 0

    print(f"unknown command {cmd!r}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())

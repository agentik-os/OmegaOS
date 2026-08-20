# Brand {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install brand-os` | Installs this OS into your environment | Once, first |
| `agentik configure brand-os` | Collects the minimum context it needs | After install |
| `agentik run brand-os` | Starts the OS | Every session |
| `agentik doctor brand-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update brand-os` | Updates to the latest version | When a release lands |
| `agentik eval brand-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/brand` | Orients: reports system version, coverage and the next decision | nothing | current system state, open gaps, one next action |
| `/brand-core` | Establishes the identity core from the claim and the register | positioning statement, audience register | register, archetype, promise, each traced to its source |
| `/brand-name` | Names a company, product or feature and tests the name | a thing to name, its category | candidate names with claim, category and register verdicts |
| `/brand-voice` | Writes voice rules as accept and refuse pairs | the identity core, real drafts | rules, each with a sentence it accepts and one it refuses |
| `/brand-visual` | Builds the visual system | the identity core, surface constraints | type, colour, spacing, imagery direction, logo usage |
| `/brand-tokens` | Emits the visual system as machine readable tokens | the visual system | tokens with contrast and size constraints checked |
| `/brand-audit` | Audits an artifact rule by rule against the system | an artifact or a URL | pass or fail per rule, with the offending element cited |
| `/brand-asset` | Registers or releases an asset with usage rules | an asset, its intended surfaces | a library entry with allowed and forbidden surfaces |
| `/brand-system` | Renders the whole brand system as one versioned artifact | nothing | the system document with its version and change history |
| `/brand-evolve` | Versions a change and produces the migration list | the proposed change | the version diff and every surface carrying the old system |
| `/brand-handoff` | Packages the system for one downstream unit | a target OS slug | the subset that unit consumes and what it may not alter |
| `/brand-review` | Audits a sample of live surfaces for drift | a surface list or a sampling rule | a drift report ranked by how often the surface is seen |

### `/brand`

```
/brand
```

Reports where the system actually stands: which parts exist, which parts are
still taste rather than rules, which live surfaces failed their last audit, and
what decision is waiting on you.

**When to reach for it:** at the start of any brand session, and before any
release under the identity.
**Returns:** system version, coverage per part, open audit failures, one named
next action.

### `/brand-core`

```
/brand-core
```

Derives the identity core from the claim and the audience register. Every
element it proposes carries the line of the positioning statement or the
audience utterance it came from. Elements that trace to neither are cut, out
loud.

**When to reach for it:** once a positioning statement exists, and again after
any category change.
**Returns:** register, archetype, promise and naming conventions, each with its
trace, plus what was cut and why.

### `/brand-name`

```
/brand-name --for product --context "the onboarding module"
```

Generates and tests names against three adversaries: does it fit the claim,
does it read as belonging to the category, and does it survive the audience
register. Legal clearance is out of scope and is stated as such.

**When to reach for it:** when naming anything customers will say aloud.
**Returns:** candidates with a verdict per test, and the ones that failed with
the reason, so the same name is not proposed again next quarter.

### `/brand-voice`

```
/brand-voice
/brand-voice --from-drafts ./last-twenty-posts
```

Writes the voice rules. Each rule is a pair: a sentence the rule accepts and a
sentence it refuses. A rule that cannot refuse any real sentence is dropped
rather than kept as an aspiration.

**When to reach for it:** after the identity core, and whenever an audit keeps
failing on something no rule covers.
**Returns:** the rule set, each with its accept and refuse example, and the
list of proposed rules that were dropped for being unfalsifiable.

### `/brand-visual`

```
/brand-visual
```

Builds the visual system: type scale, colour ramps, spacing scale, imagery
direction and logo usage. It tests each decision against the hard environments,
small screens, print, one colour, and against the exclusion the position named.

**When to reach for it:** after the identity core, before any surface is built.
**Returns:** the system with a rationale per decision, and the environments each
decision was tested against.

### `/brand-tokens`

```
/brand-tokens --format css
/brand-tokens --format json
```

Emits the visual system as values a real surface consumes, with contrast ratios
and minimum sizes computed. A value that fails its accessibility constraint
fails here, and the nearest passing value is proposed.

**When to reach for it:** before handing anything to Design {OS}, and after
every visual change.
**Returns:** the token set, the constraint check per token, and the failures
with their proposed replacements.

### `/brand-audit`

```
/brand-audit ./landing-page.html
/brand-audit --text "paste a draft here"
```

Runs every rule against one artifact and names the offending element: the
sentence, the colour value, the crop, the type size. It does not return a score
out of ten, because a score hides which rule failed.

**When to reach for it:** before every release, and on any artifact somebody
claims is on brand.
**Returns:** pass or fail per rule with the element cited, and a verdict of on
system, off system, or off system with a recorded exception.

### `/brand-asset`

```
/brand-asset --register ./logo-mono.svg
/brand-asset --release --to partner --scope "one landing page"
```

Registers an asset with its usage rules, or prepares a release to a third
party. A release names the exact assets, the exact surfaces and the duration,
and it always requires a human decision.

**When to reach for it:** when an artifact becomes reusable, and any time a
partner, agency or affiliate asks for files.
**Returns:** the library entry with allowed and forbidden surfaces, or the
release package pending approval.

### `/brand-system`

```
/brand-system
```

Renders identity, naming, voice rules and the visual system as one versioned
artifact, with its change history. This is the thing you hand a new hire.

**When to reach for it:** on any version bump, and at onboarding.
**Returns:** the full system document with version, date and the diff since the
previous version.

### `/brand-evolve`

```
/brand-evolve --change "new type scale"
```

Versions a change and produces the migration list: every surface currently
carrying the old system. The version does not close until each listed surface
is corrected or explicitly waived by a human with a reason.

**When to reach for it:** whenever the system stops fitting the claim or the
audience, and never as a quiet edit.
**Returns:** the version diff, the migration list, and the open surfaces
blocking closure.

### `/brand-handoff`

```
/brand-handoff design-os
/brand-handoff content-os
```

Packages the subset one downstream unit consumes. Design gets tokens and
extension rules, Content gets voice rules and editorial surface guidance,
Storyteller gets the register constraints a story may not contradict.

**When to reach for it:** at every handoff, and after every version bump.
**Returns:** the unit specific package, and an explicit statement of what that
unit may extend versus what it must not reinterpret.

### `/brand-review`

```
/brand-review --sample live
```

Audits a sample of surfaces that are actually live, ranked by how many people
see them, and reports drift. Drift is normal; undetected drift is what turns a
system back into taste.

**When to reach for it:** on a cadence, and after any period of fast shipping.
**Returns:** the drift report by surface, the rules most often broken, and the
candidate rule changes if the system is losing on the same rule everywhere.

## Command summary

| Command | Does |
|---|---|
| `/brand` | where the system stands, and the next decision |
| `/brand-core` | the identity core, every element traced to its source |
| `/brand-name` | name a thing and test it against claim, category, register |
| `/brand-voice` | voice rules as accept and refuse pairs |
| `/brand-visual` | type, colour, spacing, imagery, logo usage |
| `/brand-tokens` | the visual system as values a surface consumes |
| `/brand-audit` | rule by rule verdict on one artifact, element cited |
| `/brand-asset` | register an asset, or release it to a third party |
| `/brand-system` | the whole system as one versioned artifact |
| `/brand-evolve` | version a change and migrate every affected surface |
| `/brand-handoff` | package the system for one downstream unit |
| `/brand-review` | sample live surfaces and report drift |

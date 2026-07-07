# Third-party skill collections (superpowers + gstack)

OmegaOS vendors two MIT-licensed third-party skill collections at pinned commit
SHAs. They install ALWAYS-ON during `./install.sh` (Phase 6.91) and are fully
reproducible: a fresh `git clone OmegaOS && ./install.sh` provisions the exact
same skills every time. The installer is additive by construction and never
regresses an existing skill or hook. Opt out with `OMEGA_SKIP_THIRD_PARTY=1`.

Installer script: `scripts/install-third-party-skills.sh`.

## 1. What ships

### obra/superpowers (14 process skills + SessionStart hook)

Fourteen skills that live at `~/.claude/skills/<name>` as symlinks into the
pinned clone. Their bare frontmatter names do not collide with any existing
OmegaOS skill:

1. `brainstorming`
2. `dispatching-parallel-agents`
3. `executing-plans`
4. `finishing-a-development-branch`
5. `receiving-code-review`
6. `requesting-code-review`
7. `subagent-driven-development`
8. `systematic-debugging`
9. `test-driven-development`
10. `using-git-worktrees`
11. `using-superpowers`
12. `verification-before-completion`
13. `writing-plans`
14. `writing-skills`

Plus a `SessionStart` hook (matcher `startup|clear|compact`) that runs
`hooks/session-start`, which cats `using-superpowers/SKILL.md` into the session
context. It emits the Claude-native `hookSpecificOutput.additionalContext`
shape only when `CLAUDE_PLUGIN_ROOT` is set and `COPILOT_CLI` is unset, so the
installer passes `CLAUDE_PLUGIN_ROOT` explicitly on the merged hook command.

### garrytan/gstack (54 gstack-* skills + /gstack front door + browse binary)

The `/gstack` router is the front door; the suite installs as real
`~/.claude/skills/gstack-<name>/` directories, each holding a `SKILL.md`
symlink into the clone. The `--prefix` flag namespaces every skill as
`gstack-*` (see the collision rationale in section 6). The skills:

`gstack` (router) plus: `autoplan`, `benchmark`, `benchmark-models`, `browse`,
`canary`, `careful`, `codex`, `context-restore`, `context-save`, `cso`,
`design-consultation`, `design-html`, `design-review`, `design-shotgun`,
`devex-review`, `diagram`, `document-generate`, `document-release`, `freeze`,
`gstack-upgrade`, `guard`, `health`, `investigate`, `ios-clean`,
`ios-design-review`, `ios-fix`, `ios-qa`, `ios-sync`, `land-and-deploy`,
`landing-report`, `learn`, `make-pdf`, `office-hours`, `open-gstack-browser`,
`pair-agent`, `plan-ceo-review`, `plan-design-review`, `plan-devex-review`,
`plan-eng-review`, `plan-tune`, `qa`, `qa-only`, `retro`, `review`, `scrape`,
`setup-browser-cookies`, `setup-deploy`, `setup-gbrain`, `ship`, `skillify`,
`spec`, `sync-gbrain`, `unfreeze`.

gstack also builds the `browse` bun binary (agentic local browser control) and,
best-effort, a Playwright Chromium and an emoji font, via its own `./setup`.

## 2. Where things land on the machine

| Thing | Path |
| --- | --- |
| superpowers clone (pinned) | `~/.omega/repos/superpowers` |
| superpowers skill symlinks | `~/.claude/skills/<name>` (14 links) |
| superpowers SessionStart hook | one entry in `~/.claude/settings.json` |
| gstack clone (pinned) | `~/.claude/skills/gstack` |
| gstack namespaced skills | `~/.claude/skills/gstack-<name>/` (54 dirs) |
| gstack browse binary | `~/.claude/skills/gstack/browse/dist/browse` |

The gstack clone MUST live exactly at `~/.claude/skills/gstack` because gstack
skills hardcode that path internally.

## 3. Pinned SHAs, versions, and why we pin

| Collection | Repo | Pin SHA | Version |
| --- | --- | --- | --- |
| superpowers | `github.com/obra/superpowers` | `d884ae04edebef577e82ff7c4e143debd0bbec99` | v6.1.1 |
| gstack | `github.com/garrytan/gstack` | `11de390be1be6849eb9a15f91ff4922dd16c589a` | v1.58.5.0 |

Pinning to a reviewed commit (not a moving branch tip) is what makes the install
reproducible (L0 install-parity) and safe: the pinned tree was safety-reviewed
CLEAN, so a compromised or drifted upstream tip cannot land on an operator box
without a deliberate, reviewed pin bump. Both pins are full 40-hex SHAs; the
verify-install gate enforces that.

## 4. Update procedure

1. Pick the new upstream commit you want and confirm it is clean.
2. Edit the two pin variables in `scripts/install-third-party-skills.sh`:
   `SUPERPOWERS_PIN` and/or `GSTACK_PIN`.
3. Re-run `bash scripts/install-third-party-skills.sh` (or `./install.sh`).
   `pin_clone` is idempotent: it fetches the new SHA and detaches onto it.
4. To test a candidate pin without editing the file, env-override it:
   `SUPERPOWERS_PIN=<sha> GSTACK_PIN=<sha> bash scripts/install-third-party-skills.sh`.

## 5. Opt-out and removal

Opt out of the whole step: set `OMEGA_SKIP_THIRD_PARTY=1` before `./install.sh`.

To remove after install:

```
# superpowers
rm -rf ~/.omega/repos/superpowers
for s in brainstorming dispatching-parallel-agents executing-plans \
  finishing-a-development-branch receiving-code-review requesting-code-review \
  subagent-driven-development systematic-debugging test-driven-development \
  using-git-worktrees using-superpowers verification-before-completion \
  writing-plans writing-skills; do rm -f ~/.claude/skills/$s; done
# then delete the superpowers SessionStart entry from ~/.claude/settings.json
# (the entry whose command contains superpowers/hooks/session-start)

# gstack
rm -rf ~/.claude/skills/gstack ~/.claude/skills/gstack-*
```

The superpowers symlinks point into `~/.omega/repos/superpowers`, not into
`~/.omega/skills`, so `omega sync` prune (which removes only dangling symlinks
into `~/.omega`) never touches them while the clone exists.

## 6. Design decisions

- **Pinned-clone vs vendor.** We clone at a pinned SHA rather than copying the
  files into the OmegaOS repo. This keeps OmegaOS lean, preserves the upstream
  git provenance, and makes a pin bump a one-line, auditable change.
- **`--prefix` collision rationale.** gstack in flat mode would create skills
  named `design` and `diagram`, which already exist in OmegaOS. `--prefix`
  namespaces every gstack skill as `gstack-*`, so there is zero collision with
  the existing library.
- **Additive hook merge.** The superpowers hook is merged into
  `~/.claude/settings.json` with the same jq dedupe pattern OmegaOS uses for its
  own hooks: it selects out any prior superpowers SessionStart entry, then
  appends exactly one. Every other key and hook is preserved byte-for-byte, and
  the merge is idempotent (re-running keeps exactly one entry).
- **companion-tools overlap self-skip.** `scripts/install-companion-tools.sh`
  has its own best-effort superpowers install that self-skips when
  `~/.claude/skills` already contains a brainstorm/superpower entry. Because this
  Phase 6.91 install creates those entries, that companion branch becomes a
  no-op. No edit to companion-tools is needed or made.

## 7. Doctrine reconciliation (R-CLI, R-BROWSER, R-TEST)

gstack ships `/gstack-browse`, a local agentic browser controller that, like
OmegaOS, favors a CLI over an MCP browser tool. This ALIGNS with R-CLI (prefer
CLI over MCP) and fits inside R-BROWSER:

- E2E of OUR OWN apps stays Playwright CLI (R-TEST, R-BROWSER known-steps path).
- Agentic browsing of unknown UIs has two options: gstack `browse` (local, on
  this machine) or `browser-use` (the paid cloud SDK, R-BROWSER unknown-UI
  path). gstack browse is the local, no-cloud-key option.

None of these change the existing browser doctrine; gstack browse is an
additional local tool inside the same decision map.

## 8. Telemetry

gstack telemetry defaults OFF (local JSONL only; see the DEFAULTS table in
`bin/gstack-config`, key `telemetry` defaulting to `off`). Nothing is sent off
the machine by the install or by default operation.

# OmegaOS

A terminal control plane for running a fleet of AI coding agents in parallel, where every agent obeys the same typed rulebook.

[English](README.md) | [Français](README.fr.md) | [Русский](README.ru.md) | [中文](README.zh.md)

[![CI](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml/badge.svg)](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml) ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg) ![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)

OmegaOS is not a library you import. You install it on a Linux box. You get the `omega` command, a TUI for watching and killing sessions, and an orchestration layer that hands work to agents. There's also a Telegram bridge if you want to drive it from your phone.

The default agent runtime is Claude Code. Lots of tools run agents in parallel. What's different here is that every agent, however deep in the tree, carries the same non-negotiable rules, injected as plain text into its prompt. That's the doctrine, and it's where to start.

Version 0.1.0. I run it daily; expect rough edges.

## The doctrine

There's a typed registry of 6 Laws and 20 Rules. It lives in Rust, at `crates/omega-core/src/rules.rs`, so it's a compiled artifact and not a YAML file someone forgot to update.

**Laws are inviolable.** They bind every agent and they override every rule and every task. There are six:

- **L0 — Ship the truth.** A change isn't done until a clean rebuild reproduces it and it's pushed. Anything less is a draft.
- **L1 — Runtime is the only truth.** Code and comments state intent. Only running it reveals reality. When they disagree, runtime wins.
- **L2 — Researcher, not sycophant.** Challenge a flawed premise with reasoning before you act. No fake confidence. "This should work" without evidence is a lie.
- **L3 — Decide and proceed.** A dispatched agent is autonomous. It never stops to ask "should I continue?" It decides, executes, and reports after.
- **L4 — Done means 100%, verified.** 92% is not done. Enumerate the tasks, finish each, verify each against runtime.
- **L5 — Quality over speed.** No streamlined, lightweight, or quick variant of a real protocol. A 403 or a 401 is an abort, not a pass.

**Rules are operational.** Twenty of them, sorted into Universal, QualityGate, Orchestration, Reporting, and Safety. Each Rule is scoped to the roles it binds: Master, Oracle, Worker. A worker doesn't get burdened with orchestration rules it can't act on, and an oracle doesn't carry the worker's file-locking discipline. Same registry, different slices.

### The funnel

This is the mechanism. One function, `rules::agent_context_block(scope)`, builds the role-scoped slice of Laws and Rules and injects it into the system prompt of every agent the moment it's dispatched.

A worker three levels down the tree carries the same six Laws as the Master at the top. Nobody can spawn a child that quietly drops L5 to go faster, because the child's prompt is assembled from the same registry by the same function.

Because the doctrine is just text, it works the same whether the backend is Claude, GPT, Gemini, or something you add later.

See the whole thing:

```
omega rules list
```

![omega rules list — the 6 Laws and 20 Rules, printed by OmegaOS](assets/omega-rules.svg)

## Architecture

Four levels, top to bottom.

**Level 1 — Human interface.** The TUI, the CLI (40+ commands), and the Telegram bridge all drive the same layer underneath.

**Level 2 — AISB Master.** A persistent agent that stays running, auto-restarts if it dies, and resumes its own conversation with `--continue`. It ships 13 agent templates named after Matrix characters (Oracle, Morpheus, Seraph, Keymaker, Smith, Niobe, Architect, Merovingian, Neo, Zion, Link, Construct, Pythia). The Master is a dispatcher. It only classifies and routes work to oracles.

**Level 3 — Oracle.** One per project. It classifies the request, plans, dispatches workers, and runs the quality gate at the end. An oracle orchestrates. It does not edit project code itself.

**Level 4 — Workers.** Ephemeral. They run in parallel, and each one is scoped to its own files by a file-lock claim: one writer per file, enforced with advisory file locks (fs2). The lock is real, not a convention. A worker signals completion by writing a `done.json` with status `done_clean`, `pending`, or `failed`; without that status it isn't done.

## How a mission runs

A request enters one of three ways: the TUI, the `omega` CLI, or the Telegram bridge. Wherever it starts, it lands on the AISB Master. The Master reads it, classifies it, and routes it to the oracle that owns the relevant project. It does not touch files. Its whole job is to decide where the work goes.

The oracle takes it from there. It owns exactly one project, and it plans the mission, splits it into tasks, and dispatches a worker per task. When the workers report back it runs a quality gate, then reports up the chain. What it never does is edit the project's code. The grader and the writer are different agents, so the grade isn't self-assessment. Because the grader didn't write the code, its grade is independent.

Workers are short-lived and run in parallel. Before a worker writes to a file it claims that file with an advisory lock (via `fs2`), so two workers physically cannot write the same file at the same time. This gives one writer per file, enforced by the lock rather than by convention. A worker does its task, checks the result against actual runtime, and writes a `done.json` with a status of `done_clean`, `pending`, or `failed`. The oracle reads that file, acknowledges, and closes the session. If the status isn't `done_clean`, the mission isn't done.

A worker doesn't have to chew through its subtasks one at a time. It can run a workflow in-process: spawn parallel sub-agents, check their outputs, and combine them into one answer. Code review uses this, as do research, audits, and design work. It's usually cheaper and the output is better than dispatching a fresh worker for every subtask.

Verification is deliberately adversarial: a worker reporting "done" doesn't end the check; its claim still has to be verified. Each claim goes to independent agents, and it only survives if a majority (two of three) agree. Each finding is checked against the other agents before it's accepted. The Quality Arsenal audits plug in right here, at the gate.

This depends on the doctrine funnel above: every agent, at every level, gets its role-scoped Laws and Rules injected the moment it's dispatched. A worker three levels down gets the same L0–L5 rules as the Master.

This README section is itself an example. A workflow produced it. One agent wrote the draft, independent readers went through it hunting for AI-generated prose, another agent revised against what they flagged, and native speakers handled the translation. So no part of this text came from a single unreviewed pass.

## Stack

It's a Rust workspace with three crates:

- `omega-core` — orchestration, the rules registry, doctor, timeline, cleanup, patrol, file-scope locking.
- `omega-cli` — the `omega` binary, built on `clap`.
- `omega-tui` — the session manager, built on `ratatui`.

Underneath, it runs on [rmux](https://github.com/agentik-os/rmux), a Rust terminal multiplexer: a daemon, a typed SDK, and PTY handling. rmux is a typed Rust library, so OmegaOS calls it directly instead of shelling out to tmux and parsing text. There is no tmux dependency anywhere.

Bun and TypeScript do the PDF report rendering, through Next.js and Playwright. Bash shows up in exactly one place: the install bootstrap.

## Install

You need a Linux box and a Rust toolchain. The installer builds `rmux` and `omega` from source, so the first run isn't instant.

```
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

When it finishes, run the doctor.

### Connecting remotely

The rmux daemon owns every session, so your agents keep running after you disconnect. To get back to them, **attach** — reconnect your terminal to a session that's already running:

```
rmux attach              # re-attach to the last session
rmux attach -t claude-1  # attach to a specific one
rmux list-sessions       # see what's live
```

Detach again with `Ctrl-b d` — the session and its agents keep running without you.

`omega` wraps the entrypoints you actually reach for:

```
omega                       # open the TUI session manager (browse / launch / monitor)
omega attach -t claude-1    # drop straight into one session to work in it
omega master                # jump to the Master session
omega list                  # list every live session
```

Use the menu (`omega`) to manage and launch; use a direct attach (`omega attach -t …`, or `rmux attach -t …`) when you want to type heads-down in a single session — the menu's preview *mirrors* the pane, while a direct attach is the lowest-latency path.

Over SSH from a laptop, plain SSH waits a full network round-trip before echoing each keystroke, so on a distant box typing feels laggy and agent output arrives in chunks — no matter how fast the box is, because it's latency, not CPU. `install.sh` installs [`mosh`](https://mosh.org) for this: it echoes your keystrokes locally and ships screen diffs over UDP, so typing is instant and streaming is smooth at any latency. Connect straight into a session with:

```
mosh user@host -- omega attach -t claude-1
```

In a client like **Termius**: set the host IP + port, turn the **mosh** toggle on, and add a startup snippet — `omega` for the menu, or `omega attach -t <session>` to land directly in a session.

(Use rmux's `Alt+Up/Down` for scrollback, not mosh's PageUp.) The installer also wires `/etc/rmux.conf` and a UTF-8 locale system-wide, so every account — root and future users — gets the same hardened session (mouse scroll, drag-select to the local clipboard over SSH, snappy keys, truecolor) with no per-user setup.

## Usage

`omega doctor` is the first thing to run, and the thing to run whenever something feels off. It checks the whole stack:

```
OmegaOS doctor

  [+] binary           omega 0.1.0
  [+] rmux daemon      connected, 8 live session(s)
  [+] rmux socket      /tmp/rmux-1000/default
  [+] doctrine         6 Laws + 20 Rules
  [+] agent CLI        claude available
  [+] state dir        ~/.omega/state
  [!] telegram service omega-telegram.service inactive (start: systemctl --user start omega-telegram)
  [!] hooks            hook scripts missing from ~/.omega/hooks
  [+] secrets dir      ~/.omega present
  [+] memory           18422MB available
```

The `[!]` lines are warnings, not errors. Here the Telegram service is stopped and the hook scripts aren't installed. Both are optional. Everything else is green.

The command surface, abbreviated to the ones you'll actually reach for:

```
omega menu          Launch the TUI session manager
omega doctor        One-shot health check of the whole stack
omega rules list    List the Laws and Rules
omega dispatch      Dispatch a mission to an oracle
omega orchestrate   Run a full mission end-to-end (classify, plan, dispatch, monitor, gate)
omega spawn-worker  Spawn a worker under the current oracle
omega team          Spawn a team of agents in split panes
omega done          Signal task completion (called by workers)
omega timeline      Replay an oracle's dispatch-to-done history
omega resurrect     Re-spawn a crashed oracle from its persisted state
omega cleanup       Nuclear cleanup of stray sessions and stale state
omega backup        Back up the irreproducible ~/.omega state to a single tgz
omega telegram      Manage the Telegram bridge
omega pdf           Generate a PDF report
```

`omega menu` opens the TUI. The rmux daemon owns every PTY, and sessions carry a role: Master, Oracle, Worker, Home (your own interactive shells), or System (daemons like the Telegram bridge). The TUI lists them with live progress and lets you kill, lock, and rename. There's a kill-all, a nuclear cleanup for when state goes stale, and the doctor built in.

The everyday loop is small. `omega orchestrate` runs a full mission end to end. `omega timeline` replays what an oracle did, dispatch by dispatch. And when an oracle crashes, `omega resurrect` brings it back from its persisted state.

## Quality arsenal

There are around two dozen forensic audits under `skills/audits/` — `codeaudit`, `secaudit`, `perfaudit`, `a11yaudit`, `uiuxaudit`, `flowaudit`, `seoaudit`, `apiaudit`, and more. Each one runs a Gestalt-Popper protocol: a clarity gate up front, then active falsification — the audit tries to break the thing instead of confirming it — then 10x scrutiny on the single most important point instead of spreading attention evenly. When an oracle finishes a mission it auto-selects the audits that fit what just changed, so you don't have to remember which audits to run.

## Linear integration

If you track user feedback in [Linear](https://linear.app), OmegaOS resolves the tickets end to end. Two commands.

`/omg-linear-setup` is a one-time wizard, run inside your own app. It installs an in-app feedback widget (it captures a screenshot, the page URL, the clicked element, and the browser console at report time), the Linear labels the pipeline keys off, and the API route that turns a widget report into a Linear issue. It detects your stack, auth provider, and UI library first, so it writes code that fits the project rather than a generic template.

`/omg-linear` does the work. It reads the open tickets, and for each one it fixes the code, captures before/after evidence, then runs the [Quality Arsenal](#quality-arsenal) audits that fit the change. A ticket only advances if those audits hit 100/100. Then it posts a fix-verification comment on the ticket and moves it to a review state — `In Review` if your team has one, otherwise a neutral `Omega Review` it creates. It never marks a ticket Done; a human does that after checking. The v2 engine runs this through a Workflow: it triages the open tickets, fans the per-ticket fix-and-audit out in parallel, and verifies each resolution adversarially before commenting.

It's trigger-guarded. OmegaOS only touches Linear when you ask for it by name (`/omg-linear`, `fix linear`, a ticket id like `KOM-7`, or a `linear.app` link). The bare word "feedback" never sets it off, and it won't mention Linear unless you do.

```
omega_dir=~/.omega          # the protocol ships to ~/.omega/skills/linear/
/omg-linear-setup           # once per app — installs the widget + labels + route
/omg-linear                 # resolve open tickets: fix -> audit -> comment -> In Review
```

## Limits

I'd rather you know these going in.

- **Linux-first.** Developed on a headless VPS. No Windows. macOS is untested but should mostly work, since it's just Rust and rmux.
- The TUI assumes a 256-color terminal. On a 16-color terminal it'll be ugly.
- The default agent runtime is Claude Code, so you need the `claude` CLI and an Anthropic account. Other agents (pi, codex, gemini, glm) install via `omega install` and run, but they're less exercised.
- **Single machine.** The rmux daemon is local. There's no multi-host orchestration.
- It's 0.1.0. I use it daily, but you'll find rough edges I haven't hit yet.

## Credits

OmegaOS builds on a lot of other people's work:

The largest debt is [rmux](https://github.com/agentik-os/rmux), the Rust terminal multiplexer everything here runs on.

The rest of the Rust stack:

- [ratatui](https://github.com/ratatui/ratatui) and [crossterm](https://github.com/crossterm-rs/crossterm) — the TUI.
- [tokio](https://github.com/tokio-rs/tokio) — the async runtime.
- [clap](https://github.com/clap-rs/clap) and `clap_complete` — the CLI and shell completions.
- [serde](https://github.com/serde-rs/serde) with `serde_json`, `serde_yaml`, and `toml` — config and state.
- [anyhow](https://github.com/dtolnay/anyhow) and [thiserror](https://github.com/dtolnay/thiserror) — error handling.
- `chrono` (timestamps), `dirs` (paths), `fs2` (the advisory file locks behind scope claims), `regex`, `tempfile`, `tracing` with `tracing-subscriber` (logging), and `reqwest` (Telegram and PDF HTTP).

[Claude Code](https://www.anthropic.com) by Anthropic is the agent runtime.

## License

Dual licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Standard Rust convention. Pick whichever you prefer.

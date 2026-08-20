# Agentik Runtime {OS}: Commands

Every command the Runtime exposes. A command that is not documented here does
not exist.

Read the table left to right: what you type, what happens, when you would reach
for it, and what you get back.

---

## Getting started

You do not need to know which OS you want. Start here.

### `agentik`

Run with no arguments. The Runtime asks one question: **what are you trying to
accomplish?** From your answer it composes a stack and offers to install the
first OS in it.

This is the intended entry point for a new user. It never opens with a list of
72 systems.

```bash
agentik
```

### `agentik compose "<objective>"`

Turn an objective into an ordered stack of OS units, with the reason for each.
Does not install anything.

```bash
agentik compose "I want to launch an AI SaaS"
agentik compose "I want to find my first consulting clients"
```

**Returns:** the ordered stack, each entry with its number, name and why it is
in the list. You accept, edit, or discard it.

---

## Lifecycle

### `agentik install <os> [--target <env>]`

Install one OS, or a named stack, into your environment. Resolves declared
dependencies first and offers to install any that are missing.

```bash
agentik install blueprint                 # one OS
agentik install builder-stack             # a named stack
agentik install blueprint --target chatgpt
```

| Flag | Meaning |
|---|---|
| `--target <env>` | prepare for a specific environment: `claude`, `chatgpt`, `gemini`, `codex`. Defaults to the environment you are in. |
| `--with-deps` | install declared dependencies without asking |
| `--dry-run` | print what would be installed, change nothing |

**When to use it:** once per OS, first.
**Returns:** the files placed, the dependencies resolved, and a `doctor` summary.

### `agentik configure <os>`

Collect the minimum context the OS needs to be useful **now**. It deliberately
does not ask for everything it could ever use.

```bash
agentik configure execution
```

**When to use it:** right after install, and again whenever your situation
changes.
**Returns:** the values stored, and what remains optional.

### `agentik run <os>`

Start the OS. This is the command you use every day.

```bash
agentik run blueprint
agentik run execution
```

**When to use it:** every working session.
**Returns:** an active session. The OS opens with its own first question.

### `agentik update <os> | --all`

Update to the latest version. Checks current version, available version,
compatibility, dependencies and migrations, then shows you the changelog before
applying anything.

```bash
agentik update blueprint
agentik update --all
```

| Flag | Meaning |
|---|---|
| `--all` | every installed OS |
| `--check` | report what would update, change nothing |

**When to use it:** when a release lands, or on a cadence you choose.
**Returns:** old version, new version, and the changelog entries between them.
A breaking change is never applied without asking.

### `agentik remove <os>`

Uninstall an OS. Your configuration and any data it produced are kept unless
you pass `--purge`, and `--purge` always asks first.

**When to use it:** when you are genuinely done with it.

---

## Diagnosis and trust

### `agentik doctor [<os>]`

Report what works, what does not, and what to do next. Every surface is
reported present or absent individually. There is no green summary badge
hiding a missing capability.

```bash
agentik doctor              # everything
agentik doctor blueprint    # one OS
```

**Returns:** per-surface status. When an environment cannot support a required
capability, it names the capability, the environment, and the fallback.

### `agentik eval <os>`

Run the OS's evaluation suites. This is the difference between an OS and a
prompt: a prompt can sound impressive and still fail, an OS is testable.

```bash
agentik eval blueprint
```

**Returns:** per-suite pass or fail, with the failing assertion named. Install
reports files; only this reports behaviour.

### `agentik list [--group <g>] [--installed]`

List the suite. 72 units across 9 groups.

```bash
agentik list
agentik list --group build
agentik list --installed
```

---

## Inside OmegaOS

The same suite is reachable from the terminal UI:

```bash
omega menu          # then the OS tab
```

The OS tab renders the same registry, grouped, with a readiness glyph per unit.
Readiness there is measured from disk surfaces, and deliberately tops out at
"runtime and tests present, not executed": finding test files is not proof
anybody ran them.

---

## Command summary

| Command | Does |
|---|---|
| `agentik` | asks your objective, composes a stack |
| `agentik compose "<objective>"` | objective to ordered stack |
| `agentik install <os>` | install, resolving dependencies |
| `agentik configure <os>` | collect the minimum needed context |
| `agentik run <os>` | start using it |
| `agentik update <os>` | update, showing the changelog first |
| `agentik remove <os>` | uninstall, keeping your data by default |
| `agentik doctor [<os>]` | what works, what does not, what next |
| `agentik eval <os>` | does it actually behave correctly |
| `agentik list` | the suite |

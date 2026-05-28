# RUST-BUN-DEFAULT — Rust + Bun for everything that can be

**Category:** Code Quality
**Added:** 2026-05-28

## Rule

Anything that CAN be written in Rust or Bun, IS written in Rust or Bun.
These are the OmegaOS canonical languages because they are:

- **Faster** at startup (Rust binary: ~5ms; Bun: ~25ms vs Node 200ms+)
- **Type-safe** (Rust enums + match exhaustiveness, TypeScript with Bun)
- **Modern** primitives (Rust async/tokio, Bun built-in TS + bundler + test)
- **Better for agentic systems** (Rust for orchestration, Bun for the few
  TS surfaces we have)

## Hierarchy

1. **Rust** — first choice for orchestration, daemons, CLIs, state machines, parsers, networking
2. **Bun** — first choice for browser-touching code (Playwright/PDFgen, browser tests), CLI scripts, anything DOM-related
3. **Bash** — only for bootstrap (install.sh, OS-level setup before Rust is compiled)
4. **Python / Node** — ONLY when a critical dependency requires it (e.g. Playwright works most reliably with Node; an SDK that only exists in Python). Document the exception.

## What does NOT belong

- Python for orchestration logic (replace with Rust)
- Bash for non-bootstrap logic (replace with Rust)
- Node for things Bun can do (replace with Bun)
- Python scripts in install.sh (write Rust subcommands instead)

## When in doubt

Write it in Rust. The startup cost and binary distribution are decisive
for an OS-grade tool. Reach for Bun only when you literally need DOM
rendering (Playwright) or a vast existing TS ecosystem.

## Origin

OmegaOS replaced an ~18000-line Python AISB system with Rust. The
Python version had: slow startup (500-1500ms per CLI invocation), venv
hell during install, subprocess hangs in subprocess.run, GIL limits on
concurrency. Rust solved all four. We do not want to reintroduce those
problems by adding Python or stay-with-the-old-stack inertia.

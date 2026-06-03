# omega-os

One-command installer for **OmegaOS** — the agentic terminal OS (rmux + AI orchestration).

```bash
npx omega-os
```

Shows the OMEGA banner, then:

1. Clones the public repo [`agentik-os/OmegaOS`](https://github.com/agentik-os/OmegaOS)
   (into `~/Station/OmegaOS` if `~/Station` exists, else `./OmegaOS`)
2. Runs its `install.sh` — builds rmux + omega from source, sets up `~/.omega`,
   skills, the 13 AISB agents, rules, hooks, mosh, bun (~8 min on a fresh box)

While it builds, enjoy the **Matrix rain** — or press **`g`** to play **Snake**
(`m` returns to the rain). The OMEGA progress bar stays pinned at the bottom the
whole time. `--plain` disables the animation and shows a simple progress bar.

```bash
npx omega-os --dir /opt/omega   # custom location
npx omega-os --plain            # disable animation, simple bar only
npx omega-os --help
```

After install:

```bash
source ~/.zshrc
omega doctor    # verify (should be healthy)
omega           # launch the TUI
```

Set up Telegram / LLM providers from the TUI — pressing **Enter** on any panel
opens a guided wizard (no command line needed).

**Requirements:** Linux/macOS, `git`, internet, `sudo` for OS packages.
Zero npm runtime dependencies (Node builtins only). Node ≥ 16.

License: MIT OR Apache-2.0 · © Agentik OS

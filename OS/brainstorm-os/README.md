# Brainstorm OS

AgentikOS operative system #3 of the OS suite.

Idea generation and capture: the operating system for producing, ranking and storing ideas.

## Status

**Awaiting drop** - the payload has not been integrated yet. It will arrive as a
zip via the Deposit box and be unpacked here (see `OS/README.md` for the
integration pipeline).

## Integration checklist (when the zip lands)

- [ ] Unpack the payload into this directory
- [ ] Document the runtime here: entrypoint, dependencies, configuration
- [ ] Wire secrets by NAME only (values in `~/.omega/secrets/`, never here)
- [ ] Verify `install.sh` copies everything needed to `~/.omega/os/brainstorm-os/`
- [ ] Verify the TUI OS tab shows it as integrated

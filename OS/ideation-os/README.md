# Ideation OS

AgentikOS build chain #01 of the OS suite.

Idea generation and capture: produce, rank and store the ideas the chain starts from.

## Status

**Awaiting drop** - the payload has not been integrated yet. It will arrive
as a zip via the Deposit box and be unpacked here (see `OS/README.md` and
`docs/OS-SUITE.md` for the integration pipeline).

## Integration checklist (when the zip lands)

- [ ] Unpack the payload into this directory
- [ ] Document the runtime here: entrypoint, dependencies, configuration
- [ ] Wire secrets by NAME only (values in `~/.omega/secrets/`, never here)
- [ ] Verify `install.sh` copies everything needed to `~/.omega/os/ideation-os/`
- [ ] Verify the TUI OS tab shows it as integrated

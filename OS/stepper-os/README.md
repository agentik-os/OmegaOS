# Stepper OS

AgentikOS operative system #5 of the OS suite.

Step-by-step execution: the operating system that walks a blueprint one verified step at a time.

## Status

**Awaiting drop** - the payload has not been integrated yet. It will arrive as a
zip via the Deposit box and be unpacked here (see `OS/README.md` for the
integration pipeline).

## Integration checklist (when the zip lands)

- [ ] Unpack the payload into this directory
- [ ] Document the runtime here: entrypoint, dependencies, configuration
- [ ] Wire secrets by NAME only (values in `~/.omega/secrets/`, never here)
- [ ] Verify `install.sh` copies everything needed to `~/.omega/os/stepper-os/`
- [ ] Verify the TUI OS tab shows it as integrated

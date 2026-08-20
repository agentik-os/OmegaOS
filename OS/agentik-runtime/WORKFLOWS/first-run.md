# Workflow: first run

**Trigger.** A new user with nothing installed. This is what `agentik` with no
arguments does.

**Produces.** One installed, configured OS and a real result from it.

## Steps

1. Ask exactly one question: what are you trying to accomplish. Do not list the
   72 units. A catalogue is what makes people close the terminal.
2. Compose a stack from the answer (see `compose-a-stack.md`). Show it, and say
   plainly which single unit you would start with and why.
3. Install that one unit only. Resolve its `requires` first.
4. Configure only the inputs its `SETUP.md` marks required.
5. Run it. Hand over to the OS, which opens with its own first question.
6. Say what the rest of the stack is for, and that it installs later when the
   first unit has actually paid for itself.

## Completion test

The user has produced one real artifact from one OS, and can name the next unit
they would install and why. If they have a stack installed and no artifact, this
workflow failed.

## Failure paths

| Situation | Response |
|---|---|
| the objective is too vague to compose | ask for the outcome they want in three months, not the task |
| the composed stack is rejected | ask what is wrong with it, recompose, never install anyway |
| the first unit's dependency is missing | name it, offer to install, do not proceed silently |

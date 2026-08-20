# Workflow: Strategy kernel

**Produces:** an approved strategy kernel: a diagnosis of the critical
challenge, a guiding policy that rules options out, and a set of coherent
actions with the resources behind them.

## Trigger

An ambition exists and is being treated as a strategy: a revenue target, a
growth goal, a launch date, or a list of initiatives with no stated obstacle
behind them. Also triggered by `opportunity.named` from Trend & Opportunity {OS}
or `brainstorm.concept.selected` arriving with nothing decided.

Runs the strategy kernel protocol.

## Steps

1. **State ambition and scope.** Write the ambition in the operator's own words,
   the decision horizon it serves, and what is explicitly out of scope for this
   kernel. An ambition with no horizon cannot be reviewed later.
2. **Retrieve the minimum authorized context.** Prior kernels, standing
   exclusions, live constraints, and any evidence already compiled upstream:
   `market.validation.completed`, `validation.verdict.issued`,
   `business_model.viability.assessed`, `research.evidence.compiled`. Do not
   gather new evidence here; request it if it is missing.
3. **Separate the material.** Sort everything into fact, statement, inference,
   assumption and unknown. Label every material claim E1 (authoritative or
   primary) to E5 (preference or value). An assumption dressed as a fact at this
   step becomes a funded bet three steps later.
4. **Diagnose the critical challenge.** Name the single obstacle that, if
   removed, unblocks the most of the rest. List the candidate challenges you
   rejected and why. If several challenges genuinely compete, stop and ask which
   one it is rather than picking.
5. **Emit `strategy.diagnosis.created`.**
6. **Choose the guiding policy.** A policy that constrains action. Prove it
   constrains by naming an attractive, plausible action the policy forbids. If
   nothing is forbidden, the policy is a slogan and the step is not done.
7. **Define coherent actions.** A small set, each one shown to reinforce at
   least one other. Name any pair that pulls against another and resolve it,
   rather than keeping both and calling it balance.
8. **Allocate.** Put time, attention, people and capital against each action, in
   real units, against real capacity. Consume `health.capacity.assessed` and
   `capital.reallocation.proposed` rather than assuming availability.
9. **State assumptions, risks and metrics.** Register the assumptions the kernel
   depends on, run a pre-mortem on the largest one, and define one leading
   signal, one lagging signal and one guardrail, each naming the decision it may
   affect. Assumptions that need settling are handed to Validation {OS} as
   claims, not argued here.
10. **Define review and kill triggers.** What observable event brings this
    kernel back for review, and what would end it. A date alone is not a
    trigger.
11. **Compress to one page.** Challenge, policy, actions, allocation, metrics,
    kill triggers. If it does not fit, the kernel is not yet a choice: return to
    step 6.
12. **Route approval.** Any capital commitment, people decision or change to an
    already-approved strategic objective goes to the human approval boundary
    now, and consequential change goes to Review & Governance {OS} as
    `strategy.change.requested`.
13. **Emit `strategy.kernel.approved`** once the approval returns, and write the
    kernel, its objectives and its metrics to canonical state through Context &
    Memory {OS}.

## Completion test

- The diagnosis names ONE critical challenge, and the rejected candidates are
  listed with reasons.
- The guiding policy forbids at least one named, attractive action.
- Every action is shown to reinforce at least one other, and no unresolved pair
  pulls in opposite directions.
- Every action carries an allocation in real units, and the total fits the
  capacity actually reported, not the capacity hoped for.
- Every material claim carries an E1 to E5 label.
- The kernel fits on one page.
- Review and kill triggers are observable events, not dates alone.
- Owner, completion evidence, review trigger and the memory handoff identifier
  are all present.

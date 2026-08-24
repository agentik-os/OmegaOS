//! AGK Agentic Engineering Lab — the mission loop Omega oracles actually run.
//!
//! This is not a docs-only file. Dispatch seeds the oracle plan with these
//! steps, and the oracle prompt tells the agent to drive `omega progress`
//! / `omega spawn-worker` through them. Writer agents cannot self-approve;
//! `omega done` remains a candidate.

/// Understand → Explain → Design → Build → Debug → Test → Evaluate → Secure
/// → Deploy → Observe → Improve.
pub const LAB_LOOP_STEPS: &[&str] = &[
    "Understand",
    "Explain",
    "Design",
    "Build",
    "Debug",
    "Test",
    "Evaluate",
    "Secure",
    "Deploy",
    "Observe",
    "Improve",
];

/// Pipe-separated plan string for `omega progress --plan`.
pub fn lab_plan_spec() -> String {
    LAB_LOOP_STEPS.join("|")
}

/// Prompt block injected into every dispatched oracle so the Lab loop is
/// operational, not a blog post.
pub fn oracle_lab_block() -> String {
    format!(
        "\n## AGK Agentic Engineering Lab (run this, do not narrate it)\n\
         Persist this plan first: `omega progress <oracle> --plan \"{}\"`\n\
         Walk the steps in order. Keep exactly one task `doing`.\n\
         Required coding-agent dimensions on every mission: repo context, editing, \
         shell, tests, git, sandbox, verification, human-in-the-loop, finish reports.\n\
         Writers (claude|codex|glm) cannot self-approve. `omega done` is a candidate, \
         never a verdict. Fake-done is forbidden.\n\
         YOU fill R-RUBRIC when you write the worker prompt — Done Criteria AND \
         a Verify Command (a runtime check, not a bare filename to eval). Do not \
         leave that for a human `--force`.\n\
         `omega spawn-worker <task> \"<brief>\\nDone Criteria: <measurable>\\nVerify Command: <runtime check>\" --dir <project-dir> --files a,b`\n\
         Workers start in that --dir (the project). The parent never evals Verify Command at spawn.\n\
         Workers are claude|codex|glm only. Hermes is Home (`omega new --agent hermes`), \
         never dispatch and never a worker.\n",
        lab_plan_spec()
    )
}

/// Fields a worker brief must carry so the R-RUBRIC CLI gate lets it spawn.
pub const DONE_CRITERIA_LABEL: &str = "Done Criteria:";
pub const VERIFY_COMMAND_LABEL: &str = "Verify Command:";

/// True when a worker prompt already satisfies the spawn-worker rubric gate.
pub fn worker_prompt_has_rubric(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    let has_done = lower.contains("done criteria")
        || lower.contains("done:")
        || lower.contains("done-criteria");
    let has_verify = lower.contains("verify");
    has_done && has_verify
}

/// Ensure an oracle-authored brief includes the two R-RUBRIC fields.
///
/// Human `omega spawn-worker` from a shell still refuses a missing rubric
/// (unless `--force`). Oracle-originated briefs get the fields appended so a
/// worker is not blocked on prompt wording.
pub fn ensure_oracle_worker_rubric(prompt: &str, task: &str) -> String {
    if worker_prompt_has_rubric(prompt) {
        return prompt.to_string();
    }
    let mut out = prompt.trim_end().to_string();
    let lower = out.to_lowercase();
    if !(lower.contains("done criteria")
        || lower.contains("done:")
        || lower.contains("done-criteria"))
    {
        out.push_str(&format!(
            "\n\n{DONE_CRITERIA_LABEL} task `{task}` is complete, verified by runtime evidence, \
             and reported via `omega done` with a summary. Fake-done is forbidden.\n"
        ));
    }
    if !out.to_lowercase().contains("verify") {
        let artifact = format!("{task}.evidence");
        out.push_str(&format!("\n{VERIFY_COMMAND_LABEL} test -f {artifact}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_plan_is_the_eleven_step_loop() {
        assert_eq!(LAB_LOOP_STEPS.len(), 11);
        assert_eq!(
            lab_plan_spec(),
            "Understand|Explain|Design|Build|Debug|Test|Evaluate|Secure|Deploy|Observe|Improve"
        );
        let block = oracle_lab_block();
        assert!(block.contains("omega progress"));
        assert!(block.contains("Done Criteria"));
        assert!(block.contains("Verify Command"));
        assert!(block.contains("spawn-worker"));
        assert!(block.contains("never dispatch"));
    }

    #[test]
    fn oracle_briefs_gain_rubric_fields_when_missing() {
        let raw = "implement orch test file";
        assert!(!worker_prompt_has_rubric(raw));
        let filled = ensure_oracle_worker_rubric(raw, "orch-test");
        assert!(worker_prompt_has_rubric(&filled), "{filled}");
        assert!(filled.contains("Done Criteria:"));
        assert!(filled.contains("Verify Command:"));
        assert!(
            filled.contains("test -f orch-test.evidence"),
            "auto-fill must be a runtime check, not a bare filename to eval: {filled}"
        );
    }

    #[test]
    fn complete_briefs_are_left_alone() {
        let raw = "Write ORCH_TEST.txt\nDone Criteria: file exists\nVerify Command: test -f ORCH_TEST.txt";
        assert!(worker_prompt_has_rubric(raw));
        assert_eq!(ensure_oracle_worker_rubric(raw, "t"), raw);
    }
}

#!/usr/bin/env python3
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "hooks"))

import omega_plan_state


class HookPlanStateFixtures(unittest.TestCase):
    def analyze(self, name):
        result = omega_plan_state.analyze(
            ROOT / "tests" / "fixtures" / "hooks" / name
        )
        self.assertIsNotNone(result)
        return result

    def test_all_provider_and_negative_fixtures(self):
        cases = {
            "codex-update-plan.jsonl": {
                "plan_ever": True,
                "total_tasks": 2,
                "open_items": ["Verify runtime"],
            },
            "codex-successful-verification.jsonl": {
                "edited": True,
                "verified": True,
            },
            "codex-failed-verification.jsonl": {
                "edited": True,
                "verified": False,
            },
            "codex-irrelevant-command.jsonl": {
                "edited": True,
                "verified": False,
            },
            "codex-custom-tool-verification.jsonl": {
                "edited": True,
                "verified": True,
            },
            "claude-task-plan.jsonl": {
                "plan_ever": True,
                "total_tasks": 1,
                "open_items": [],
            },
            "claude-todo-open.jsonl": {
                "plan_ever": True,
                "total_tasks": 2,
                "open_items": ["Verify runtime"],
            },
            "claude-successful-verification.jsonl": {
                "edited": True,
                "verified": True,
            },
            "claude-edit-invalidates-verification.jsonl": {
                "edited": True,
                "verified": False,
            },
            "gemini-update-plan.jsonl": {
                "plan_ever": True,
                "total_tasks": 2,
                "open_items": ["Verify runtime"],
            },
            "gemini-successful-verification.jsonl": {
                "edited": True,
                "verified": True,
            },
            "sidechain-plan-is-ignored.jsonl": {
                "plan_ever": False,
                "edited": False,
                "total_tasks": 0,
            },
            # R-PREFLIGHT: a read-only preflight awaiting approval. The finish
            # guard exempts exactly this shape (preflight AND zero mutations),
            # so both keys are asserted — an exemption keyed on the text alone
            # would be a bypass.
            "claude-preflight-awaiting-approval.jsonl": {
                "plan_ever": False,
                "edited": False,
                "mutations": 0,
                "preflight": True,
            },
        }
        self.assertEqual(len(cases), 13)
        for fixture, expected in cases.items():
            with self.subTest(fixture=fixture):
                observed = self.analyze(fixture)
                for key, value in expected.items():
                    self.assertEqual(observed[key], value)

    def test_preflight_detection_needs_structure_not_vocabulary(self):
        # The markers must OPEN a line and be a heading/label, so ordinary prose
        # that happens to say "plan" or "goal" never buys a session a free stop.
        preflight_en = (
            "**Goal.** Ship the checkout route.\n\n"
            "**Blocking questions.** None.\n\n"
            "**Assumptions.**\n1. Price ids come from the operator.\n\n"
            "**Plan.** route.ts, then the webhook.\n"
        )
        preflight_fr = (
            "## Objectif\nAjouter la route de paiement.\n\n"
            "## Questions bloquantes\nAucune.\n\n"
            "## Hypothèses\n1. Les prix existent déjà.\n\n"
            "## Plan\nroute.ts puis le webhook.\n"
        )
        two_families_only = "**Goal.** Ship it.\n\n**Plan.** Later today.\n"
        prose = (
            "I read the code. The plan for the migration is clear and the goals "
            "for later are noted. Tell me how you want to proceed."
        )
        self.assertTrue(omega_plan_state.is_preflight(preflight_en))
        self.assertTrue(omega_plan_state.is_preflight(preflight_fr))
        self.assertFalse(omega_plan_state.is_preflight(two_families_only))
        self.assertFalse(omega_plan_state.is_preflight(prose))
        self.assertFalse(omega_plan_state.is_preflight(""))
        self.assertFalse(omega_plan_state.is_preflight(None))

    def test_missing_transcript_is_unobservable(self):
        self.assertIsNone(
            omega_plan_state.analyze(ROOT / "tests" / "fixtures" / "missing.jsonl")
        )


if __name__ == "__main__":
    unittest.main()

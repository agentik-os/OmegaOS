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
        }
        self.assertEqual(len(cases), 12)
        for fixture, expected in cases.items():
            with self.subTest(fixture=fixture):
                observed = self.analyze(fixture)
                for key, value in expected.items():
                    self.assertEqual(observed[key], value)

    def test_missing_transcript_is_unobservable(self):
        self.assertIsNone(
            omega_plan_state.analyze(ROOT / "tests" / "fixtures" / "missing.jsonl")
        )


if __name__ == "__main__":
    unittest.main()

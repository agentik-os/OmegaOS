#!/usr/bin/env python3
"""Apply the OS Builder release threshold to a filled scorecard.

The sixteen quality dimensions are JUDGED, not measured: a human or a model
reads the package and assigns 0 to 5 per dimension with written evidence. That
judgment cannot be automated honestly, so this tool does not attempt it.

What IS deterministic is the arithmetic that turns sixteen scores into a release
verdict, and that arithmetic is exactly what gets fudged when it lives in prose.
"average is about four and a half" is how a build with a 3 in security ships.
So the threshold is code, it reads a scorecard file, and it says RELEASE or
BLOCKED with the specific dimension that blocked it.

THE THRESHOLD (see EVALS/OS-QUALITY-RUBRIC.md for how the upstream wording was
resolved):

  1. all sixteen dimensions present, each an integer 0..5, each with evidence
  2. every dimension >= 4
  3. the five CRITICAL dimensions >= 4 with no waiver possible
  4. the mean over all sixteen >= 4.3

A waiver may lift rule 2 for a NON critical dimension only, and only when it
carries an approver and a reason. It can never lift rule 3 or rule 4.

Usage:
    score_os.py <scorecard.json>            verdict, human readable
    score_os.py <scorecard.json> --json     machine-readable, for a gate
    score_os.py --template                  print an empty scorecard to fill

Exit codes: 0 RELEASE, 1 BLOCKED, 2 malformed scorecard or usage error.
"""
import json
import sys

DIMENSIONS = [
    "value_proposition", "scope", "domain_depth", "human_skill",
    "operating_logic", "evidence_discipline", "decision_quality",
    "artifact_quality", "executive_usability", "security", "testability",
    "traceability", "reusability", "installability", "handoffs", "adapters",
]

# Never waivable. An OS weak in any of these is not weak, it is wrong.
CRITICAL = {
    "evidence_discipline", "operating_logic", "artifact_quality",
    "security", "testability",
}

MIN_DIMENSION = 4
MIN_AVERAGE = 4.3


def template():
    return {
        "os": "<slug>",
        "version": "0.0.0",
        "scored_by": "<who or what produced these scores>",
        "date": "<YYYY-MM-DD>",
        "scores": {d: {"score": 0, "evidence": ""} for d in DIMENSIONS},
        "waivers": [],
    }


def read_scores(card):
    """Return (scores, structural_errors). Scores map dimension -> int."""
    errors = []
    raw = card.get("scores")
    if not isinstance(raw, dict):
        return {}, ["'scores' is missing or is not an object"]

    scores = {}
    for dim in DIMENSIONS:
        entry = raw.get(dim)
        if entry is None:
            errors.append(f"missing dimension: {dim}")
            continue
        if isinstance(entry, (int, float)) and not isinstance(entry, bool):
            value, evidence = entry, ""
        elif isinstance(entry, dict):
            value, evidence = entry.get("score"), entry.get("evidence") or ""
        else:
            errors.append(f"{dim}: unsupported entry type {type(entry).__name__}")
            continue
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            errors.append(f"{dim}: score is not a number")
            continue
        if value != int(value) or not 0 <= value <= 5:
            errors.append(f"{dim}: score {value} is not an integer 0..5")
            continue
        if not str(evidence).strip():
            errors.append(f"{dim}: score {int(value)} carries no evidence")
            continue
        scores[dim] = int(value)

    for unknown in sorted(set(raw) - set(DIMENSIONS)):
        errors.append(f"unknown dimension: {unknown}")
    return scores, errors


def read_waivers(card):
    """Return (waived_dimensions, waiver_errors)."""
    waived, errors = set(), []
    for entry in card.get("waivers") or []:
        if not isinstance(entry, dict):
            errors.append("waiver is not an object")
            continue
        dim = entry.get("dimension")
        if dim not in DIMENSIONS:
            errors.append(f"waiver names unknown dimension: {dim!r}")
            continue
        if dim in CRITICAL:
            errors.append(f"waiver on CRITICAL dimension {dim} is not permitted")
            continue
        if not str(entry.get("approver") or "").strip():
            errors.append(f"waiver on {dim} has no approver")
            continue
        if not str(entry.get("reason") or "").strip():
            errors.append(f"waiver on {dim} has no reason")
            continue
        waived.add(dim)
    return waived, errors


def evaluate(card):
    scores, errors = read_scores(card)
    waived, waiver_errors = read_waivers(card)
    errors += waiver_errors

    if errors:
        return {"verdict": "MALFORMED", "errors": errors, "blockers": [],
                "average": None, "scores": scores, "waived": sorted(waived)}

    blockers = []
    for dim, value in sorted(scores.items()):
        if value >= MIN_DIMENSION:
            continue
        if dim in CRITICAL:
            blockers.append(f"{dim} = {value}, CRITICAL, minimum {MIN_DIMENSION}, no waiver possible")
        elif dim in waived:
            continue
        else:
            blockers.append(f"{dim} = {value}, minimum {MIN_DIMENSION}, no waiver on file")

    average = sum(scores.values()) / len(DIMENSIONS)
    if average < MIN_AVERAGE:
        blockers.append(f"average {average:.4f} is below the {MIN_AVERAGE} threshold")

    return {
        "verdict": "RELEASE" if not blockers else "BLOCKED",
        "errors": [],
        "blockers": blockers,
        "average": round(average, 4),
        "scores": scores,
        "waived": sorted(waived),
    }


def main():
    if "--template" in sys.argv:
        print(json.dumps(template(), indent=2))
        return 0

    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    as_json = "--json" in sys.argv
    if len(args) != 1:
        print(__doc__.strip().split("Usage:")[1].strip(), file=sys.stderr)
        return 2

    try:
        with open(args[0], encoding="utf-8") as handle:
            card = json.load(handle)
    except (OSError, ValueError) as exc:
        print(f"cannot read scorecard: {exc}", file=sys.stderr)
        return 2
    if not isinstance(card, dict):
        print("scorecard must be a JSON object", file=sys.stderr)
        return 2

    result = evaluate(card)
    result["os"] = card.get("os")
    result["version"] = card.get("version")
    result["scored_by"] = card.get("scored_by")
    result["date"] = card.get("date")

    if as_json:
        print(json.dumps(result, indent=2))
        return {"RELEASE": 0, "BLOCKED": 1, "MALFORMED": 2}[result["verdict"]]

    print(f"os      : {result['os']} {result['version']}")
    print(f"scored  : {result['scored_by']} on {result['date']}")

    if result["verdict"] == "MALFORMED":
        print("\nMALFORMED  the scorecard cannot be graded")
        for err in result["errors"]:
            print(f"        {err}")
        print("\nA missing score is not a zero. Fill it or say why it cannot be filled.")
        return 2

    print(f"average : {result['average']} (threshold {MIN_AVERAGE})\n")
    for dim in DIMENSIONS:
        value = result["scores"][dim]
        flag = ""
        if dim in CRITICAL:
            flag = " CRITICAL"
        if value < MIN_DIMENSION:
            flag += " WAIVED" if dim in result["waived"] else " BELOW"
        bar = "#" * value + "." * (5 - value)
        print(f"  {bar}  {value}  {dim}{flag}")

    print()
    if result["verdict"] == "RELEASE":
        print("RELEASE  threshold met. Proceed to phase 14.")
        return 0
    print(f"BLOCKED  {len(result['blockers'])} problem(s)")
    for blocker in result["blockers"]:
        print(f"        {blocker}")
    print()
    print("Repair the named dimensions and re-score. Do not re-score without")
    print("changing the package: a second opinion on the same artifact is not a repair.")
    return 1


if __name__ == "__main__":
    sys.exit(main())

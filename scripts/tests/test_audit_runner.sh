#!/usr/bin/env bash
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT/skills/audits/_shared/audit-runner.sh"
TEST_ROOT="$(mktemp -d)"
export OMEGA_DIR="$TEST_ROOT/omega"
PROJECT="$TEST_ROOT/project"
mkdir -p "$OMEGA_DIR/skills/audits" "$OMEGA_DIR/lib/audit-gather" "$PROJECT"
cp "$ROOT/skills/audits/registry.toml" "$OMEGA_DIR/skills/audits/registry.toml"

python3 - "$ROOT/skills/audits/registry.toml" \
    "$ROOT/skills/audits/audit-orchestrator/SKILL.md" <<'PY'
import re
import sys
import tomllib

with open(sys.argv[1], "rb") as handle:
    registry = tomllib.load(handle)
ids = [entry["id"] for entry in registry["audits"]]
assert registry["meta"]["total_audits"] == len(ids) == 23
text = open(sys.argv[2], encoding="utf-8").read()
full_mode = text.split("## Full Audit Mode", 1)[1].split("## State Tracking", 1)[0]
for audit_id in ids:
    assert len(re.findall(rf"(?<![a-z]){re.escape(audit_id)}(?![a-z])", full_mode)) == 1, audit_id
assert "Skip Plan + Fix" not in text
PY

cleanup() {
    rm -rf "$TEST_ROOT"
}
trap cleanup EXIT

fail() {
    echo "test_audit_runner: $*" >&2
    exit 1
}

assert_exit() {
    local expected="$1"
    shift
    set +e
    "$@" >/dev/null 2>&1
    local actual=$?
    set -e
    [ "$actual" -eq "$expected" ] || fail "expected exit $expected, got $actual: $*"
}

set -e

assert_exit 2 "$RUNNER" code "$PROJECT" --hinge="runtime behavior"
assert_exit 2 "$RUNNER" unknown "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"
assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --threshold=101

cat > "$OMEGA_DIR/lib/audit-gather/code.sh" <<'SH'
#!/usr/bin/env bash
exit 7
SH
chmod +x "$OMEGA_DIR/lib/audit-gather/code.sh"
assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"

cat > "$OMEGA_DIR/lib/audit-gather/code.sh" <<'SH'
#!/usr/bin/env bash
set -eu
mkdir -p "$1"
printf '{"violations":[]}\n' > "$1/results.json"
SH
chmod +x "$OMEGA_DIR/lib/audit-gather/code.sh"
assert_exit 0 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"

[ -d "$PROJECT/audits/.codeaudit/raw" ] || \
    fail "runner did not use the canonical registry-id output root"
[ ! -e "$PROJECT/.code" ] || fail "legacy root-level audit directory was created"

python3 - "$PROJECT/audits/.codeaudit/evidence-summary.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    summary = json.load(handle)
assert summary["schema_version"] == 1
assert summary["audit"] == "codeaudit"
assert summary["user_need"] == "reliable audits"
assert summary["hinge"] == "runtime behavior"
assert summary["provenance"]["gather_mode"] == "programmatic"
assert summary["evidence"]["tool_outputs"][0]["file"] == "results.json"
PY

assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --finalize

python3 - "$PROJECT/audits/.codeaudit/verdict.json" <<'PY'
import json
import sys

verdict = {
    "score": 69,
    "confidence": "high",
    "skill_used": "codeaudit",
    "user_need_match": {
        "quote": "reliable audits",
        "addressed": True,
        "evidence": "scripts/tests/test_audit_runner.sh:1",
        "edge_cases_covered": [],
    },
    "falsifiable_tests": [{"name": "runner", "passed": True}],
    "hinge_findings": [
        {
            "location": "runtime behavior",
            "concern": "fail-open result",
            "verified_safe_by": "runner",
        }
    ],
    "issues_found_and_fixed": [],
    "confidence_basis": "Direct runner execution.",
}
with open(sys.argv[1], "w", encoding="utf-8") as handle:
    json.dump(verdict, handle)
PY
assert_exit 1 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --finalize

python3 - "$PROJECT/audits/.codeaudit/verdict.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    verdict = json.load(handle)
verdict["score"] = 70
with open(sys.argv[1], "w", encoding="utf-8") as handle:
    json.dump(verdict, handle)
PY
assert_exit 0 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --finalize

echo "test_audit_runner: PASS"

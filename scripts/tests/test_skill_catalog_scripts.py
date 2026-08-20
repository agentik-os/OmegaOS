#!/usr/bin/env python3
"""Contract tests for the SkillCatalogV1 Atlas/RAG projections."""

import json
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


REPO = Path(__file__).resolve().parents[2]
ATLAS_SCRIPT = REPO / "scripts/omega-skills-atlas.py"
RAG_SCRIPT = REPO / "scripts/omega-skills-rag.py"
CLI_SCRIPT = REPO / "scripts/omega-skills"


def catalog(digest, skills):
    return {
        "schema_version": 1,
        "content_digest": digest,
        "skills": skills,
        "warnings": [],
    }


def source_digest(root_id, relative_path, content):
    normalized = content.replace("\r\n", "\n").encode()
    rows = [[root_id, relative_path, hashlib.sha256(normalized).hexdigest()]]
    payload = json.dumps(rows, ensure_ascii=False, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def skill(name, description, relative_path, category="Custom"):
    return {
        "name": name,
        "description": description,
        "root_id": "omegaos",
        "relative_path": relative_path,
        "content_digest": "f" * 64,
        "aliases": [],
        "triggers": [name],
        "category": category,
        "phases": None,
        "max_score": None,
        "read_only": False,
        "dependencies": {},
        "provider_states": {
            "claude": {"state": "enabled"},
            "codex": {"state": "enabled"},
        },
    }


class SkillCatalogScriptTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.omega = Path(self.temp.name)
        self.catalog_path = self.omega / "skill-catalog-v1.json"
        self.env = os.environ.copy()
        self.env.update({
            "OMEGA_DIR": str(self.omega),
            "OMEGA_SKILL_CATALOG": str(self.catalog_path),
            "OPENAI_API_KEY": "",
        })

    def tearDown(self):
        self.temp.cleanup()

    def write_catalog(self, value):
        self.catalog_path.write_text(json.dumps(value), encoding="utf-8")

    def run_script(self, script, *args, check=True):
        return subprocess.run(
            [sys.executable, str(script), *args],
            env=self.env,
            text=True,
            capture_output=True,
            check=check,
        )

    def test_atlas_consumes_canonical_catalog_and_keeps_nested_families(self):
        self.write_catalog(catalog("a" * 64, [
            skill(
                "high-end-visual-design",
                "Premium UI",
                "design/high-end-visual-design/SKILL.md",
                "Design",
            ),
            skill(
                "caio-ai-readiness-assessment",
                "CAIO readiness",
                "caio/caio-ai-readiness-assessment/SKILL.md",
            ),
            skill(
                "marketing-master",
                "Marketing mastery",
                "marketing-mastery/marketing-master/SKILL.md",
                "Marketing",
            ),
        ]))
        self.run_script(ATLAS_SCRIPT)
        atlas = json.loads(
            (self.omega / "skills-atlas.json").read_text(encoding="utf-8"))
        self.assertEqual(atlas["catalog_hash"], "a" * 64)
        self.assertEqual(atlas["native_count"], 3)
        self.assertEqual(
            {row["name"] for row in atlas["native"]},
            {
                "high-end-visual-design",
                "caio-ai-readiness-assessment",
                "marketing-master",
            },
        )

    def test_legacy_fallback_is_recursive_and_excludes_vendor(self):
        self.catalog_path.unlink(missing_ok=True)
        wanted = self.omega / "skills/design/deep/high-end-visual-design"
        vendor = self.omega / "skills/pdfgen/node_modules/playwright/skill"
        wanted.mkdir(parents=True)
        vendor.mkdir(parents=True)
        wanted.joinpath("SKILL.md").write_text(
            "---\nname: high-end-visual-design\n"
            "description: Premium UI\n---\n",
            encoding="utf-8",
        )
        vendor.joinpath("SKILL.md").write_text(
            "---\nname: vendor-playwright\n"
            "description: Excluded\n---\n",
            encoding="utf-8",
        )
        self.run_script(ATLAS_SCRIPT)
        atlas = json.loads(
            (self.omega / "skills-atlas.json").read_text(encoding="utf-8"))
        self.assertEqual(atlas["native_count"], 1)
        self.assertEqual(atlas["native"][0]["name"], "high-end-visual-design")

    def test_rag_rebuilds_when_catalog_hash_drifts(self):
        self.write_catalog(catalog("a" * 64, [
            skill("alpha", "First description", "alpha/SKILL.md"),
        ]))
        self.run_script(ATLAS_SCRIPT)
        self.run_script(RAG_SCRIPT, "build")
        first = json.loads(
            (self.omega / "skills-rag/meta.json").read_text(encoding="utf-8"))
        self.assertEqual(first["catalog_hash"], "a" * 64)

        self.write_catalog(catalog("b" * 64, [
            skill("alpha", "Changed description", "alpha/SKILL.md"),
        ]))
        result = self.run_script(RAG_SCRIPT, "query", "changed", "--json")
        self.assertIn("index drift detected", result.stderr)
        second = json.loads(
            (self.omega / "skills-rag/meta.json").read_text(encoding="utf-8"))
        self.assertEqual(second["catalog_hash"], "b" * 64)
        self.assertNotEqual(first["corpus_hash"], second["corpus_hash"])

    def test_cli_rebuilds_stale_atlas_before_search(self):
        bin_dir = self.omega / "bin"
        bin_dir.mkdir()
        shutil.copy2(ATLAS_SCRIPT, bin_dir / ATLAS_SCRIPT.name)
        shutil.copy2(RAG_SCRIPT, bin_dir / RAG_SCRIPT.name)
        self.write_catalog(catalog("a" * 64, [
            skill("alpha", "Alpha", "alpha/SKILL.md"),
        ]))
        self.run_script(bin_dir / ATLAS_SCRIPT.name)
        self.write_catalog(catalog("b" * 64, [
            skill(
                "high-end-visual-design",
                "Premium UI",
                "design/high-end-visual-design/SKILL.md",
                "Design",
            ),
        ]))
        result = subprocess.run(
            ["bash", str(CLI_SCRIPT), "high-end-visual-design"],
            env=self.env,
            text=True,
            capture_output=True,
            check=True,
        )
        self.assertIn("high-end-visual-design", result.stdout)
        atlas = json.loads(
            (self.omega / "skills-atlas.json").read_text(encoding="utf-8"))
        self.assertEqual(atlas["catalog_hash"], "b" * 64)

    def test_cli_recompiles_catalog_when_source_tree_changes(self):
        source_root = self.omega / "source-skills"
        skill_dir = source_root / "alpha"
        skill_dir.mkdir(parents=True)
        relative = "alpha/SKILL.md"
        initial = "---\nname: alpha\ndescription: First\n---\n"
        changed = "---\nname: alpha\ndescription: Changed\n---\n"
        skill_dir.joinpath("SKILL.md").write_text(initial, encoding="utf-8")
        value = catalog("a" * 64, [skill("alpha", "First", relative)])
        value["source_roots"] = [{"id": "omegaos", "path": str(source_root)}]
        value["source_tree_digest"] = source_digest("omegaos", relative, initial)
        self.write_catalog(value)

        bin_dir = self.omega / "bin"
        bin_dir.mkdir()
        shutil.copy2(ATLAS_SCRIPT, bin_dir / ATLAS_SCRIPT.name)
        shutil.copy2(RAG_SCRIPT, bin_dir / RAG_SCRIPT.name)
        self.run_script(bin_dir / ATLAS_SCRIPT.name)

        fake_cli = self.omega / "fake-omega.py"
        fake_cli.write_text(
            "#!/usr/bin/env python3\n"
            "import hashlib,json,sys\n"
            "from pathlib import Path\n"
            "out=Path(sys.argv[sys.argv.index('--out')+1])\n"
            "data=json.loads(out.read_text())\n"
            "root=Path(data['source_roots'][0]['path'])\n"
            "raw=(root/'alpha/SKILL.md').read_bytes().replace(b'\\r\\n',b'\\n')\n"
            "digest=hashlib.sha256(raw).hexdigest()\n"
            "rows=[['omegaos','alpha/SKILL.md',digest]]\n"
            "data['skills'][0]['content_digest']=digest\n"
            "data['skills'][0]['description']='Changed'\n"
            "data['source_tree_digest']=hashlib.sha256(json.dumps(rows,separators=(',',':')).encode()).hexdigest()\n"
            "data['content_digest']='b'*64\n"
            "out.write_text(json.dumps(data))\n",
            encoding="utf-8",
        )
        fake_cli.chmod(0o755)
        self.env["OMEGA_CLI"] = str(fake_cli)
        skill_dir.joinpath("SKILL.md").write_text(changed, encoding="utf-8")

        result = subprocess.run(
            ["bash", str(CLI_SCRIPT), "alpha"],
            env=self.env,
            text=True,
            capture_output=True,
            check=True,
        )
        self.assertIn("Changed", result.stdout)
        atlas = json.loads(
            (self.omega / "skills-atlas.json").read_text(encoding="utf-8"))
        self.assertEqual(atlas["catalog_hash"], "b" * 64)
        self.assertEqual(
            atlas["source_tree_digest"], source_digest("omegaos", relative, changed))


if __name__ == "__main__":
    unittest.main()

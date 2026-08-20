#!/usr/bin/env python3
"""Static falsification tests for install and release provenance contracts."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class ReleaseContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.install = ROOT.joinpath("install.sh").read_text(encoding="utf-8")
        cls.release = ROOT.joinpath(".github/workflows/release.yml").read_text(
            encoding="utf-8"
        )
        cls.ci = ROOT.joinpath(".github/workflows/ci.yml").read_text(encoding="utf-8")

    def test_prebuilt_requires_checksum_and_build_info(self):
        self.assertIn('if ! curl -fsSL "$base/$tarball.sha256"', self.install)
        self.assertIn('-f "$tmp/BUILD-INFO.json"', self.install)
        self.assertNotIn("if the .sha256 sidecar exists", self.install)

    def test_source_build_never_falls_back_to_unlocked_dependencies(self):
        self.assertIn("cargo_build_live --locked", self.install)
        self.assertNotIn("cargo_build_live --locked || cargo_build_live", self.install)

    def test_rmux_revision_is_fail_closed(self):
        self.assertIn('checkout --detach -q "$RMUX_REV"', self.install)
        self.assertIn('checkout --detach "${{ steps.rmux.outputs.rev }}"', self.release)
        self.assertNotIn("rev checkout failed; using default branch", self.release)

    def test_release_emits_sbom_and_signed_attestations(self):
        self.assertIn("anchore/sbom-action@v0.24.0", self.release)
        self.assertGreaterEqual(self.release.count("uses: actions/attest@v4"), 2)
        self.assertIn("BUILD-INFO.json", self.release)

    def test_ci_quality_gates_are_blocking(self):
        self.assertIn("cargo fmt --all -- --check", self.ci)
        self.assertIn("cargo clippy --workspace --all-targets -- -D warnings", self.ci)
        self.assertNotIn("continue-on-error", self.ci)


if __name__ == "__main__":
    unittest.main()

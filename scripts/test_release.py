#!/usr/bin/env python3
"""Unit tests for scripts/release/plan.sh and publish.sh against a fake `gh`
that keeps releases as folders."""

import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FAKE_GH = """#!/bin/sh
echo "$*" >>"$FAKE_GH/calls.log"
case "$1 $2" in
  "release view")
    [ -d "$FAKE_GH/releases/$3" ] || exit 1
    case "$*" in *--json*) ls "$FAKE_GH/releases/$3" ;; esac ;;
  "release delete") rm -rf "$FAKE_GH/releases/$3" ;;
  "release delete-asset") rm "$FAKE_GH/releases/$3/$4" ;;
  "release edit") ;;
  "release upload")
    [ -z "$FAKE_GH_FAIL_UPLOAD" ] || exit 1
    tag="$3"; shift 3
    for a in "$@"; do case "$a" in -*) ;; *) cp "$a" "$FAKE_GH/releases/$tag/" ;; esac; done ;;
  "api -X") [ -z "$FAKE_GH_FAIL_TAG" ] || { echo "HTTP 403" >&2; exit 1; } ;;
  "release create")
    tag="$3"; shift 3; mkdir -p "$FAKE_GH/releases/$tag"
    for a in "$@"; do case "$a" in -*) break ;; *) cp "$a" "$FAKE_GH/releases/$tag/" ;; esac; done ;;
  *) exit 2 ;;
esac
"""


class ReleaseScriptsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.gh = Path(self.tmp.name) / "gh-home"
        (self.gh / "releases").mkdir(parents=True)
        bin_dir = Path(self.tmp.name) / "bin"
        bin_dir.mkdir()
        (bin_dir / "gh").write_text(FAKE_GH)
        (bin_dir / "gh").chmod(0o755)
        self.env = dict(os.environ, FAKE_GH=str(self.gh), PATH=f"{bin_dir}:{os.environ['PATH']}")
        self.env.pop("GITHUB_OUTPUT", None)

    def tearDown(self):
        self.tmp.cleanup()

    def run_script(self, *args):
        return subprocess.run([str(ROOT / "scripts/release" / args[0]), *args[1:]], env=self.env,
                              capture_output=True, text=True)

    def plan(self, channel):
        r = self.run_script("plan.sh", channel)
        self.assertEqual(r.returncode, 0, r.stderr)
        return dict(line.split("=", 1) for line in r.stdout.splitlines())

    def test_the_first_edge_build_of_a_day_also_makes_the_dated_release(self):
        first = self.plan("edge")
        self.assertEqual(first["tags"], f"edge edge-{first['date']}")
        self.assertEqual(first["toolchain"], f"edge-{first['date']}-{first['commit'][:8]}")
        (self.gh / "releases" / f"edge-{first['date']}").mkdir()
        self.assertEqual(self.plan("edge")["tags"], "edge")

    def test_a_stable_release_is_made_once_per_version(self):
        stable = self.plan("stable")
        self.assertEqual(stable["tags"], f"v{stable['version']}")
        self.assertEqual(stable["toolchain"], stable["version"])
        (self.gh / "releases" / f"v{stable['version']}").mkdir()
        r = self.run_script("plan.sh", "stable")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("already released", r.stderr)
        self.assertEqual(self.run_script("plan.sh", "nightly").returncode, 2)

    def publish(self, dist, toolchain, tags, ok=True):
        r = self.run_script("publish.sh", str(dist), "edge", "0.3.0", "a" * 40, "2026-11-22", toolchain, tags)
        self.assertEqual(r.returncode == 0, ok, r.stderr)

    def test_edge_replaces_the_rolling_release_and_leaves_dated_ones(self):
        dist = Path(self.tmp.name) / "dist"
        dist.mkdir()
        (dist / "cpl-x86_64-unknown-linux-musl.tar.gz").write_bytes(b"one")
        (dist / "editors.tar.gz").write_bytes(b"editors")
        self.publish(dist, "edge-2026-11-22-aaaaaaaa", "edge edge-2026-11-22")
        releases = self.gh / "releases"
        dated = json.loads((releases / "edge-2026-11-22" / "latest.json").read_text())
        self.assertEqual(dated["toolchain"], "edge-2026-11-22-aaaaaaaa")
        sums = (releases / "edge" / "SHA256SUMS").read_text()
        line = f"{hashlib.sha256(b'one').hexdigest()}  cpl-x86_64-unknown-linux-musl.tar.gz"
        self.assertIn(line, sums)
        self.assertEqual((releases / "edge" / "cpl-x86_64-unknown-linux-musl.tar.gz.sha256").read_text(), line + "\n")
        self.assertFalse((releases / "edge" / "editors.tar.gz.sha256").exists())

        (dist / "cpl-x86_64-unknown-linux-musl.tar.gz").write_bytes(b"two")
        (dist / "editors.tar.gz").unlink()
        self.publish(dist, "edge-2026-11-22-bbbbbbbb", "edge")
        rolling = json.loads((releases / "edge" / "latest.json").read_text())
        self.assertEqual(rolling["toolchain"], "edge-2026-11-22-bbbbbbbb")
        self.assertEqual((releases / "edge" / "cpl-x86_64-unknown-linux-musl.tar.gz").read_bytes(), b"two")
        self.assertFalse((releases / "edge" / "editors.tar.gz").exists())
        self.assertEqual((releases / "edge-2026-11-22" / "cpl-x86_64-unknown-linux-musl.tar.gz").read_bytes(), b"one")
        calls = (self.gh / "calls.log").read_text().splitlines()
        self.assertEqual(sum(c.startswith("release create edge-2026-11-22 ") for c in calls), 1)
        self.assertFalse([c for c in calls if c.startswith("release delete edge")])
        self.assertIn(f"api -X PATCH repos/{{owner}}/{{repo}}/git/refs/tags/edge -f sha={'a' * 40} -F force=true", calls)
        uploads = [c for c in calls if c.startswith("release upload edge ")]
        self.assertIn("latest.json", uploads[-1])
        self.assertNotIn("latest.json", uploads[-2])

    def test_a_failed_edge_upload_leaves_the_old_edge_installable(self):
        dist = Path(self.tmp.name) / "dist"
        dist.mkdir()
        (dist / "cpl-x86_64-unknown-linux-musl.tar.gz").write_bytes(b"one")
        self.publish(dist, "edge-2026-11-22-aaaaaaaa", "edge")
        (dist / "cpl-x86_64-unknown-linux-musl.tar.gz").write_bytes(b"two")
        self.env["FAKE_GH_FAIL_UPLOAD"] = "1"
        self.publish(dist, "edge-2026-11-22-bbbbbbbb", "edge", ok=False)
        edge = self.gh / "releases" / "edge"
        self.assertEqual(json.loads((edge / "latest.json").read_text())["toolchain"], "edge-2026-11-22-aaaaaaaa")
        self.assertEqual((edge / "cpl-x86_64-unknown-linux-musl.tar.gz").read_bytes(), b"one")

    def test_edge_publishes_its_assets_when_the_tag_cannot_move(self):
        dist = Path(self.tmp.name) / "dist"
        dist.mkdir()
        (dist / "cpl-x86_64-unknown-linux-musl.tar.gz").write_bytes(b"one")
        self.publish(dist, "edge-2026-11-22-aaaaaaaa", "edge")
        (dist / "cpl-x86_64-unknown-linux-musl.tar.gz").write_bytes(b"two")
        self.env["FAKE_GH_FAIL_TAG"] = "1"
        self.publish(dist, "edge-2026-11-22-bbbbbbbb", "edge")
        edge = self.gh / "releases" / "edge"
        self.assertEqual(json.loads((edge / "latest.json").read_text())["toolchain"], "edge-2026-11-22-bbbbbbbb")
        self.assertEqual((edge / "cpl-x86_64-unknown-linux-musl.tar.gz").read_bytes(), b"two")
        edits = [l for l in (self.gh / "calls.log").read_text().splitlines() if l.startswith("release edit edge")]
        self.assertNotIn("--target", edits[-1])

    def test_the_jetbrains_repo_points_each_channel_at_its_plugin(self):
        out = Path(self.tmp.name)
        for channel, where in (("edge", "download/edge/"), ("stable", "latest/download/")):
            subprocess.run(["python3", ROOT / "scripts/release/jetbrains_repo.py", out, channel], check=True, capture_output=True)
            xml = (out / "updatePlugins.xml").read_text()
            self.assertIn(f'id="dev.cplang.ide" url="https://github.com/ryanbai1412/cp-lang/releases/{where}cp-lang-jetbrains.zip"', xml)
            self.assertIn("<idea-version since-build=", xml)


if __name__ == "__main__":
    unittest.main()

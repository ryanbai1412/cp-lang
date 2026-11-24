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


SHIP_GH = """#!/bin/sh
case "$1 $2" in
  "run list") cat "$FAKE_CI/runs" ;;
  "run view") cat "$FAKE_CI/jobs-$3" ;;
  "api repos/"*) r="${2##*/}"; cat "$FAKE_CI/compare-$r" 2>/dev/null || echo behind ;;
  *) exit 2 ;;
esac
"""
CURL_ARGS = """#!/bin/sh
out=/dev/stdout w=
while [ $# -gt 0 ]; do case $1 in -o) out=$2; shift ;; -w) w=1; shift ;; esac; shift; done
"""
SHIP_CURL = CURL_ARGS + """if [ -f "$FAKE_CI/edge" ]; then printf '{"commit":"%s"}' "$(cat "$FAKE_CI/edge")" >"$out"; code=200; else code=404; fi
[ -z "$w" ] || printf %s "$code"
"""
SHIPPING = ["gate (conform:diff@1/2)", "cli (windows-latest)", "acceptance solutions stay AC (1/1)"]


class ShippableTest(unittest.TestCase):
    """scripts/release/shippable.sh (handbook/guidelines/ci-checklist.md ci/R26)."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.ci = Path(self.tmp.name)
        bin_dir = self.ci / "bin"
        bin_dir.mkdir()
        for name, body in (("gh", SHIP_GH), ("curl", SHIP_CURL)):
            (bin_dir / name).write_text(body)
            (bin_dir / name).chmod(0o755)
        self.runs = []

    def tearDown(self):
        self.tmp.cleanup()

    def run_ci(self, sha, red=(), pending=(), extra_red=()):
        """Adds a CI run (newest last) whose shipping jobs pass except `red`."""
        run_id = str(100 + len(self.runs))
        self.runs.append(f"{run_id} {sha}")
        lines = [("" if j in pending else "failure" if j in red else "success") + "\t" + j for j in SHIPPING]
        lines += [f"failure\t{j}" for j in extra_red] + ["success\tsite build"]
        (self.ci / f"jobs-{run_id}").write_text("\n".join(lines) + "\n")

    def edge_at(self, sha, newer=()):
        """Edge shipped `sha`; the commits in `newer` came after it."""
        (self.ci / "edge").write_text(sha)
        for s in [sha] + [r.split()[1] for r in self.runs]:
            if s != sha and s not in newer:
                (self.ci / f"compare-{s}...{sha}").write_text("ahead")
        (self.ci / f"compare-{sha}...{sha}").write_text("identical")

    def pick(self):
        (self.ci / "runs").write_text("\n".join(reversed(self.runs)) + "\n")
        env = dict(os.environ, GITHUB_REPOSITORY="o/r", FAKE_CI=str(self.ci),
                   PATH=f"{self.ci / 'bin'}:{os.environ['PATH']}")
        out = subprocess.run([str(ROOT / "scripts/release/shippable.sh")], env=env,
                             capture_output=True, text=True, check=True)
        return out.stdout.strip()

    def test_with_no_edge_every_shipping_job_must_pass(self):
        self.run_ci("a1")
        self.run_ci("b2", red=["cli (windows-latest)"])
        self.assertEqual(self.pick(), "a1")

    def test_a_red_outside_the_shipping_jobs_does_not_hold_edge(self):
        self.run_ci("a1", extra_red=["tests (hash)"])
        self.assertEqual(self.pick(), "a1")

    def test_a_job_already_red_at_edge_may_stay_red(self):
        self.run_ci("a1", red=["cli (windows-latest)"])
        self.run_ci("b2", red=["cli (windows-latest)"])
        self.edge_at("a1", newer=["b2"])
        self.assertEqual(self.pick(), "b2")

    def test_a_job_green_at_edge_and_red_now_blocks(self):
        self.run_ci("a1")
        self.run_ci("b2")
        self.run_ci("c3", red=["gate (conform:diff@1/2)"])
        self.edge_at("a1", newer=["b2", "c3"])
        self.assertEqual(self.pick(), "b2")

    def test_nothing_newer_than_edge_qualifies_gives_the_edge_commit(self):
        self.run_ci("a1")
        self.run_ci("b2", red=["cli (windows-latest)"])
        self.edge_at("a1", newer=["b2"])
        self.assertEqual(self.pick(), "a1")

    def test_a_run_whose_shipping_jobs_have_not_finished_is_skipped(self):
        self.run_ci("a1")
        self.run_ci("b2", pending=["acceptance solutions stay AC (1/1)"])
        self.assertEqual(self.pick(), "a1")


WATCH_GH = """#!/bin/sh
echo "$*" >>"$FAKE_CI/calls"
case "$1 $2" in
  "api repos/o/r/actions/"*) cat "$FAKE_CI/ran" ;;
  "api repos/o/r/commits/"*) cat "$FAKE_CI/live-at" ;;
  "api repos/o/r/compare/"*) echo behind ;;
  "release view") cat "$FAKE_CI/shipped" ;;
  "run list") [ ! -f "$FAKE_CI/runs" ] || cat "$FAKE_CI/runs" ;;
  "run view") printf 'success site build\\nsuccess site browser tests\\n' ;;
  "issue list") [ ! -f "$FAKE_CI/open" ] || cat "$FAKE_CI/open" ;;
  "issue "*) ;;
  *) exit 2 ;;
esac
"""
WATCH_CURL = CURL_ARGS + """if [ -f "$FAKE_CI/site-down" ]; then code=503; else printf '{"commit":"live1"}' >"$out"; code=200; fi
[ -n "$w" ] || [ "$code" = 200 ] || exit 22
[ -z "$w" ] || printf %s "$code"
"""


class WatchdogTest(unittest.TestCase):
    """scripts/ci-watchdog.sh (handbook/guidelines/ci-checklist.md ci/R23)."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.ci = Path(self.tmp.name)
        bin_dir = self.ci / "bin"
        bin_dir.mkdir()
        for name, body in (("gh", WATCH_GH), ("curl", WATCH_CURL)):
            (bin_dir / name).write_text(body)
            (bin_dir / name).chmod(0o755)

    def tearDown(self):
        self.tmp.cleanup()

    def watch(self, ran=5, shipped=30, live=30, deployable=False):
        """Runs the watchdog with ages in minutes; returns its issue calls."""
        def at(minutes):
            return subprocess.run(["date", "-u", "-d", f"-{minutes} min", "+%Y-%m-%dT%H:%M:%SZ"],
                                  capture_output=True, text=True, check=True).stdout
        (self.ci / "ran").write_text(at(ran))
        (self.ci / "shipped").write_text(at(shipped))
        (self.ci / "live-at").write_text(at(live))
        if deployable:
            (self.ci / "runs").write_text("7 new1\n")
        env = dict(os.environ, GITHUB_REPOSITORY="o/r", FAKE_CI=str(self.ci), RETRY_WAIT="0",
                   PATH=f"{self.ci / 'bin'}:{os.environ['PATH']}")
        subprocess.run([str(ROOT / "scripts/ci-watchdog.sh")], env=env, capture_output=True, text=True, check=True)
        return [c for c in (self.ci / "calls").read_text().splitlines() if c.startswith("issue ") and "list" not in c]

    def test_all_fresh_opens_nothing(self):
        self.assertEqual(self.watch(deployable=True), [])

    def test_a_stalled_queue_opens_the_issue(self):
        self.assertTrue(any("issue create" in c for c in self.watch(ran=45)))

    def test_a_stale_edge_opens_the_issue(self):
        self.assertTrue(any("issue create" in c for c in self.watch(shipped=150)))

    def test_a_site_behind_a_deployable_commit_opens_the_issue(self):
        self.assertTrue(any("issue create" in c for c in self.watch(live=150, deployable=True)))
        self.assertIn("cp-lang.org serves live1", (self.ci / "calls").read_text())

    def test_an_old_live_site_with_nothing_newer_to_deploy_is_fine(self):
        self.assertEqual(self.watch(live=150), [])

    def test_an_unreachable_site_opens_the_issue(self):
        (self.ci / "site-down").write_text("")
        self.assertTrue(any("issue create" in c for c in self.watch(deployable=True)))
        self.assertIn("site-deployable.sh failed", (self.ci / "calls").read_text())

    def test_recovery_closes_the_open_issue(self):
        (self.ci / "open").write_text("12\n")
        self.assertTrue(any(c.startswith("issue close") for c in self.watch()))


if __name__ == "__main__":
    unittest.main()

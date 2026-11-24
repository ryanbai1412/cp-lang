"""Workflow invariants: every job has a time cap (ci/R28), and blocking jobs
run on Linux, macOS and Windows (ci/R29), and only a green job marks a cached
target/ as built at HEAD (ci/R32), and releases are signed (ci/R33), and the tests compile once per run (ci/R37); handbook/guidelines/ci-checklist.md."""
import glob
import os
import re
import subprocess
import tempfile
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def uncapped_jobs(text):
    """Jobs under `jobs:` with neither `timeout-minutes:` nor a reusable `uses:`."""
    out, job, capped, in_jobs = [], None, False, False
    for line in text.split("\n") + ["end:"]:
        top = re.match(r"^\S", line)
        name = in_jobs and re.match(r"^  ([\w-]+):\s*(#.*)?$", line)
        if (top or name) and job and not capped:
            out.append(job)
        if top:
            job, in_jobs = None, line.startswith("jobs:")
        elif name:
            job, capped = name.group(1), False
        elif job and re.match(r"^    (timeout-minutes|uses):", line):
            capped = True
    return out


def job_block(text, job):
    m = re.search(rf"^  {re.escape(job)}:\s*\n((?:    .*\n|\s*\n)*)", text, re.M)
    return m.group(1) if m else ""



def fake_curl(body):
    """A `curl` that answers 200 with `body`, to stdout or to `-o FILE` with
    `-w '%{http_code}'` as scripts/retry.sh's fetch_optional asks."""
    return ('out=\nwhile [ $# -gt 0 ]; do case $1 in -o) out=$2; shift 2 ;; -w) shift 2 ;; *) shift ;; esac; done\n'
            f"if [ -n \"$out\" ]; then printf '%s\\n' '{body}' >\"$out\"; printf 200; else printf '%s\\n' '{body}'; fi")

class WorkflowTest(unittest.TestCase):
    def test_only_a_green_job_stamps_the_cached_target(self):
        with open(os.path.join(ROOT, ".github", "workflows", "ci.yml"), encoding="utf-8") as f:
            stamps = [l for l in f if "> target/.ci-sha" in l]
        self.assertTrue(stamps)
        for line in stamps:
            self.assertIn('"${{ job.status }}" != success ] ||', line,
                          "a red job's target/ may miss rebuilt crates; keep the restored .ci-sha")

    def test_every_release_is_signed(self):
        with open(os.path.join(ROOT, ".github", "workflows", "release.yml"), encoding="utf-8") as f:
            release = f.read()
        sign = job_block(release, "sign")
        self.assertIn("needs: [plan, publish]", sign)
        self.assertIn("uses: actions/attest-build-provenance@", sign)
        self.assertIn("subject-path: dist/*", sign)
        self.assertIn("id-token: write", sign)

    def test_the_conform_corpus_part_caches_generated_tests(self):
        with open(os.path.join(ROOT, ".github", "workflows", "ci.yml"), encoding="utf-8") as f:
            ci = f.read()
        # Every conform part reads the cases; the corpus part saves them even when red.
        self.assertRegex(ci, r"cache/restore@v4\n\s+if: startsWith\(matrix.part, 'conform:'\)\n\s+with:\n\s+path: target/corpus-tests")
        self.assertRegex(ci, r"cache/save@v4\n\s+if: \$\{\{ always\(\) && matrix.part == 'conform:corpus\+docs' \}\}\n\s+with:\n\s+path: target/corpus-tests")

    def test_site_selection_skips_superseded_runs(self):
        # Superseded ci runs end `cancelled` ~100 times an hour and ran no job;
        # scanning each one spent the repo's API rate limit (ci/R25).
        with open(os.path.join(ROOT, ".github", "workflows", "site.yml"), encoding="utf-8") as f:
            self.assertIn("github.event.workflow_run.conclusion != 'cancelled'", job_block(f.read(), "select"))
        with open(os.path.join(ROOT, "scripts", "site-deployable.sh"), encoding="utf-8") as f:
            deployable = f.read()
        self.assertIn('.conclusion != "cancelled"', deployable)
        self.assertIn('[ "$sha" != "$live" ]', deployable)

    def test_site_selection_stops_at_runs_older_than_the_live_build(self):
        # The live commit's own run is usually cancelled, so it never matches
        # `$live`; without the time stop every listed run cost a `gh run view`
        # and the selection hit the API rate limit (ci/R25).
        with tempfile.TemporaryDirectory() as d:
            calls = os.path.join(d, "calls")
            for name, body in (
                ("curl", fake_curl('{"commit":"live","built":"2026-11-23T16:15:00Z"}')),
                ("gh", f'echo "$*" >>{calls}\n[ "$2" = list ] && printf "%s\\n" '
                       '"2 new 2026-11-23T17:00:00Z" "1 old 2026-11-23T16:00:00Z" "0 older 2026-11-23T15:00:00Z"\ntrue'),
            ):
                path = os.path.join(d, name)
                with open(path, "w", encoding="utf-8") as f:
                    f.write("#!/usr/bin/env bash\n" + body + "\n")
                os.chmod(path, 0o755)
            env = {**os.environ, "PATH": d + os.pathsep + os.environ["PATH"], "GITHUB_REPOSITORY": "o/r"}
            env.pop("GITHUB_OUTPUT", None)
            out = subprocess.run(["bash", os.path.join(ROOT, "scripts", "site-deployable.sh")],
                                 env=env, capture_output=True, text=True)
            self.assertEqual(out.returncode, 0, out.stderr)
            self.assertEqual(out.stdout, "")
            with open(calls, encoding="utf-8") as f:
                views = [l for l in f if l.startswith("run view")]
            self.assertEqual(len(views), 1, "only the run newer than the live build is inspected")

    def test_the_site_deploys_only_when_every_browser_part_passed(self):
        # The browser tests run as matrix parts; one failing part must hold
        # the deploy as the single job did.
        with tempfile.TemporaryDirectory() as d:
            for name, body in (
                ("curl", fake_curl('{"commit":"live","built":"2026-11-23T16:15:00Z"}')),
                ("gh", '[ "$2" = list ] && printf "%s\\n" "2 new 2026-11-23T18:00:00Z" "1 old 2026-11-23T17:00:00Z"\n'
                       '[ "$2" = view ] && [ "$3" = 2 ] && printf "%s\\n" "success site build" '
                       '"success site browser tests (1/2)" "failure site browser tests (2/2)"\n'
                       '[ "$2" = view ] && [ "$3" = 1 ] && printf "%s\\n" "success site build" '
                       '"success site browser tests (1/2)" "success site browser tests (2/2)"\ntrue'),
            ):
                path = os.path.join(d, name)
                with open(path, "w", encoding="utf-8") as f:
                    f.write("#!/usr/bin/env bash\n" + body + "\n")
                os.chmod(path, 0o755)
            env = {**os.environ, "PATH": d + os.pathsep + os.environ["PATH"], "GITHUB_REPOSITORY": "o/r"}
            env.pop("GITHUB_OUTPUT", None)
            out = subprocess.run(["bash", os.path.join(ROOT, "scripts", "site-deployable.sh")],
                                 env=env, capture_output=True, text=True)
            self.assertEqual((out.returncode, out.stdout), (0, "old\n"), out.stderr)

    def test_release_does_not_plan_on_superseded_runs(self):
        # Planning on each cancelled ci run spent the API rate limit that the
        # site and release selection share (ci/R26).
        with open(os.path.join(ROOT, ".github", "workflows", "release.yml"), encoding="utf-8") as f:
            plan = job_block(f.read(), "plan")
        self.assertIn("github.event.workflow_run.conclusion == 'failure'", plan)
        self.assertNotIn("== 'cancelled'", plan)

    def test_a_stale_live_site_turns_the_schedule_red(self):
        # The alert for the site falling behind main (ci/R25) never blocks a
        # deploy and costs no API call beyond the selection's.
        with open(os.path.join(ROOT, ".github", "workflows", "site.yml"), encoding="utf-8") as f:
            fresh = job_block(f.read(), "fresh")
        self.assertIn("- name: live site is current", fresh)
        self.assertIn("github.event_name == 'schedule'", fresh)
        self.assertIn("needs: [select, deploy]", fresh)
        self.assertIn("ci/R25", fresh)
        with open(os.path.join(ROOT, "scripts", "site-deployable.sh"), encoding="utf-8") as f:
            self.assertIn("updatedAt", f.read())

    def test_blocking_jobs_cover_every_os(self):
        with open(os.path.join(ROOT, ".github", "workflows", "ci.yml"), encoding="utf-8") as f:
            ci = f.read()
        # Linux runs the contest CLI in the gate, so `cli` covers the other two.
        for job, oses in (("vscode", "ubuntu macos windows"), ("cli", "macos windows")):
            block = job_block(ci, job)
            self.assertNotIn("continue-on-error", block, f"{job} must block main")
            for os_name in oses.split():
                self.assertIn(f"{os_name}-latest", block, f"{job} must run on {os_name}")
        self.assertIn("scripts/check.sh", job_block(ci, "gate"))

    def test_every_job_has_a_timeout(self):
        for path in sorted(glob.glob(os.path.join(ROOT, ".github", "workflows", "*.yml"))):
            with open(path, encoding="utf-8") as f:
                self.assertEqual(uncapped_jobs(f.read()), [],
                                 f"{os.path.relpath(path, ROOT)}: add timeout-minutes to these jobs")

    def test_a_hung_gate_test_fails_by_name_before_the_job_cap(self):
        with open(os.path.join(ROOT, ".config", "nextest.toml"), encoding="utf-8") as f:
            gate = re.search(r"^\[profile\.gate\]\n(?:[^\[].*\n?)*", f.read(), re.M)
        self.assertIsNotNone(gate, "nextest.toml needs a [profile.gate]")
        m = re.search(r'slow-timeout = \{ period = "(\d+)s", terminate-after = (\d+) \}', gate.group(0))
        self.assertIsNotNone(m, "[profile.gate] needs a slow-timeout with terminate-after")
        with open(os.path.join(ROOT, "scripts", "check.sh"), encoding="utf-8") as f:
            self.assertRegex(f.read(), r"nextest run [^\n]*--profile gate [^\n]*--retries 1 --flaky-result fail --partition")
        with open(os.path.join(ROOT, ".github", "workflows", "ci.yml"), encoding="utf-8") as f:
            cap = int(re.search(r"^    timeout-minutes: (\d+)", job_block(f.read(), "gate"), re.M).group(1))
        # Both tries of a hung test end with minutes left for the build.
        self.assertLess(2 * int(m.group(1)) * int(m.group(2)), (cap - 5) * 60)

    def test_the_workspace_tests_compile_once(self):
        with open(os.path.join(ROOT, ".github", "workflows", "ci.yml"), encoding="utf-8") as f:
            ci = f.read()
        self.assertIn("cargo nextest archive --workspace --archive-file nextest.tar.zst", job_block(ci, "tests-build"))
        shards = job_block(ci, "tests")
        self.assertIn("needs: tests-build", shards)
        self.assertIn("CPL_NEXTEST_ARCHIVE: nextest.tar.zst", shards)
        for compiles in ("Restore target/", "rust-cache", "cargo build", "cargo test"):
            self.assertNotIn(compiles, shards, f"tests shards run the archive, never compile (ci/R37): {compiles}")
        self.assertNotIn("tests:hash", job_block(ci, "gate"), "the gate must not rebuild the tests")
        with open(os.path.join(ROOT, "scripts", "check.sh"), encoding="utf-8") as f:
            self.assertIn('--archive-file "$CPL_NEXTEST_ARCHIVE"', f.read())

    def test_no_ci_job_fails_without_turning_main_red(self):
        with open(os.path.join(ROOT, ".github", "workflows", "ci.yml"), encoding="utf-8") as f:
            self.assertNotIn("continue-on-error", f.read(),
                             "a red nobody sees is not a check: block, or report from the watchdog (ci/R29)")

    def test_every_workflow_sets_rust_up_through_one_action(self):
        wf = os.path.join(ROOT, ".github", "workflows")
        for name in sorted(os.listdir(wf)):
            with open(os.path.join(wf, name), encoding="utf-8") as f:
                text = f.read()
            for own in ("rustup toolchain install", "Swatinem/rust-cache", "scripts/ci-mtime.sh"):
                self.assertNotIn(own, text, f"{name}: use ./.github/actions/setup-rust (ci/R38)")
            for key in re.findall(r"TARGET_KEY: (\S+)", text):
                self.assertEqual(key, "target-ci-v2",
                                 f"{name}: restores a target/ ci.yml never saves (ci/R38)")

    def test_finds_an_uncapped_job(self):
        text = "on: push\njobs:\n  a:\n    timeout-minutes: 5\n    steps: []\n  b:\n    runs-on: x\n    steps:\n      - timeout-minutes: 3\n"
        self.assertEqual(uncapped_jobs(text), ["b"])


if __name__ == "__main__":
    unittest.main()

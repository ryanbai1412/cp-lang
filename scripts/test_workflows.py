"""Workflow invariants: every job has a time cap (ci/R28), and blocking jobs
run on Linux, macOS and Windows (ci/R29), and only a green job marks a cached
target/ as built at HEAD (ci/R32), and releases are signed (ci/R33); handbook/guidelines/ci-checklist.md."""
import glob
import os
import re
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

    def test_finds_an_uncapped_job(self):
        text = "on: push\njobs:\n  a:\n    timeout-minutes: 5\n    steps: []\n  b:\n    runs-on: x\n    steps:\n      - timeout-minutes: 3\n"
        self.assertEqual(uncapped_jobs(text), ["b"])


if __name__ == "__main__":
    unittest.main()

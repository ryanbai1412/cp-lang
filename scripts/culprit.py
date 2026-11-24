#!/usr/bin/env python3
"""Name the commit that broke each failing test, and check its revert.

    culprit.py --good SHA --bad SHA TEST...        TEST: `binary_id name`
    culprit.py --good SHA --bad SHA --from-log F   the FAIL lines of a CI log
        [--json OUT] [--push-reverts] [--in-place]

Main's CI cancels superseded runs, so a red run covers dozens of commits.
For each failing nextest test this bisects good..bad in a scratch worktree
(--in-place: in this clean checkout, as CI does), building and running only that test: a build failure
skips the commit, a missing test counts as passing. Tests that bisect to
one commit are grouped. For each culprit it reverts it on top of bad and
reruns its tests; --push-reverts pushes a revert that makes them pass to
`culprit/revert-<sha>`. A test that passes at bad is reported flaky, one
that fails at good as already failing.

Before any bisect it names each failing test's suspects in seconds: the
newest commits in good..bad that touch the test's file (else its crate),
with their lanes. A shard red for one test still routes every other test
it hides, and the report is saved after each test, so a bisect cut by the
job's timeout keeps what it found.
"""

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from domain_of import domain_of  # noqa: E402

FAIL = re.compile(r"\b(?:FAIL|TIMEOUT|SIGSEGV|SIGABRT|SIGKILL)\s+\[\s*[\d.]+s\]\s+"
                  r"(?:\(\s*\d+/\d+\)\s+)?(\S+)\s+(\S+)")


def failing_tests(log):
    """The distinct (binary_id, test) pairs named by nextest FAIL lines."""
    seen = []
    for line in log.splitlines():
        m = FAIL.search(line)
        if m and m.groups() not in seen:
            seen.append(m.groups())
    return seen


def nextest_cmd(tests):
    package = {b.split("::")[0] for b, _ in tests}
    expr = " | ".join(f"(binary_id({b}) & test(={t}))" for b, t in tests)
    cmd = ["cargo", "nextest", "run", "--no-fail-fast", "-E", expr]
    for p in sorted(package):
        cmd += ["-p", p]
    return cmd


ROOT = os.getcwd()


def git(*args, cwd, check=True):
    return subprocess.run(["git", *args], cwd=cwd, check=check, text=True,
                          capture_output=True).stdout.strip()


def run_tests(tests, cwd):
    """'pass', 'fail' or 'skip' (did not build) for `tests` at cwd's HEAD."""
    out = subprocess.run(nextest_cmd(tests), cwd=cwd, capture_output=True, text=True)
    if out.returncode == 0 or out.returncode == 4:  # 4: no such test here
        return "pass"
    return "fail" if out.returncode == 100 else "skip"


def bisect(test, good, bad, cwd):
    fd, script = tempfile.mkstemp(prefix="culprit-step-", suffix=".sh")
    os.close(fd)
    cmd = "cd " + shlex.quote(cwd) + " && " + " ".join(shlex.quote(a) for a in nextest_cmd([test]))
    with open(script, "w") as f:
        f.write("#!/bin/sh\n" + cmd + " >/dev/null 2>&1\n"
                "c=$?\n[ $c -eq 0 ] || [ $c -eq 4 ] && exit 0\n[ $c -eq 100 ] && exit 1\nexit 125\n")
    os.chmod(script, 0o755)
    git("bisect", "start", bad, good, cwd=cwd)
    try:
        out = subprocess.run(["git", "bisect", "run", script], cwd=cwd,
                             stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True).stdout
        if re.search(r"first '?bad'? commit", out):
            return git("rev-parse", "refs/bisect/bad", cwd=cwd)
        sys.stderr.write(f"culprit: bisect of {test[1]} found nothing:\n"
                         + "\n".join(out.splitlines()[-15:]) + "\n")
        return None
    finally:
        git("bisect", "reset", cwd=cwd, check=False)
        os.remove(script)


def describe(sha, cwd):
    subject = git("log", "-1", "--format=%s", sha, cwd=cwd)
    domain = git("log", "-1", "--format=%(trailers:key=Domain,valueonly)", sha, cwd=cwd)
    if not domain:
        paths = git("show", "--format=", "--name-only", sha, cwd=cwd).splitlines()
        domain = domain_of(paths) or ""
    return subject, domain.strip()


def try_revert(sha, tests, bad, cwd, push):
    git("checkout", "-q", "--detach", bad, cwd=cwd)
    r = subprocess.run(["git", "revert", "--no-edit", sha], cwd=cwd,
                       capture_output=True, text=True)
    if r.returncode != 0:
        git("revert", "--abort", cwd=cwd, check=False)
        return {"clean": False}
    result = {"clean": True, "passes": run_tests(tests, cwd) == "pass"}
    if push and result["passes"]:
        branch = f"culprit/revert-{sha[:9]}"
        subprocess.run(["git", "push", "-f", "origin", f"HEAD:refs/heads/{branch}"],
                       cwd=cwd, check=True)
        result["branch"] = branch
    return result


def source_of(test, cwd):
    """The tracked file defining `test`: its integration-test source, or the
    crate file with its `fn`; None when not found."""
    binary, name = test
    crate, _, target = binary.partition("::")
    if target:
        path = f"crates/{crate}/tests/{target}.rs"
        if git("ls-files", path, cwd=cwd):
            return path
    fn = name.split("::")[-1]
    hits = git("grep", "-l", "-E", rf"fn {re.escape(fn)}\b", "--", f"crates/{crate}", cwd=cwd, check=False)
    return hits.splitlines()[0] if hits else None


def triage(good, bad, tests, cwd, keep=3):
    """Each test's suspects: the newest `keep` commits in good..bad touching
    its file, else its crate, each with its lane."""
    out = []
    for test in tests:
        path = source_of(test, cwd)
        crate = f"crates/{test[0].split('::')[0]}"
        shas = []
        for where in [p for p in (path, crate) if p]:
            shas = git("log", "--format=%H", f"-{keep}", f"{good}..{bad}", "--", where, cwd=cwd).split()
            if shas:
                break
        suspects = []
        for sha in shas:
            subject, domain = describe(sha, cwd)
            suspects.append({"sha": sha, "subject": subject, "domain": domain})
        out.append({"test": " ".join(test), "file": path, "suspects": suspects})
    return out


def find(good, bad, tests, push=False, in_place=False, report=None, save=lambda r: None):
    report = report or {"good": good, "bad": bad}
    for key in ("culprits", "flaky", "already_failing", "unbisected"):
        report.setdefault(key, [])
    if in_place:
        wt, home = ROOT, git("rev-parse", "HEAD", cwd=ROOT)
    else:
        # Its own target/: a dev build bakes its checkout's std path in.
        wt = tempfile.mkdtemp(prefix="culprit-")
        git("worktree", "add", "--detach", wt, bad, cwd=ROOT)
    try:
        found = {}
        for test in tests:
            name = " ".join(test)
            git("checkout", "-q", "--detach", bad, cwd=wt)
            save(report)
            if run_tests([test], wt) != "fail":
                report["flaky"].append(name)
                continue
            known = next((sha for sha in found
                          if explains(sha, test, wt)), None)
            if known:
                found[known].append(test)
                continue
            git("checkout", "-q", "--detach", good, cwd=wt)
            if run_tests([test], wt) == "fail":
                report["already_failing"].append(name)
                continue
            sha = bisect(test, good, bad, wt)
            if sha:
                found.setdefault(sha, []).append(test)
            else:
                report["unbisected"].append(name)
        for sha, its in found.items():
            subject, domain = describe(sha, wt)
            report["culprits"].append({
                "sha": sha, "subject": subject, "domain": domain,
                "tests": [" ".join(t) for t in its],
                "revert": try_revert(sha, its, bad, wt, push)})
            save(report)
    finally:
        if in_place:
            git("checkout", "-q", "--detach", home, cwd=ROOT, check=False)
        else:
            git("worktree", "remove", "--force", wt, cwd=ROOT, check=False)
    return report


def explains(sha, test, cwd):
    """Whether `test` passes just before `sha` and fails at it."""
    git("checkout", "-q", "--detach", sha, cwd=cwd)
    if run_tests([test], cwd) != "fail":
        return False
    git("checkout", "-q", "--detach", f"{sha}^", cwd=cwd)
    return run_tests([test], cwd) == "pass"


def summary(report):
    lines = [f"suspects per failing test in {report['good'][:9]}..{report['bad'][:9]}:"]
    for t in report.get("triage", []):
        who = ", ".join(f"{s['sha'][:9]} ({s['domain'] or 'no domain'})" for s in t["suspects"])
        lines.append(f"- {t['test']} [{t['file'] or 'file not found'}]: {who or 'no commit touches it'}")
    lines.append(f"culprits in {report['good'][:9]}..{report['bad'][:9]}:")
    for c in report.get("culprits", []):
        rv = c["revert"]
        state = ("revert conflicts" if not rv["clean"] else
                 f"revert passes, on {rv['branch']}" if rv.get("branch") else
                 "revert passes" if rv["passes"] else "revert still fails")
        lines.append(f"- {c['sha'][:9]} ({c['domain'] or 'no domain'}) {c['subject']}: "
                     f"{len(c['tests'])} test(s), {state}")
        lines += [f"    {t}" for t in c["tests"]]
    for key in ("flaky", "already_failing", "unbisected"):
        if report.get(key):
            lines.append(f"{key.replace('_', ' ')}: " + ", ".join(report[key]))
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--good")
    ap.add_argument("--bad")
    ap.add_argument("--summary", metavar="JSON", help="print a saved report's summary")
    ap.add_argument("--triage-only", action="store_true", help="name suspects; bisect nothing")
    ap.add_argument("--from-log")
    ap.add_argument("--json")
    ap.add_argument("--push-reverts", action="store_true")
    ap.add_argument("--in-place", action="store_true", help="bisect this clean checkout, not a worktree")
    ap.add_argument("tests", nargs="*", help="`binary_id name`, e.g. 'cpl-cli::diagnostics golden'")
    args = ap.parse_args()
    if args.summary:
        with open(args.summary) as f:
            print(summary(json.load(f)))
        return
    if not (args.good and args.bad):
        ap.error("--good and --bad are required")
    tests = [tuple(t.split(None, 1)) for t in args.tests]
    if args.from_log:
        with open(args.from_log, errors="replace") as f:
            tests += [t for t in failing_tests(f.read()) if t not in tests]
    if not tests:
        print("culprit: no failing nextest tests to bisect")
        return
    global ROOT
    ROOT = git("rev-parse", "--show-toplevel", cwd=os.getcwd())
    good = git("rev-parse", args.good, cwd=ROOT)
    bad = git("rev-parse", args.bad, cwd=ROOT)
    def save(report):
        if args.json:
            with open(args.json, "w") as f:
                json.dump(report, f, indent=2)

    report = {"good": good, "bad": bad, "triage": triage(good, bad, tests, ROOT)}
    save(report)
    print(summary(report), flush=True)
    if args.triage_only:
        return
    report = find(good, bad, tests, args.push_reverts, args.in_place, report, save)
    save(report)
    print(summary(report))


if __name__ == "__main__":
    main()

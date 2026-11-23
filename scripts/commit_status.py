"""Mark every main commit a CI run covered with a `ci/covered` commit status.

Main CI tests only the newest tip when a run frees up, so most commits never
get a run of their own (GitHub shows them "cancelled"). After each finished
run this gives every commit since the previous finished run a status:

    green run: each commit   success  "passed, covered by run N"
    red run:   the tip       failure  "run N failed here"
               the others    pending  "untested: run N failed at <tip>"
    culprit named:           failure  "culprit of run N: <tests>"

Usage:
    python3 scripts/commit_status.py --run N --conclusion success|failure \\
        --head SHA [--prev SHA] [--url URL] [--dry-run]
    python3 scripts/commit_status.py --run N --culprits culprit.json [--url URL]
"""
import argparse
import json
import os
import subprocess

CONTEXT = "ci/covered"
LIMIT = 400


def statuses(commits, run, conclusion, url):
    """[(sha, state, description)] for `commits`, newest first."""
    if not commits:
        return []
    head = commits[0]
    if conclusion == "success":
        return [(c, "success", f"passed, covered by run {run}") for c in commits]
    out = [(head, "failure", f"run {run} failed here")]
    out += [(c, "pending", f"untested: run {run} failed at {head[:9]}") for c in commits[1:]]
    return out


def culprit_statuses(report, run):
    out = []
    for c in report.get("culprits", []):
        tests = ", ".join(t.split()[-1] for t in c["tests"])
        out.append((c["sha"], "failure", f"culprit of run {run}: {tests}"[:140]))
    return out


def commits(prev, head):
    spec = f"{prev}..{head}" if prev else head
    out = subprocess.run(["git", "rev-list", "--first-parent", f"--max-count={LIMIT}", spec],
                         check=True, capture_output=True, text=True).stdout
    return out.split()


def post(sha, state, desc, url):
    body = {"state": state, "context": CONTEXT, "description": desc[:140]}
    if url:
        body["target_url"] = url
    repo = os.environ["GITHUB_REPOSITORY"]
    subprocess.run(["gh", "api", "-X", "POST", f"repos/{repo}/statuses/{sha}", "--input", "-"],
                   input=json.dumps(body), check=True, text=True, stdout=subprocess.DEVNULL)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--run", required=True)
    ap.add_argument("--conclusion")
    ap.add_argument("--head")
    ap.add_argument("--prev")
    ap.add_argument("--culprits")
    ap.add_argument("--url")
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args()
    if a.culprits:
        with open(a.culprits) as f:
            todo = culprit_statuses(json.load(f), a.run)
    else:
        todo = statuses(commits(a.prev, a.head), a.run, a.conclusion, a.url)
    for sha, state, desc in todo:
        print(f"{sha[:9]} {state:7} {desc}")
        if not a.dry_run:
            post(sha, state, desc, a.url)


if __name__ == "__main__":
    main()

"""Force-cancel a main CI run that GitHub left hanging after its work ended.

A hosted job can finish every step and then sit in GitHub's own "Complete
job" step for good; a plain cancel does not end it, and the run holds main's
queue slot (handbook/process/landing-and-ci.md#the-queue). A run is hung
when every job has ended or has finished all its steps but "Complete job",
and the last step ended over STUCK_AFTER ago. Such a run is force-cancelled
and its tip given the `ci/covered` status "red: runner hang, covered by run
N", N being the run waiting behind it, which tests a tree containing it and
overwrites that status when it finishes. A run past LONG of wall time that
is still working is only reported: cancelling it would leave its tip's gate
unrun.

Usage: uv run python scripts/ci_unstick.py [--dry-run]
"""
import argparse
import json
import os
import subprocess
from datetime import datetime, timezone

STUCK_AFTER = 3 * 60
LONG = 15 * 60
CONTEXT = "ci/covered"


def at(stamp):
    return datetime.fromisoformat(stamp.replace("Z", "+00:00")).timestamp() if stamp else None


def hung_since(jobs):
    """When the run's last step ended, if every job has done all its work and
    at least one still waits in "Complete job"; else None."""
    ended, waiting = [], False
    for job in jobs:
        if job["status"] == "completed":
            ended.append(at(job["completed_at"]))
            continue
        steps = [s for s in job.get("steps", []) if s["name"] != "Complete job"]
        if job["status"] != "in_progress" or not steps or any(s["status"] != "completed" for s in steps):
            return None
        waiting = True
        ended += [at(s["completed_at"]) for s in steps]
    ended = [t for t in ended if t is not None]
    return max(ended) if waiting and ended else None


def verdict(run, jobs, now):
    """("hung", seconds stuck), ("long", seconds of wall time) or None."""
    since = hung_since(jobs)
    if since is not None and now - since > STUCK_AFTER:
        return ("hung", int(now - since))
    wall = now - at(run["run_started_at"])
    if wall > LONG:
        return ("long", int(wall))
    return None


def gh(*args, body=None):
    out = subprocess.run(["gh", "api", *args] + (["--input", "-"] if body else []),
                         input=json.dumps(body) if body else None,
                         check=True, capture_output=True, text=True).stdout
    return json.loads(out) if out.strip() else None


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args()
    repo = os.environ["GITHUB_REPOSITORY"]
    runs = f"repos/{repo}/actions/workflows/ci.yml/runs?branch=main&event=push&per_page=5"
    listed = gh(f"{runs}&status=in_progress")["workflow_runs"]
    waiting = gh(f"{runs}&status=pending")["workflow_runs"] + gh(f"{runs}&status=queued")["workflow_runs"]
    now = datetime.now(timezone.utc).timestamp()
    for run in listed:
        jobs = gh(f"repos/{repo}/actions/runs/{run['id']}/jobs?per_page=100")["jobs"]
        v = verdict(run, jobs, now)
        if v is None:
            print(f"ci-unstick: run {run['id']} is working")
            continue
        kind, secs = v
        if kind == "long":
            print(f"ci-unstick: run {run['id']} is still working after {secs // 60} min; left to finish")
            continue
        nxt = f"run {waiting[0]['id']}" if waiting else "the next run"
        desc = f"red: runner hang, covered by {nxt}"
        print(f"ci-unstick: run {run['id']} ({run['head_sha'][:9]}) hung {secs}s after its last step; "
              f"force-cancelling: {desc}")
        if a.dry_run:
            continue
        gh("-X", "POST", f"repos/{repo}/actions/runs/{run['id']}/force-cancel")
        gh("-X", "POST", f"repos/{repo}/statuses/{run['head_sha']}",
           body={"state": "failure", "context": CONTEXT, "description": desc, "target_url": run["html_url"]})


if __name__ == "__main__":
    main()

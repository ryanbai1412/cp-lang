#!/usr/bin/env python3
"""The hourly migration report: one Markdown summary of the last hour, from cached data.

    uv run --quiet python scripts/hourly.py [--hours H] [--ref origin/main]

Counters come from scripts/debt-ratchet.json at the ref now and an hour ago; conformance
from the two newest fresh full runs scripts/fresh-run.sh kept under target/cpl-next/fresh/;
main CI from `gh run list`; landing phases from land.sh's log (.git/cpl-land.log). Nothing
here runs a case or a counter, so it takes seconds. plan/STATUS.md stays scripts/status.py's.
"""

import importlib.util
import json
import os
import statistics
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FRESH = ROOT / "target" / "cpl-next" / "fresh"
AREAS = (("cpl-next*", "crates/cpl-next"), ("std", "std/"), ("spec", "spec/"))


def git(*args):
    r = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    return r.stdout.strip() if r.returncode == 0 else ""


def module(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / file)
    mod = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(mod)
    return mod


def ratchet_at(rev):
    text = git("show", f"{rev}:scripts/debt-ratchet.json") if rev else ""
    return json.loads(text) if text else {}


def gated_next(ratchet_mod):
    return sorted(n for n, (_, _, gated) in ratchet_mod.COUNTERS.items() if gated and n.startswith("next_"))


def counters(names, now, then):
    rows = [(n, now.get(n, 0), now.get(n, 0) - then.get(n, now.get(n, 0))) for n in names]
    total, before = sum(r[1] for r in rows), sum(then.get(n, now.get(n, 0)) for n in names)
    out = [f"**Gated next_\\* sum:** {total:,} ({total - before:+,} this window; "
           f"{sum(1 for r in rows if r[1])} of {len(rows)} above 0)", "",
           "| counter | value | change |", "|---|---:|---:|"]
    out += [f"| `{n}` | {v:,} | {d:+,} |" for n, v, d in sorted(rows, key=lambda r: (-abs(r[2]), -r[1])) if v or d]
    return out, total, before


def churn(ref, since):
    out = ["| area | commits | +lines | -lines |", "|---|---:|---:|---:|"]
    for label, prefix in AREAS:
        log = git("log", "--no-merges", f"--since={since}", "--format=@%H", "--numstat", ref, "--", f"{prefix}*" if not prefix.endswith("/") else prefix)
        commits = sum(1 for l in log.splitlines() if l.startswith("@"))
        add = rem = 0
        for l in log.splitlines():
            parts = l.split("\t")
            if len(parts) == 3 and parts[0].isdigit() and parts[1].isdigit():
                add, rem = add + int(parts[0]), rem + int(parts[1])
        out.append(f"| {label} | {commits} | {add:,} | {rem:,} |")
    return out


def run_at(d):
    def lines(name):
        p = d / name
        return [l for l in p.read_text(encoding="utf-8").splitlines() if l.strip()] if p.exists() else []
    wrong = [l.split("\t") for l in lines("wrong.tsv")]
    kinds = {}
    for w in wrong:
        kinds[w[1] if len(w) > 1 else "?"] = kinds.get(w[1] if len(w) > 1 else "?", 0) + 1
    return {"sha": d.name, "at": (d / "stamp").stat().st_mtime if (d / "stamp").exists() else d.stat().st_mtime,
            "passing": set(lines("passing.txt")), "wrong": {w[0]: w[1] if len(w) > 1 else "?" for w in wrong},
            "kinds": kinds, "refused": len(lines("refused.tsv")), "error_typed": len(lines("error-typed.txt"))}


def conformance(hours):
    runs = sorted((d for d in FRESH.glob("*") if d.is_dir()), key=lambda d: d.stat().st_mtime) if FRESH.exists() else []
    if not runs:
        return ["No fresh full run kept yet (scripts/fresh-run.sh)."], 0
    cur = run_at(runs[-1])
    k = cur["kinds"]
    age = (time.time() - cur["at"]) / 60
    out = [f"Fresh full run on `{cur['sha']}` ({age:.0f} min ago): {len(cur['passing']):,} pass, "
           f"{len(cur['wrong']):,} wrong ({k.get('wrong-accept', 0)} wrong-accept, {k.get('wrong-output', 0)} wrong output, "
           f"{k.get('split', 0)} backend disagreements), {cur['error_typed']} error-typed, {cur['refused']:,} refused."]
    if len(runs) < 2:
        return out, 0
    prev = run_at(runs[-2])
    lost = sorted(prev["passing"] - cur["passing"])
    span = max((cur["at"] - prev["at"]) / 3600, 1 / 60)
    commits = git("rev-list", "--count", f"{prev['sha']}..{cur['sha']}") or "?"
    out.append(f"Pass to fail since `{prev['sha']}`: {len(lost)} ({len(lost) / span:.1f}/h), in "
               f"{commits} commits `{prev['sha']}..{cur['sha']}`")
    for case in lost[:40]:
        now = cur["wrong"].get(case, "refused")
        culprit = git("log", "-1", "--format=%h %s", f"{prev['sha']}..{cur['sha']}", "--", f"spec/**/{case}") or "range above"
        out.append(f"- `{case}` → {now} ({culprit})")
    if len(lost) > 40:
        out.append(f"- and {len(lost) - 40} more")
    return out, len(lost) / span


def ci(hours):
    r = subprocess.run(["gh", "run", "list", "--workflow", "ci.yml", "--branch", "main", "-L", "200", "--json",
                        "headSha,status,conclusion,createdAt,updatedAt"], cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        return [f"Main CI: unknown ({r.stderr.strip()[:80]})"]
    def t(s):
        return datetime.fromisoformat(s.replace("Z", "+00:00")).timestamp()
    done = sorted(((t(x["updatedAt"]), x["conclusion"], x["headSha"][:9]) for x in json.loads(r.stdout)
                   if x["status"] == "completed" and x["conclusion"] in ("success", "failure")))
    if not done:
        return ["Main CI: no completed verdict in the last 200 runs."]
    now, start = time.time(), time.time() - hours * 3600
    red, state, since = 0.0, None, start
    for at, conclusion, _ in done:
        if at > start:
            red += (min(at, now) - since) if state == "failure" else 0
            since = at
        state = conclusion
    red += (now - since) if state == "failure" else 0
    last_green = next((at for at, c, _ in reversed(done) if c == "success"), None)
    streak = f"red since {datetime.fromtimestamp(last_green, timezone.utc):%H:%M} UTC" if state == "failure" and last_green else (
        "red for the whole kept history" if state == "failure" else "green")
    at, conclusion, sha = done[-1]
    return [f"Main CI: **{'green' if conclusion == 'success' else 'red'}** at `{sha}` "
            f"({(now - at) / 60:.0f} min ago), {streak}; {red / 60:.0f} of the last {hours * 60:.0f} min red."]


def end_state():
    es = module("end_state", "end-state.py")
    suites, counts, _ = es.load()
    states = [es.state(s, counts)[0] for s in suites]
    return [f"End-state suites: {sum(s != 'unmeasured' for s in states)} of {len(states)} measured, "
            f"{states.count('done')} meet their target."]


def landings(hours):
    git_dir = Path(git("rev-parse", "--git-common-dir") or ".git")
    log = (ROOT / git_dir if not git_dir.is_absolute() else git_dir) / "cpl-land.log"
    start = time.time() - hours * 3600
    rows = []
    for l in (log.read_text(encoding="utf-8").splitlines() if log.exists() else []):
        try:
            row = json.loads(l)
        except ValueError:
            continue
        if row.get("landed", 0) >= start:
            rows.append(row)
    if not rows:
        return ["No landings from this clone in the window."]
    phases = sorted({p for r in rows for p in r.get("phases", {})})
    med = lambda xs: statistics.median(xs) if xs else 0
    out = [f"Landings from this clone: {len(rows)}, median {med([r['seconds'] for r in rows]):.0f} s, "
           f"{sum(r.get('tries', 1) - 1 for r in rows)} failed pushes retried.", "",
           "| phase | median s | max s |", "|---|---:|---:|"]
    for p in phases:
        xs = [r["phases"][p] for r in rows if p in r.get("phases", {})]
        out.append(f"| {p} | {med(xs):.0f} | {max(xs)} |")
    return out


def lane_ratchet():
    """Main's strict --check on lane-j-check, as its last pushed gate counted it (debt-ratchet.py's
    lane mode keeps it): 0 before the slice-1 switch (coordinator 02:38)."""
    git_dir = Path(git("rev-parse", "--git-common-dir") or ".git")
    path = (ROOT / git_dir if not git_dir.is_absolute() else git_dir) / "cpl-lane-ratchet.json"
    try:
        row = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return ["lane-j-check's full ratchet: no lane push from this clone has counted it yet."]
    at = datetime.fromtimestamp(row["at"], timezone.utc)
    return [f"lane-j-check's full ratchet (main's --check): {row['errors']} errors at `{row['sha'][:9]}` "
            f"({at:%Y-%m-%d %H:%M} UTC); the slice-1 switch needs 0."]


def forecast(names, total, before, hours, ref):
    rate = (before - total) / hours
    day_rev = git("rev-list", "-1", f"--before={int(time.time() - 6 * 3600)}", ref)
    day = ratchet_at(day_rev)
    six = None
    if day:
        six = (sum(day.get(n, 0) for n in names if isinstance(day.get(n), int)) - total) / 6
    def eta(r):
        return f"falling {r:.0f}/h, 0 in {total / r:.1f} h" if r and r > 0 else f"rising {-r:.0f}/h, no forecast"
    return [f"Switch forecast: gated next_\\* sum {total:,}; this window {eta(rate)}"
            + (f"; last 6 h {eta(six)}." if six is not None else ".")
            + " A counter whose rule changed moves the sum without any code paying debt."]


def main(argv):
    hours = float(argv[argv.index("--hours") + 1]) if "--hours" in argv else 1.0
    ref = argv[argv.index("--ref") + 1] if "--ref" in argv else "origin/main"
    since = int(time.time() - hours * 3600)
    then_rev = git("rev-list", "-1", f"--before={since}", ref)
    dr = module("debt_ratchet", "debt-ratchet.py")
    names = gated_next(dr)
    rows, total, before = counters(names, ratchet_at(ref), ratchet_at(then_rev))
    conf, per_hour = conformance(hours)
    out = [f"# Migration status, last {hours:g} h (to `{git('rev-parse', '--short=9', ref)}`, "
           f"{datetime.now(timezone.utc):%Y-%m-%d %H:%M} UTC)", "", "## Debt counters", "", *rows,
           "", "## Commits", "", *churn(ref, since), "", "## Conformance", "", *conf,
           f"Regressions per hour: {per_hour:.1f}", "", "## Main CI", "", *ci(hours),
           "", "## End state", "", *end_state(), "", "## Landing", "", *landings(hours),
           "", "## Lane ratchet", "", *lane_ratchet(),
           "", "## Forecast", "", *forecast(names, total, before, hours, ref)]
    print("\n".join(out))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

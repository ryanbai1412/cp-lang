#!/usr/bin/env python3
"""Write plan/product/BLOCKERS.md: Milestone 1 blockers ranked across acceptance runs.

    uv run python research/acceptance/blockers.py           write it
    uv run python research/acceptance/blockers.py --check   fail if it is stale or a run is malformed

Reads `research/acceptance/runs/*.json` (format in `research/acceptance/PROTOCOL.md`).
A finding is keyed by its `id`, so the same blocker seen in several runs is one row.
The score is language quality (Ryan, 2026-11-22): rank by kind (blocker, workaround,
non-idiomatic, hesitation), then the most runs seen in, then the most minutes lost, then
the earliest scenario step. A finding is fixed when its latest report says so (`"status":
"fixed"`, a summary starting `FIXED` or `REFUTED`, or its id in the run's `fixed` list); fixed rows leave
the ranking and are listed under "Fixed".
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
RUNS = ROOT / "research" / "acceptance" / "runs"
OUT = ROOT / "plan" / "product" / "BLOCKERS.md"
STEPS = ["understand", "try", "install", "solve", "test", "submit", "compete", "design"]
KINDS = ["blocker", "workaround", "non-idiomatic", "hesitation"]
LANES = sorted(p.name for p in (ROOT / "plan" / "domains").iterdir() if p.is_dir() and p.name != "TEMPLATE")


def cell(v):
    return str(v).replace("|", "\\|").replace("\n", " ").strip()


def load():
    runs, errors = [], []
    for path in sorted(RUNS.glob("*.json")):
        name = path.relative_to(ROOT)
        try:
            run = json.loads(path.read_text())
        except json.JSONDecodeError as e:
            errors.append(f"{name}: not JSON: {e}")
            continue
        for key in ["date", "site_commit", "steps", "findings"]:
            if key not in run:
                errors.append(f"{name}: missing `{key}`")
        steps = run.get("steps", {})
        if sorted(steps) != sorted(STEPS):
            errors.append(f"{name}: `steps` must have exactly {', '.join(STEPS)}")
        for s, v in steps.items():
            if not isinstance(v.get("pass"), bool) or not isinstance(v.get("minutes"), (int, float)):
                errors.append(f"{name}: step `{s}` needs `pass` (bool) and `minutes` (number)")
        for i, f in enumerate(run.get("findings", [])):
            where = f"{name}: finding {i} (`{f.get('id', '?')}`)"
            if f.get("step") not in STEPS:
                errors.append(f"{where}: `step` must be one of {', '.join(STEPS)}")
            if f.get("kind") not in KINDS:
                errors.append(f"{where}: `kind` must be one of {', '.join(KINDS)}")
            if f.get("lane") not in LANES:
                errors.append(f"{where}: `lane` must be one of {', '.join(LANES)}")
            if not f.get("id") or not f.get("summary"):
                errors.append(f"{where}: needs `id` and `summary`")
        run["_name"] = str(name)
        runs.append(run)
    return runs, errors


def is_fixed(f):
    s = f["summary"]
    return f.get("status") == "fixed" or s.startswith(("FIXED", "REFUTED")) or (s.startswith("NOW WORKS") and not s.startswith("NOW WORKS (partly)"))


def run_base(run):
    """One run id for the agents of a run on several OSes (`2026-11-23-5-linux` -> `2026-11-23-5`)."""
    return re.sub(r"-(linux|macos|windows)$", "", Path(run["_name"]).stem)


def quality(run):
    solved = sum(1 for p in run.get("problems", []) if str(p.get("judge", "")).startswith("AC")) or 1
    n = {k: sum(1 for f in run["findings"] if f["kind"] == k and not is_fixed(f)) for k in KINDS[1:]}
    return (f"Language quality: {n['workaround']} workarounds, {n['non-idiomatic']} non-idiomatic, "
            f"{n['hesitation']} hesitations; {sum(n.values()) / solved:.1f} per accepted problem.")


def render(runs):
    rows = {}
    for run in sorted(runs, key=lambda r: (r["date"], run_base(r))):
        grp = (run["date"], run_base(run))
        for f in run["findings"]:
            r = rows.setdefault(f["id"], {"runs": set(), "minutes": 0.0, "kind": f["kind"], "step": f["step"], "p0": False})
            r["p0"] |= bool(f.get("p0"))
            # Fixed only when every OS agent of the latest run that reports it says so.
            r["fixed"] = is_fixed(f) and (r.get("grp") != grp or r["fixed"])
            r["grp"] = grp
            r.update(lane=f["lane"], summary=f["summary"], last=run["date"], evidence=f.get("evidence", ""))
            r["runs"].add(run_base(run))
            r["minutes"] += float(f.get("minutes_lost", 0))
            if KINDS.index(f["kind"]) < KINDS.index(r["kind"]):
                r["kind"] = f["kind"]
            if STEPS.index(f["step"]) < STEPS.index(r["step"]):
                r["step"] = f["step"]
        for fid in run.get("fixed", []):
            if fid in rows and (rows[fid].get("grp") != grp or rows[fid]["fixed"]):
                rows[fid].update(fixed=True, last=run["date"], grp=grp)
    fixed = sorted((kv for kv in rows.items() if kv[1]["fixed"]), key=lambda kv: (kv[1]["lane"], kv[0]))
    ranked = sorted((kv for kv in rows.items() if not kv[1]["fixed"]), key=lambda kv: (
        not kv[1]["p0"], KINDS.index(kv[1]["kind"]), -len(kv[1]["runs"]), -kv[1]["minutes"], STEPS.index(kv[1]["step"]), kv[0]))
    out = ["# Milestone 1 blockers", "",
           "Generated by `research/acceptance/blockers.py` from `research/acceptance/runs/*.json`;",
           "edit those instead. Each row is one finding from the acceptance run",
           "(`research/acceptance/PROTOCOL.md`), merged across runs by its id. The score is language",
           "quality. Rank: P0 first (a silent wrong answer or a hidden cost, marked `p0` by the judge),",
           "then kind (blocker, workaround, non-idiomatic, hesitation), then runs seen in,",
           "then minutes lost, then the earliest scenario step. Each lane's rows are also sent to its `plan/domains/<lane>/worklog.md`.", ""]
    if runs:
        latest = max(runs, key=lambda r: (r["date"], r["_name"]))
        base = run_base(latest)
        group = sorted((r for r in runs if run_base(r) == base),
                       key=lambda r: r["_name"])
        if len(group) > 1:
            out += [f"Latest run: `{base}` ({latest['date']}), one agent per OS:", ""]
            for r in group:
                passed = [s for s in STEPS if r["steps"][s]["pass"]]
                out += [f"- `{Path(r['_name']).stem}` ({r['os']}, site `{r['site_commit']}`): "
                        f"{len(passed)}/{len(STEPS)} steps pass. " + quality(r)]
            out += [""]
        else:
            passed = [s for s in STEPS if latest["steps"][s]["pass"]]
            out += [f"Latest run: `{latest['_name']}` ({latest['date']}, site `{latest['site_commit']}`): "
                    f"{len(passed)}/{len(STEPS)} steps pass.", "", quality(latest), ""]
        cis = [r for r in group if "ci" in r]
        if cis:
            ci = cis[-1]["ci"]
            out += [f"CLI on every OS (CI {ci['run']}): Linux {ci['linux']}, macOS {ci['macos']}, Windows {ci['windows']}.", ""]
        out += ["| Step | " + " | ".join(STEPS) + " |", "|---|" + "---|" * len(STEPS)]
        for r in group:
            tag = f" ({Path(r['_name']).stem[len(base) + 1:]})" if len(group) > 1 else ""
            out += [f"| Pass{tag} | " + " | ".join("yes" if r["steps"][s]["pass"] else "**no**" for s in STEPS) + " |",
                    f"| Minutes{tag} | " + " | ".join(f"{r['steps'][s]['minutes']:g}" for s in STEPS) + " |"]
        out += [""]
    else:
        out += ["No acceptance run recorded yet.", ""]
    out += [f"{len(runs)} run(s), {len(ranked)} open finding(s), {len(fixed)} fixed.", ""]
    if ranked:
        out += ["| # | Kind | Step | Lane | Finding | Runs | Minutes lost | Last seen | Evidence |",
                "|---|---|---|---|---|---|---|---|---|"]
        for n, (fid, r) in enumerate(ranked, 1):
            out.append(f"| {n} | {'**P0** ' if r['p0'] else ''}{r['kind']} | {r['step']} | {r['lane']} | `{cell(fid)}`: {cell(r['summary'])} "
                       f"| {len(r['runs'])} | {r['minutes']:g} | {r['last']} | {cell(r['evidence'])} |")
        out.append("")
        counts = {}
        for _, r in ranked:
            counts.setdefault(r["lane"], [0] * len(KINDS))[KINDS.index(r["kind"])] += 1
        out += ["## By lane", "", "| Lane | Blockers | Workarounds | Non-idiomatic | Hesitations |", "|---|---|---|---|---|"]
        for lane in sorted(counts, key=lambda l: (-counts[l][0], -sum(counts[l]), l)):
            out.append(f"| {lane} | " + " | ".join(str(c) for c in counts[lane]) + " |")
        out.append("")
    if fixed:
        out += ["## Fixed", "", "| Lane | Finding | Runs | Confirmed |", "|---|---|---|---|"]
        for fid, r in fixed:
            out.append(f"| {r['lane']} | `{cell(fid)}`: {cell(r['summary'])} | {len(r['runs'])} | {r['last']} |")
        out.append("")
    return "\n".join(out)


def main(argv):
    runs, errors = load()
    for e in errors:
        print(f"blockers: {e}", file=sys.stderr)
    if errors:
        return 1
    text = render(runs)
    if "--check" in argv:
        if not OUT.exists() or OUT.read_text() != text:
            print("blockers: plan/product/BLOCKERS.md is stale; run research/acceptance/blockers.py", file=sys.stderr)
            return 1
        return 0
    OUT.write_text(text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

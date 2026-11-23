#!/usr/bin/env python3
"""Append each lane's findings from one acceptance run to that lane's worklog.

    research/acceptance/send_rows.py runs/2026-11-23-1.json

One bullet per lane (P0 first, then by kind), with each finding's id, kind, step,
summary and evidence, so the lane can act without the transcript. Lanes come from
`plan/domains/<lane>/`; ranking matches blockers.py.
"""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
KINDS = ["blocker", "workaround", "non-idiomatic", "hesitation"]


def main(argv):
    path = Path(argv[0]).resolve()
    run = json.loads(path.read_text())
    name = path.relative_to(ROOT)
    by_lane = {}
    for f in run["findings"]:
        by_lane.setdefault(f["lane"], []).append(f)
    for lane, fs in sorted(by_lane.items()):
        fs.sort(key=lambda f: (not f.get("p0"), KINDS.index(f["kind"]), -float(f.get("minutes_lost", 0))))
        lines = [f"- {run['date']} (idioms, Milestone 1 acceptance run `{name}`; ranked list in plan/product/BLOCKERS.md): {len(fs)} finding(s) for {lane}."]
        for f in fs:
            tag = "**P0** " if f.get("p0") else ""
            lines.append(f"  - {tag}`{f['id']}` ({f['kind']}, step `{f['step']}`): {f['summary']} Evidence: {f.get('evidence', '')}")
        w = ROOT / "plan" / "domains" / lane / "worklog.md"
        w.write_text(w.read_text().rstrip("\n") + "\n" + "\n".join(lines) + "\n")
        print(f"{lane}: {len(fs)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

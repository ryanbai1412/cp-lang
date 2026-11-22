#!/usr/bin/env python3
"""Queue and train numbers from the land log (git notes under refs/notes/land).

    land_stats.py [--hours H] [--json] [--metric NAME]

scripts/land.sh appends one JSON line per landed lane to the note on the
commit it landed: lane, landed (unix s), wait_s (queued to pushed, minus the
gate), gate_s (the train's gate wall time), riders, splits, holder. Only the
last H hours count (default 24). --metric prints one value, for metrics.toml.
"""

import json
import subprocess
import sys
import time


def notes(ref="refs/notes/land"):
    out = subprocess.run(["git", "notes", "--ref", ref, "list"],
                         capture_output=True, text=True).stdout.split()
    blobs = out[0::2]
    if not blobs:
        return []
    text = subprocess.run(["git", "cat-file", "--batch"], input="\n".join(blobs) + "\n",
                          capture_output=True, text=True, check=True).stdout
    rows = []
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("{"):
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                pass
    return rows


def pct(values, p):
    if not values:
        return None
    s = sorted(values)
    return s[min(len(s) - 1, int(p / 100 * len(s)))]


def stats(rows, hours, now):
    rows = [r for r in rows if r.get("landed", 0) >= now - hours * 3600]
    waits = [r["wait_s"] for r in rows if isinstance(r.get("wait_s"), (int, float))]
    trains = {(r["landed"], r.get("holder")): r for r in rows}.values()
    gates = [t["gate_s"] for t in trains if isinstance(t.get("gate_s"), (int, float))]
    per_lane = {}
    for r in rows:
        per_lane[r.get("lane", "?")] = per_lane.get(r.get("lane", "?"), 0) + 1
    return {
        "hours": hours,
        "landings": len(rows),
        "trains": len(gates),
        "queue_wait_p50_s": pct(waits, 50),
        "queue_wait_max_s": max(waits) if waits else None,
        "train_gate_p50_s": pct(gates, 50),
        "train_gate_max_s": max(gates) if gates else None,
        "landings_per_hour": round(len(rows) / hours, 2),
        "landings_per_hour_by_lane": {k: round(v / hours, 2) for k, v in sorted(per_lane.items())},
    }


def main(argv):
    hours = 24.0
    if "--hours" in argv:
        hours = float(argv[argv.index("--hours") + 1])
    s = stats(notes(), hours, time.time())
    if "--metric" in argv:
        v = s[argv[argv.index("--metric") + 1]]
        print(json.dumps(v) if isinstance(v, dict) else v)
    elif "--json" in argv:
        print(json.dumps(s, indent=2))
    else:
        for k, v in s.items():
            print(f"{k}: {v}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""Landing numbers: how long scripts/land.sh took, and who landed what.

    land_stats.py [--hours H] [--json] [--metric NAME] [--log PATH]

Durations come from the land log, one JSON line per landing that
scripts/land.sh appends to .git/cpl-land.log (domain, landed, seconds,
tries). Landings per domain count the `Domain:` trailers on main (default
origin/main) over the same window. Only the last H hours count (default 24).
--metric prints one value, for metrics.toml.
"""

import json
import os
import re
import subprocess
import sys
import time


def log_rows(path=None):
    if path is None:
        git_dir = subprocess.run(["git", "rev-parse", "--git-common-dir"],
                                 capture_output=True, text=True, check=True).stdout.strip()
        path = os.path.join(git_dir, "cpl-land.log")
    rows = []
    try:
        with open(path, encoding="utf-8") as f:
            for line in f:
                try:
                    rows.append(json.loads(line))
                except json.JSONDecodeError:
                    pass
    except FileNotFoundError:
        pass
    return rows


def lane(trailer):
    """The lane a `Domain:` trailer names; some lanes add a sentence after it."""
    m = re.match(r"\s*([a-z][\w-]*)", trailer)
    return m.group(1) if m else "none"


def trailer_rows(hours, ref="origin/main"):
    out = subprocess.run(["git", "log", f"--since={int(hours * 3600)} seconds ago", "--format=%ct %(trailers:key=Domain,valueonly,separator=)", ref],
                         capture_output=True, text=True).stdout
    rows = []
    for line in out.splitlines():
        ts, _, domain = line.partition(" ")
        if ts.isdigit():
            rows.append({"domain": lane(domain), "landed": int(ts)})
    return rows


def pct(values, p):
    if not values:
        return None
    s = sorted(values)
    return s[min(len(s) - 1, int(p / 100 * len(s)))]


def stats(lands, commits, hours, now):
    def recent(rows):
        return [r for r in rows if r.get("landed", 0) >= now - hours * 3600]
    secs = [r["seconds"] for r in recent(lands) if isinstance(r.get("seconds"), (int, float))]
    per = {}
    for r in recent(commits):
        per[r["domain"]] = per.get(r["domain"], 0) + 1
    return {
        "hours": hours,
        "landings_logged": len(secs),
        "land_p50_s": pct(secs, 50),
        "land_p95_s": pct(secs, 95),
        "land_max_s": max(secs) if secs else None,
        "commits_per_hour": round(sum(per.values()) / hours, 2),
        "commits_per_hour_by_domain": {k: round(v / hours, 2) for k, v in sorted(per.items())},
    }


def main(argv):
    hours = float(argv[argv.index("--hours") + 1]) if "--hours" in argv else 24.0
    log = argv[argv.index("--log") + 1] if "--log" in argv else None
    s = stats(log_rows(log), trailer_rows(hours), hours, time.time())
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

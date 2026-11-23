#!/usr/bin/env python3
"""Devex share per lane, and commit velocity, from main's history.

    devex_stats.py [--hours H] [--ref REF] [--json]

A devex commit is one whose subject only keeps the factory running
(regenerate, re-bless, renumber, ratchet bump, fix a gate) rather than
shipping lane work; the regen commits land.sh and CI write count as none. Share = devex commits / commits per lane: a proxy for
the share of time each lane spends on devex, held under 5%. Infra's own
commits are infra's job and listed separately. Velocity counts all commits
on REF (default origin/main) per 15-minute window, newest last, so a drop
after a rollout shows up at once.
"""

import json
import os
import re
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from domain_of import domain_of  # noqa: E402

DEVEX = re.compile(r"regenerat|re-?bless|renumber|ratchet|baseline|stale (derived|generated)|"
                   r"satisfy clippy|fix (the )?(gate|ci)|not mine", re.I)


# land.sh and CI's regen job write these themselves: machine time, not lane time.
AUTO = {"factory: regenerate generated files"}

# Lanes whose product is the ratchets or the gate: building or tightening their
# own is their work, not friction.
OWN_WORK = {"tech-debt": re.compile(r"ratchet|baseline", re.I),
            "lint-fmt": re.compile(r"ratchet|(warning )?baselines?", re.I),
            "infra": re.compile(r"ratchet|baseline|regenerat|fix (the )?(gate|ci)", re.I)}


def is_devex(lane, subject):
    own = OWN_WORK.get(lane)
    return subject not in AUTO and bool(DEVEX.search(own.sub("", subject) if own else subject))


def commits(ref, hours):
    out = subprocess.run(
        ["git", "log", ref, f"--since={int(hours * 60)} minutes ago",
         "--format=%x00%ct%x09%(trailers:key=Domain,valueonly,separator=%x2C)%x09%s", "--name-only"],
        capture_output=True, text=True, check=True).stdout
    rows = []
    for entry in out.split("\0")[1:]:
        head, _, files = entry.partition("\n")
        ts, domain, subject = (head.split("\t", 2) + ["", ""])[:3]
        # A commit without a trailer counts for the lane whose worklog it touches.
        lane = (domain.split(",")[0].strip().split()[0].rstrip(".") if domain.strip()
                else domain_of(files.split()) or "?")
        rows.append((int(ts), lane, subject))
    return rows


def stats(rows, hours, now):
    lanes = {}
    for _, lane, subject in rows:
        c = lanes.setdefault(lane, {"commits": 0, "devex": 0})
        c["commits"] += 1
        c["devex"] += is_devex(lane, subject)
    for c in lanes.values():
        c["share"] = round(c["devex"] / c["commits"], 3)
    windows = int(hours * 4)
    velocity = [0] * windows
    for ts, _, _ in rows:
        i = windows - 1 - int((now - ts) // 900)
        if 0 <= i < windows:
            velocity[i] += 1
    return {"lanes": lanes, "velocity_per_15min": velocity}


def main(argv):
    hours, ref, as_json = 3.0, "origin/main", "--json" in argv
    if "--hours" in argv:
        hours = float(argv[argv.index("--hours") + 1])
    if "--ref" in argv:
        ref = argv[argv.index("--ref") + 1]
    s = stats(commits(ref, hours), hours, time.time())
    if as_json:
        print(json.dumps(s, indent=1))
        return 0
    print(f"velocity per 15 min, oldest first: {s['velocity_per_15min']}")
    for lane, c in sorted(s["lanes"].items(), key=lambda kv: -kv[1]["share"]):
        flag = "  OVER 5%" if lane != "infra" and c["share"] > 0.05 else ""
        print(f"{lane:12} {c['devex']:3}/{c['commits']:<4} devex {c['share']:.0%}{flag}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

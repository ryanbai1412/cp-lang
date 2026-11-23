#!/usr/bin/env python3
"""The domain (lane) a change belongs to, for a missing `Domain:` trailer.

    domain_of.py PATH...         prints the domain, or nothing when unsure
    domain_of.py --commit SHA    the same for one commit's paths

$CPL_DOMAIN wins when it names a domain. Otherwise a change that touches
exactly one lane's plan/domains/<d>/ belongs to that lane: every lane logs
its work in its own worklog, and on main's history this agrees with the
trailer 9 times in 10. Anything else is unsure. The commit-msg hook uses
this to add the trailer, and devex_stats.py to count commits without one.
"""

import os
import re
import subprocess
import sys

DOMAINS = ("infra", "compiler", "perf", "diagnostics", "tech-debt", "lint-fmt",
           "tools", "spec", "stdlib", "idioms", "site", "debugger")

LANE_DIR = re.compile(r"^plan/domains/([a-z-]+)/")


def domain_of(paths, configured=None):
    if configured in DOMAINS:
        return configured
    lanes = {m.group(1) for p in paths if (m := LANE_DIR.match(p)) and m.group(1) in DOMAINS}
    return lanes.pop() if len(lanes) == 1 else None


def main(argv):
    if argv[:1] == ["--commit"]:
        argv = subprocess.run(["git", "show", "--name-only", "--format=", argv[1]],
                              capture_output=True, text=True, check=True).stdout.split()
    d = domain_of(argv, os.environ.get("CPL_DOMAIN"))
    if d:
        print(d)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

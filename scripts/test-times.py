#!/usr/bin/env python3
"""The tools lane's slow tests: every test of its crates over 30 s.

    test-times.py --write          run the tools crates' tests and record the slow ones
    test-times.py --junit PATH     record them from a nextest JUnit report instead
    test-times.py --show           print the record

`--write` runs `cargo nextest run` over TOOLS_CRATES with the `times` profile
(.config/nextest.toml), which writes target/nextest/times/junit.xml, and keeps
each test over LIMIT_S in research/perf/tools-test-times.json. The debt ratchet's
`tools_tests_over_30s` counts that list, so a slow test fails the gate until it
is cut or sharded under the limit.
"""

import argparse
import json
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECORD = "research/perf/tools-test-times.json"
TOOLS_CRATES = ("cpl-cli", "cpl-lsp", "cpl-contest", "cpl-stress", "cpl-corpus")
LIMIT_S = 30.0
JUNIT = "target/nextest/times/junit.xml"


def slow_tests(junit: str, limit: float = LIMIT_S) -> dict[str, float]:
    """`binary test` -> seconds for each test case in a nextest JUnit report over `limit`."""
    out: dict[str, float] = {}
    for case in ET.fromstring(junit).iter("testcase"):
        seconds = float(case.get("time", "0"))
        if seconds > limit:
            out[f"{case.get('classname')} {case.get('name')}"] = round(seconds, 1)
    return dict(sorted(out.items(), key=lambda kv: (-kv[1], kv[0])))


def record(slow: dict[str, float]) -> str:
    command = "cargo nextest run --profile times " + " ".join(f"-p {c}" for c in TOOLS_CRATES)
    return json.dumps({"command": command, "limit_s": LIMIT_S, "over_limit": slow}, indent=2) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--junit")
    ap.add_argument("--show", action="store_true")
    args = ap.parse_args()
    if args.show:
        print((ROOT / RECORD).read_text(), end="")
        return 0
    junit = Path(args.junit) if args.junit else ROOT / JUNIT
    if args.write:
        argv = ["cargo", "nextest", "run", "--profile", "times", "--no-fail-fast"]
        for crate in TOOLS_CRATES:
            argv += ["-p", crate]
        subprocess.run(argv, cwd=ROOT, check=False)
    elif not args.junit:
        ap.error("pass --write, --junit PATH or --show")
    slow = slow_tests(junit.read_text())
    (ROOT / RECORD).write_text(record(slow))
    print(f"test-times: {len(slow)} test(s) over {LIMIT_S:.0f} s in {RECORD}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

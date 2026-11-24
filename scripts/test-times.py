#!/usr/bin/env python3
"""The tools lane's slow tests: every test of its crates over 30 s.

    test-times.py --write          run the tools crates' tests and record the slow ones
    test-times.py --junit PATH     record them from a nextest JUnit report instead
    test-times.py --junit PATH --merge
                                   re-time only the test binaries in the report
    test-times.py --show           print the record

`--write` runs `cargo nextest run` over TOOLS_CRATES with the `times` profile
(.config/nextest.toml), which writes target/nextest/times/junit.xml, and keeps
each test over LIMIT_S in research/perf/tools-test-times.json. g++ goes
through scripts/cxx-cache.py, as in CI's test parts. The debt ratchet's
`tools_tests_over_30s` counts that list, so a slow test fails the gate until it
is cut or sharded under the limit.
"""

import argparse
import json
import os
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import ci  # noqa: E402

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


def binaries(junit: str) -> set[str]:
    """The test binaries (`crate::binary`) a nextest JUnit report ran."""
    return {case.get("classname") for case in ET.fromstring(junit).iter("testcase")}


def merged(old: dict[str, float], junit: str) -> dict[str, float]:
    """`old` with every test of the report's binaries re-timed from it, so a test the
    report ran under the limit, or that its binary no longer has, leaves the record."""
    ran = binaries(junit)
    kept = {t: s for t, s in old.items() if t.split(" ", 1)[0] not in ran}
    both = kept | slow_tests(junit)
    return dict(sorted(both.items(), key=lambda kv: (-kv[1], kv[0])))


def record(slow: dict[str, float]) -> str:
    command = "cargo nextest run --profile times " + " ".join(f"-p {c}" for c in TOOLS_CRATES)
    return json.dumps({"command": command, "limit_s": LIMIT_S, "over_limit": slow}, indent=2) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--junit")
    ap.add_argument("--merge", action="store_true")
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
        # CI's tests build C++ through the g++ cache (ci.cxx_cache); so does
        # the record, or it times builds no CI run makes.
        env = dict(os.environ)
        ci.cxx_cache(ROOT / "target", env)
        subprocess.run(argv, cwd=ROOT, env=env, check=False)
    elif not args.junit:
        ap.error("pass --write, --junit PATH or --show")
    if args.merge and not args.junit:
        ap.error("--merge needs --junit PATH")
    if args.merge:
        old = json.loads((ROOT / RECORD).read_text())["over_limit"]
        slow = merged(old, junit.read_text())
    else:
        slow = slow_tests(junit.read_text())
    (ROOT / RECORD).write_text(record(slow))
    print(f"test-times: {len(slow)} test(s) over {LIMIT_S:.0f} s in {RECORD}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Reports tests that failed and then passed on nextest's retry (ci/R35).

CI's test partitions run with `--retries 1`, so a flaky test no longer turns
main red; this names each one so the sheriff routes it to its owner instead
of letting it hide.

    flaky.py <nextest-log>
"""
import os
import re
import sys

FLAKY = re.compile(r"^\s*FLAKY \d+/\d+ \[[^\]]*\] (?:\([^)]*\) )?(\S+) (\S+)\s*$")


def flaky(text):
    return sorted({f"{m.group(1)} {m.group(2)}" for line in text.splitlines() if (m := FLAKY.match(line))})


def main(argv):
    with open(argv[1], encoding="utf-8", errors="replace") as f:
        found = flaky(f.read())
    for t in found:
        print(f"::warning::flaky test (failed, then passed on retry): {t}")
    if found and (summary := os.environ.get("GITHUB_STEP_SUMMARY")):
        with open(summary, "a", encoding="utf-8") as f:
            f.write("### Flaky tests\n\n" + "".join(f"- `{t}`\n" for t in found))
    print(f"flaky: {len(found)} test(s) passed only on retry")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

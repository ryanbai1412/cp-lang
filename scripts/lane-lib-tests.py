#!/usr/bin/env python3
"""The lane-j-check landing gate's unit tests: `cargo test -p cpl-next --lib` and
`cargo test -p cpl-doc --lib` (coordinator, 00:18 and 00:25: a slice-1 switch blocker).

    lane-lib-tests.py            run both, refuse a red that tests/lane-lib-reds.txt doesn't list
    lane-lib-tests.py --list     print the reds of a run, one `<crate> <test>` a line

Every red refuses unless the list names it; the list is shrink-only: a listed test that
passes refuses too until its row goes, so a fix can't leave room for the next break. The
list empties before slice 1 lands (handbook/design/next/migration.md, the landing gate).
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REDS = ROOT / "tests" / "lane-lib-reds.txt"
CRATES = ("cpl-next", "cpl-doc")
FAILED = re.compile(r"^test (\S+) \.\.\. FAILED$")
PANICKED = re.compile(r"^---- (\S+) stdout ----$")  # `-q` prints no `test ... FAILED` line


def reds_of(output: str) -> set[str]:
    """The test paths a libtest run names as failed, in `test <path> ... FAILED` or `---- <path> stdout ----`."""
    return {m.group(1) for line in output.splitlines()
            if (m := FAILED.match(line.strip()) or PANICKED.match(line.strip()))}


def listed(text: str) -> set[tuple[str, str]]:
    """The `<crate> <test>` rows of the list; `#` starts a comment (the owner and commit)."""
    rows = set()
    for line in text.splitlines():
        words = line.split("#", 1)[0].split()
        if words:
            if len(words) != 2:
                raise SystemExit(f"{REDS.relative_to(ROOT)}: `{line}` is not `<crate> <test>  # owner, commit`")
            rows.add((words[0], words[1]))
    return rows


def verdict(found: set[tuple[str, str]], allowed: set[tuple[str, str]], ran: set[str]) -> list[str]:
    """Why the gate refuses: an unlisted red, or a listed test of a crate that ran and passed."""
    out = [f"NEW red: {c} {t} (not in tests/lane-lib-reds.txt)" for c, t in sorted(found - allowed)]
    out += [f"fixed: {c} {t} passes; drop its row from tests/lane-lib-reds.txt" for c, t in sorted(allowed - found)
            if c in ran]
    return out


def run(crate: str) -> tuple[int, str]:
    done = subprocess.run(["cargo", "test", "-q", "-p", crate, "--lib", "--no-fail-fast"], cwd=ROOT,
                          capture_output=True, text=True)
    return done.returncode, done.stdout + done.stderr


def main(argv: list[str]) -> int:
    if set(argv) - {"--list"}:
        raise SystemExit(__doc__)
    found, ran = set(), set()
    for crate in CRATES:
        code, out = run(crate)
        reds = reds_of(out)
        if code and not reds:
            sys.stderr.write(out[-4000:])
            print(f"lane-lib-tests: cargo test -p {crate} --lib failed with no test named (a build error)", file=sys.stderr)
            return 1
        ran.add(crate)
        found |= {(crate, t) for t in reds}
    if "--list" in argv:
        print("\n".join(f"{c} {t}" for c, t in sorted(found)))
        return 0
    allowed = listed(REDS.read_text(encoding="utf-8")) if REDS.is_file() else set()
    why = verdict(found, allowed, ran)
    for w in why:
        print(f"lane-lib-tests: {w}", file=sys.stderr)
    if not why:
        print(f"lane-lib-tests: ok ({len(found)} listed reds left, target 0)")
    return 1 if why else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

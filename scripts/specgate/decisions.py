"""Decision numbers in decisions.md: print the next, reserve one, or check.

Usage:
    scripts/spec-gate.py decisions                # next integer (max + 1 over this tree and origin/main)
    scripts/spec-gate.py decisions --reserve      # push a `## N. (reserved, <lane>)` stub to main, print N
    scripts/spec-gate.py decisions --check        # fail if duplicate heading numbers exist
    scripts/spec-gate.py decisions --check-landing FETCH_HEAD
                                                  # fail if HEAD heads a number the base never had

A number is taken only once it is on main, so two lanes cannot both take
`max + 1`: `--reserve` commits the stub on top of main without touching the
worktree and pushes it, retrying on a rejected push. Pull, then replace the
stub's heading with the decision. `scripts/land.sh` runs `--check-landing`.
"""
import os
import re
import subprocess
import sys
import tempfile

from retry import retry

import specgate
from specgate import Problem

HELP = "decision numbers: print the next, --reserve one on main, --check duplicates, --check-landing BASE"

ROOT = str(specgate.ROOT)
DECISIONS = os.path.join(ROOT, "decisions.md")
HEADING_RE = re.compile(r"^##\s+(\d+)\.", re.MULTILINE)
STUB_RE = re.compile(r"^##\s+(\d+)\.\s+\(reserved, [^)]*\)\s*$", re.MULTILINE)


def parse_decision_numbers(text: str) -> list[int]:
    return [int(m.group(1)) for m in HEADING_RE.finditer(text)]


def next_number(text: str) -> int:
    nums = parse_decision_numbers(text)
    return (max(nums) + 1) if nums else 1


def git(*args: str, stdin: str | None = None, env: dict | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", *args], cwd=ROOT, input=stdin, capture_output=True, text=True, env=env
    )


def show(ref: str) -> str | None:
    r = git("show", f"{ref}:decisions.md")
    return r.stdout if r.returncode == 0 else None


def upstream_text(ref: str = "origin/main") -> str:
    """decisions.md on `ref`, so a number another lane just landed is not reused."""
    git("fetch", "-q", "origin", "main")
    return show(ref) or ""


def find_duplicates(text: str) -> list[int]:
    seen: set[int] = set()
    dups: list[int] = []
    for n in parse_decision_numbers(text):
        if n in seen and n not in dups:
            dups.append(n)
        seen.add(n)
    return dups


def unreserved(base: str, head: str) -> list[int]:
    """Numbers `head` heads that `base` (main before this landing) never had."""
    have = set(parse_decision_numbers(base))
    return sorted({n for n in parse_decision_numbers(head) if n not in have})


def with_stub(text: str, n: int, lane: str) -> str:
    return text.rstrip("\n") + f"\n\n## {n}. (reserved, {lane})\n"


class PushRejected(Exception):
    """Main moved between the fetch and the push."""


def reserve(lane: str, domain: str, remote: str, tries: int) -> int:
    local = open(DECISIONS, encoding="utf-8").read() if os.path.exists(DECISIONS) else ""

    def attempt() -> int:
        f = git("fetch", "-q", remote, "main")
        if f.returncode != 0:
            print(f"spec-gate decisions: cannot fetch {remote} main (offline?): {f.stderr.strip()}", file=sys.stderr)
            return 2
        base = git("rev-parse", "FETCH_HEAD").stdout.strip()
        text = show(base) or ""
        n = max(next_number(text), next_number(local))
        blob = git("hash-object", "-w", "--stdin", stdin=with_stub(text, n, lane)).stdout.strip()
        with tempfile.TemporaryDirectory() as tmp:
            env = dict(os.environ, GIT_INDEX_FILE=os.path.join(tmp, "index"))
            git("read-tree", base, env=env)
            git("update-index", "--cacheinfo", f"100644,{blob},decisions.md", env=env)
            tree = git("write-tree", env=env).stdout.strip()
        msg = f"spec: reserve decision {n} for {lane}\n\nDomain: {domain}\n"
        commit = git("commit-tree", tree, "-p", base, stdin=msg).stdout.strip()
        if git("push", "-q", remote, f"{commit}:refs/heads/main").returncode == 0:
            print(n)
            print(
                f"spec-gate decisions: reserved {n} on main ({commit[:9]}); `git pull --rebase`, "
                f"then replace `## {n}. (reserved, {lane})` with the decision",
                file=sys.stderr,
            )
            return 0
        raise PushRejected(n)

    try:
        return retry(attempt, transient=lambda e: isinstance(e, PushRejected), tries=tries, wait=0)
    except PushRejected:
        print(f"spec-gate decisions: push rejected {tries} times; try again", file=sys.stderr)
        return 1


def check_landing(base_ref: str) -> int:
    if git("rev-parse", "-q", "--verify", f"{base_ref}^{{commit}}").returncode != 0:
        print(
            f"spec-gate decisions: {base_ref} is missing (offline, or no fetch ran): "
            "cannot check decision reservations; fetch main and land again",
            file=sys.stderr,
        )
        return 2
    base, head = show(base_ref), show("HEAD")
    if base is None or head is None:
        return 0
    new = unreserved(base, head)
    if new:
        print(
            f"spec-gate decisions: decision heading(s) {new} are not reserved on main; run "
            "`uv run scripts/spec-gate.py decisions --reserve`, pull, and put the entry under the stub",
            file=sys.stderr,
        )
        return 1
    return 0


def args(parser) -> None:
    parser.add_argument("--file", help="Path to decisions.md")
    parser.add_argument("--check", action="store_true", help="Fail if duplicate decision numbers exist")
    parser.add_argument("--reserve", action="store_true", help="Reserve the next number on main")
    parser.add_argument("--check-landing", metavar="BASE", help="Fail on headings BASE never had")
    parser.add_argument("--lane", default=os.environ.get("CPL_LANE", "unknown"))
    parser.add_argument("--domain", default=os.environ.get("CPL_DOMAIN", "spec"))
    parser.add_argument("--remote", default="origin")
    parser.add_argument("--tries", type=int, default=int(os.environ.get("CPL_LAND_TRIES", "5")))


def run(args) -> list[Problem]:
    if args.reserve:
        code = reserve(args.lane, args.domain, args.remote, args.tries)
        return [Problem("decisions.md", "no number reserved; see above")] if code else []
    if args.check_landing:
        code = check_landing(args.check_landing)
        return [Problem("decisions.md", "landing check failed; see above")] if code else []
    path = args.file or DECISIONS
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    if args.check:
        return [Problem(path, f"decision number {n} is used twice") for n in find_duplicates(text)]
    print(max(next_number(text), next_number(upstream_text())))
    return []

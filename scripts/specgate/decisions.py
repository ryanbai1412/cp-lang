"""Decision numbers in decisions.md: print the next, reserve one, or check.

Usage:
    scripts/spec-gate.py decisions                # next integer (max + 1 over this tree and origin/main)
    CPL_LANE=E scripts/spec-gate.py decisions --reserve --title "views own their keys"
                                                  # push a `## N. (reserved, E: <title>)` stub to main, print N
    scripts/spec-gate.py decisions --check        # fail if duplicate heading numbers exist
    scripts/spec-gate.py decisions --check-landing FETCH_HEAD
                                                  # fail if HEAD heads a number the base never had

A number is taken only once it is on main, so two lanes cannot both take
`max + 1`: `--reserve` commits the stub on top of main without touching the
worktree and pushes it, retrying on a rejected push. The stub's heading and
the commit's `Lane:` trailer name the lane and the title, so nobody has to
ask whose number it is. Pull, then replace the stub's heading with the decision. `scripts/land.sh` runs `--check-landing`.
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


def stub(n: int, lane: str, title: str) -> str:
    return f"## {n}. (reserved, {lane}: {title})"


def with_stub(text: str, n: int, lane: str, title: str) -> str:
    return text.rstrip("\n") + f"\n\n{stub(n, lane, title)}\n"


def reservation_problem(lane: str | None, title: str | None) -> str | None:
    """Why a reservation must not run: a stub must say whose number it is and for what."""
    if not lane or lane == "unknown" or not re.fullmatch(r"[\w-]+", lane):
        return "set CPL_LANE (or --lane) to your lane, e.g. CPL_LANE=E"
    if not title or not title.strip():
        return 'give the decision a --title, e.g. --title "views own their keys"'
    if re.search(r"[()\n]", title):
        return "a --title has no parentheses or newlines (the stub's heading closes on `)`)"
    return None


MOVED_RE = re.compile(r"fetch first|non-fast-forward|cannot lock ref|incorrect old value")


class PushRejected(Exception):
    """Main moved between the fetch and the push."""


def lane_texts(remote: str) -> list[str]:
    """decisions.md on each of `remote`'s lane branches: a number a lane already heads is taken,
    or the lane's merge into main heads it twice (679, 11-28)."""
    git("fetch", "-q", remote, f"+refs/heads/lane-*:refs/remotes/{remote}/lane-*")
    refs = git("for-each-ref", "--format=%(refname)", f"refs/remotes/{remote}/lane-*").stdout.split()
    return [show(r) or "" for r in refs]


def reserve(lane: str | None, title: str | None, domain: str, remote: str, tries: int) -> int:
    if why := reservation_problem(lane, title):
        print(f"spec-gate decisions: no number reserved: {why}", file=sys.stderr)
        return 2
    title = title.strip()
    local = open(DECISIONS, encoding="utf-8").read() if os.path.exists(DECISIONS) else ""

    def push_reservation() -> int:
        f = git("fetch", "-q", remote, "main")
        if f.returncode != 0:
            print(f"spec-gate decisions: cannot fetch {remote} main (offline?): {f.stderr.strip()}", file=sys.stderr)
            return 2
        base = git("rev-parse", "FETCH_HEAD").stdout.strip()
        text = show(base) or ""
        n = max(next_number(text), next_number(local), *map(next_number, lane_texts(remote)))
        blob = git("hash-object", "-w", "--stdin", stdin=with_stub(text, n, lane, title)).stdout.strip()
        with tempfile.TemporaryDirectory() as tmp:
            env = dict(os.environ, GIT_INDEX_FILE=os.path.join(tmp, "index"))
            git("read-tree", base, env=env)
            git("update-index", "--cacheinfo", f"100644,{blob},decisions.md", env=env)
            tree = git("write-tree", env=env).stdout.strip()
        msg = f"spec: reserve decision {n} for lane {lane}\n\n{title}\n\nDomain: {domain}\nLane: {lane}\n"
        commit = git("commit-tree", tree, "-p", base, stdin=msg).stdout.strip()
        # The commit is main's tree plus one stub, so the local pre-push hook,
        # which checks the working tree, has nothing to say about it.
        pushed = git("push", "--no-verify", "-q", remote, f"{commit}:refs/heads/main")
        if pushed.returncode == 0:
            print(n)
            print(
                f"spec-gate decisions: reserved {n} on main ({commit[:9]}); `git pull --rebase`, "
                f"then replace `{stub(n, lane, title)}` with the decision",
                file=sys.stderr,
            )
            return 0
        if not MOVED_RE.search(pushed.stderr):
            print(f"spec-gate decisions: cannot push to {remote} main: {pushed.stderr.strip()}", file=sys.stderr)
            return 1
        raise PushRejected(pushed.stderr.strip())

    try:
        return retry(push_reservation, transient=lambda e: isinstance(e, PushRejected), tries=tries, wait=0)
    except PushRejected as e:
        print(f"spec-gate decisions: main moved under {tries} pushes in a row; try again: {e}", file=sys.stderr)
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
            "`CPL_LANE=<lane> uv run scripts/spec-gate.py decisions --reserve --title <title>`, pull, and put the entry under the stub",
            file=sys.stderr,
        )
        return 1
    return 0


def args(parser) -> None:
    parser.add_argument("--file", help="Path to decisions.md")
    parser.add_argument("--check", action="store_true", help="Fail if duplicate decision numbers exist")
    parser.add_argument("--reserve", action="store_true", help="Reserve the next number on main")
    parser.add_argument("--check-landing", metavar="BASE", help="Fail on headings BASE never had")
    parser.add_argument("--lane", default=os.environ.get("CPL_LANE"))
    parser.add_argument("--title", help="With --reserve: what the decision is about")
    parser.add_argument("--domain", default=os.environ.get("CPL_DOMAIN", "spec"))
    parser.add_argument("--remote", default="origin")
    parser.add_argument("--tries", type=int, default=int(os.environ.get("CPL_LAND_TRIES", "5")))


def run(args) -> list[Problem]:
    if args.reserve:
        code = reserve(args.lane, args.title, args.domain, args.remote, args.tries)
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

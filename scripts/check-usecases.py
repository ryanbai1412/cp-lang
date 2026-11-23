#!/usr/bin/env python3
"""Check that every product use case's status is backed by evidence.

Rule: handbook/rfcs/0002-product-surfaces.md#keeping-it-true. Reads
`handbook/product/**/*-usecases.md`, where each use case is

    ### <area>/U<n>: <title>
    Need: ...
    Design: ...
    Milestone: M1-critical. Status: partial (why)
    Evidence: integration `crates/x/tests/y.rs` (`a_test_name`)

and fails when an ID repeats, a new ID takes a number already used (at HEAD
or in `scripts/usecases.json`), a field is missing, a status or milestone
is unknown, or an Evidence path or named test doesn't exist. A `supported`
or `partial` entry whose Evidence names no runnable file counts as `gap`.

Two M1 numbers, over M1-critical entries that aren't supported: *baseline*,
those open when the catalogue was first written (`scripts/usecases.json`,
only goes down), and *readiness*, all of them.

    python3 scripts/check-usecases.py            summary per area
    python3 scripts/check-usecases.py --check    fail on any error above
    python3 scripts/check-usecases.py --update   prune closed baseline IDs, raise the ID high-water marks
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STATE = "scripts/usecases.json"
FIELDS = ("Need", "Design", "Milestone", "Evidence")
STATUSES = {"supported", "partial", "gap", "deferred", "non-goal"}
MILESTONES = {"M1-critical", "M1-differentiating", "later"}
RUNNABLE = {".rs", ".cp", ".mjs", ".js", ".cjs", ".py", ".sh"}
HEAD_RE = re.compile(r"^### ([a-z][a-z0-9-]*)/U(\d+): (.+)$")
MILESTONE_RE = re.compile(r"^Milestone: ([\w-]+)\. Status: ([\w-]+)(?: \(.*\))?\.?$")
SPAN_RE = re.compile(r"`([^`]+)`")
TESTS_RE = re.compile(r"`([^`]+)`\s*\(([^)]*)\)")
TEST_NAME_RE = re.compile(r"[a-z][a-z0-9]*(?:_[a-z0-9]+)+")


def catalogues(root: Path) -> list[Path]:
    return sorted((root / "handbook/product").glob("**/*-usecases.md"))


def parse(rel: str, text: str) -> tuple[list[dict], list[str]]:
    """The entries of one catalogue and its format errors."""
    entries, errors, cur = [], [], None
    for n, line in enumerate(text.splitlines(), 1):
        if line.startswith("#"):
            cur = None
            m = HEAD_RE.match(line)
            if m:
                cur = {"area": m[1], "num": int(m[2]), "id": f"{m[1]}/U{m[2]}", "title": m[3],
                       "file": rel, "line": n, "fields": {}}
                entries.append(cur)
            elif line.startswith("### "):
                errors.append(f"{rel}:{n}: heading isn't `### <area>/U<n>: <title>`")
            continue
        if cur is None or ":" not in line:
            continue
        key, _, value = line.partition(":")
        if key in FIELDS:
            cur["fields"][key] = line if key == "Milestone" else value.strip()
    for e in entries:
        where = f"{e['file']}:{e['line']}: {e['id']}"
        missing = [f for f in FIELDS if not e["fields"].get(f)]
        if missing:
            errors.append(f"{where}: missing {', '.join(missing)}")
        m = MILESTONE_RE.match(e["fields"].get("Milestone", ""))
        if e["fields"].get("Milestone") and not m:
            errors.append(f"{where}: write `Milestone: <milestone>. Status: <status>`")
        e["milestone"], e["status"] = (m[1], m[2]) if m else ("?", "?")
        if m and m[1] not in MILESTONES:
            errors.append(f"{where}: unknown milestone {m[1]}")
        if m and m[2] not in STATUSES:
            errors.append(f"{where}: unknown status {m[2]}")
    return entries, errors


def evidence(root: Path, e: dict) -> tuple[bool, list[str]]:
    """Whether an entry names a runnable check, and the paths or tests it names that are gone."""
    text = e["fields"].get("Evidence", "")
    runnable, gone = False, []
    for span in SPAN_RE.findall(text):
        if "/" not in span or " " in span or not re.match(r"^[\w.-]+/", span) or re.match(r"^[a-z-]+/[RU]\d+$", span):
            continue
        path = root / span.rstrip("/")
        if not path.exists():
            gone.append(span)
        elif path.is_dir() or path.suffix in RUNNABLE:
            runnable = True
    for file, names in TESTS_RE.findall(text):
        path = root / file
        if not path.is_file():
            continue
        body = path.read_text(encoding="utf-8", errors="replace")
        for name in SPAN_RE.findall(names):
            if TEST_NAME_RE.fullmatch(name) and not re.search(rf"\b{re.escape(name)}\b", body):
                gone.append(f"{file} ({name})")
    return runnable, gone


def effective(root: Path, e: dict) -> str:
    """The status the evidence backs: a claim with no runnable check is a gap."""
    if e["status"] in ("supported", "partial") and not evidence(root, e)[0]:
        return "gap"
    return e["status"]


def head_ids(root: Path) -> tuple[dict[str, int], set[str]]:
    """The highest ID number per area at HEAD, and the IDs there."""
    top: dict[str, int] = {}
    ids: set[str] = set()
    try:
        files = subprocess.run(["git", "ls-tree", "-r", "--name-only", "HEAD", "handbook/product"], cwd=root,
                               capture_output=True, text=True, check=True).stdout.split()
    except (OSError, subprocess.CalledProcessError):
        return top, ids
    for f in files:
        if not f.endswith("-usecases.md"):
            continue
        text = subprocess.run(["git", "show", f"HEAD:{f}"], cwd=root, capture_output=True, text=True).stdout
        for m in re.finditer(r"(?m)^### ([a-z][a-z0-9-]*)/U(\d+):", text):
            top[m[1]] = max(top.get(m[1], 0), int(m[2]))
            ids.add(f"{m[1]}/U{m[2]}")
    return top, ids


def load(root: Path) -> tuple[list[dict], list[str], dict]:
    entries, errors = [], []
    for p in catalogues(root):
        found, errs = parse(str(p.relative_to(root)), p.read_text(encoding="utf-8"))
        entries += found
        errors += errs
    state_path = root / STATE
    state = json.loads(state_path.read_text()) if state_path.is_file() else {"baseline": [], "high_water": {}}
    seen: dict[str, dict] = {}
    for e in entries:
        if e["id"] in seen:
            errors.append(f"{e['file']}:{e['line']}: {e['id']} is also {seen[e['id']]['file']}:{seen[e['id']]['line']}")
        seen[e["id"]] = e
    head, head_set = head_ids(root)
    for e in entries:
        mark = max(head.get(e["area"], 0), state["high_water"].get(e["area"], 0))
        if e["id"] not in head_set and e["num"] <= mark:
            errors.append(f"{e['file']}:{e['line']}: {e['id']} reuses a number; the next is U{mark + 1}")
        for ref in re.findall(r"`([a-z][a-z0-9-]*/U\d+)`", e["fields"].get("Evidence", "")):
            if ref not in seen:
                errors.append(f"{e['file']}:{e['line']}: {e['id']} names no such use case {ref}")
        _, gone = evidence(root, e)
        for g in gone:
            errors.append(f"{e['file']}:{e['line']}: {e['id']} names no such evidence {g}")
        e["effective"] = effective(root, e)
    return entries, errors, state


def numbers(entries: list[dict], state: dict) -> dict[str, dict]:
    """Per-area counts, plus `all`."""
    by_id = {e["id"]: e for e in entries}
    out: dict[str, dict] = {}
    for e in entries:
        for area in (e["area"], "all"):
            a = out.setdefault(area, {"cases": 0, "supported": 0, "partial": 0, "gap": 0, "unbacked": 0,
                                      "baseline": 0, "readiness": 0})
            a["cases"] += 1
            if e["effective"] in ("supported", "partial", "gap"):
                a[e["effective"]] += 1
            a["unbacked"] += e["status"] != e["effective"]
            a["readiness"] += e["milestone"] == "M1-critical" and e["effective"] in ("partial", "gap")
    for id_ in state["baseline"]:
        e = by_id.get(id_)
        if e is None or (e["milestone"] == "M1-critical" and e["effective"] in ("partial", "gap")):
            for area in (id_.split("/")[0], "all"):
                out.setdefault(area, {"cases": 0, "supported": 0, "partial": 0, "gap": 0, "unbacked": 0,
                                      "baseline": 0, "readiness": 0})["baseline"] += 1
    return out


def table(entries: list[dict], state: dict) -> list[str]:
    """Markdown rows for plan/STATUS.md."""
    rows = ["| Area | Use cases | Supported | Partial | Gap | Claimed without evidence | M1 baseline gaps | M1 readiness gaps |",
            "|---|---|---|---|---|---|---|---|"]
    nums = numbers(entries, state)
    for area in sorted(nums, key=lambda a: (a == "all", a)):
        n = nums[area]
        rows.append(f"| {area} | {n['cases']} | {n['supported']} | {n['partial']} | {n['gap']} | {n['unbacked']} "
                    f"| {n['baseline']} | {n['readiness']} |")
    return rows


def main(argv: list[str], root: Path = ROOT) -> int:
    entries, errors, state = load(root)
    if "--update" in argv:
        open_ = {e["id"] for e in entries if e["milestone"] == "M1-critical" and e["effective"] in ("partial", "gap")}
        ids = {e["id"] for e in entries}
        if not (root / STATE).is_file():
            state["baseline"] = sorted(open_)
        else:
            state["baseline"] = [i for i in state["baseline"] if i in open_ or i not in ids]
        for e in entries:
            state["high_water"][e["area"]] = max(state["high_water"].get(e["area"], 0), e["num"])
        state["high_water"] = dict(sorted(state["high_water"].items()))
        (root / STATE).write_text(json.dumps(state, indent=2) + "\n")
        print(f"check-usecases: wrote {STATE}")
    for line in table(entries, state):
        print(line)
    for e in entries:
        if e["status"] != e["effective"]:
            print(f"  {e['file']}:{e['line']}: {e['id']} says {e['status']} but names nothing runnable; counted as gap")
    for err in errors:
        print(f"check-usecases: ERROR: {err}", file=sys.stderr)
    return 1 if errors and "--check" in argv else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

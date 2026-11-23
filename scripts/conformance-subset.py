#!/usr/bin/env python3
"""Check spec/SUBSET.md, the Milestone 1 subset, against conformance results.

Each row of SUBSET.md's table names a rule section (`R04.4` is rules
R04.4.*, `Rcli` is Rcli.*), its heading, a status and a note:

  supported   promised: every case citing it passes, or is listed under
              `## Known gaps` as `- <case>: <owner> <why>`
  documented  specified, not promised for the milestone
  gap         a known gap as a whole; the note names the owner worklog

A library family is a row too: `std-heap` holds the cases under
`spec/conformance/std-heap/`, whatever rules they cite.

--check fails when a section is missing from the table or unknown, when a
case citing a supported section fails without a known-gap line, when a
known-gap line names no owner or a case that passes, or a gap no owner.
--draft prints a table from today's results to seed or refresh the file.
Results come from a fresh `cpl-conform conformance` run, or from
`--board <path>`, a scoreboard JSON written by
`cpl-conform conformance --write --scoreboard <path>`.
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT / "spec"
CONF = SPEC / "conformance"
SUBSET = SPEC / "SUBSET.md"
RULE_DEF = re.compile(r"\(R((?:\d\d[a-z]?|[a-z]+)(?:\.\d+)+)\)")
STATUSES = ("supported", "documented", "gap")
OWNERS = re.compile(r"\b(compiler|stdlib|diagnostics|tools|lint-fmt|perf|debugger|site|infra|idioms)\b")


def section_of(rid):
    parts = rid.split(".")
    return parts[0] if not parts[0][:1].isdigit() else ".".join(parts[:2])


def sections():
    out = {}
    for path in sorted((SPEC / "core").glob("*.md")) + sorted((SPEC / "tools").glob("*.md")):
        heading = path.stem
        for line in path.read_text().split("\n"):
            if line.startswith("#"):
                heading = line.lstrip("#").strip()
            for m in RULE_DEF.finditer(line):
                key = section_of(m.group(1))
                out.setdefault(key, (path.name, heading))
    for fam in sorted(d for d in CONF.glob("std-*") if d.is_dir() and any(d.glob("*.cp"))):
        out[fam.name] = ("conformance/" + fam.name, f"library: {fam.name[4:]}")
    return out


def case_rules():
    out = {}
    for path in sorted(CONF.rglob("*.cp")):
        cited = []
        for line in path.read_text().split("\n")[1:4]:
            s = line.strip()
            if s.startswith("//") and s[2:].strip().startswith("rule:"):
                cited += s[2:].strip()[5:].split()
        rel = str(path.relative_to(CONF))
        if rel.startswith("std-"):
            cited.append(rel.split("/")[0])
        out[rel] = cited
    return out


def results(board):
    data = json.loads(Path(board).read_text())
    return data["suites"]["conformance"]["cases"]


def table():
    rows = {}
    if not SUBSET.exists():
        return rows
    for line in SUBSET.read_text().split("\n"):
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) >= 3 and cells[0][:1] in "Rs" and cells[2] in STATUSES:
            key = cells[0][1:] if cells[0].startswith("R") else cells[0]
            if key.startswith("std-") or cells[0].startswith("R"):
                rows[key] = (cells[2], cells[3] if len(cells) > 3 else "")
    return rows


def known_gaps():
    gaps = {}
    if SUBSET.exists():
        text = SUBSET.read_text()
        body = text.split("## Known gaps", 1)[1] if "## Known gaps" in text else ""
        for line in body.split("\n"):
            m = re.match(r"- `?([\w./-]+\.cp)`?: (.*)", line.strip())
            if m:
                gaps[m.group(1)] = m.group(2)
    return gaps


def owner_of(case):
    top = case.split("/")[0]
    if top.startswith("std-"):
        return "stdlib"
    return {"diagnostics": "diagnostics", "lints": "lint-fmt"}.get(top, "compiler")


def fresh_board():
    out = Path(tempfile.mkdtemp()) / "scoreboard.json"
    cmd = ["cargo", "run", "-q", "-p", "cpl-conform", "--", "conformance"]
    subprocess.run([*cmd, "--write", "--scoreboard", str(out)], cwd=ROOT, check=False, stdout=subprocess.DEVNULL)
    return str(out)


def label(k):
    return k if k.startswith("std-") else f"R{k}"


def key_order(k):
    return (0, k.split(".")[0].zfill(3), int(k.split(".")[1])) if "." in k else (1, k, 0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--board", help="scoreboard JSON; default: run cpl-conform now")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--draft", action="store_true")
    args = ap.parse_args()
    secs = sections()
    status = results(args.board or fresh_board())
    tally = defaultdict(lambda: [0, []])
    for case, rids in case_rules().items():
        for key in {section_of(r) for r in rids}:
            if status.get(case) == "pass":
                tally[key][0] += 1
            elif case in status:
                tally[key][1].append(case)
    if args.draft:
        print("| Section | Heading | Status | Note |")
        print("|---|---|---|---|")
        rows = table()
        listed = set()
        for key in sorted(secs, key=key_order):
            ok, bad = tally[key]
            st = rows.get(key, ("supported", ""))[0]
            note = f"{ok} pass" + (f", {len(bad)} known gaps" if bad and st == "supported" else "")
            if st != "supported" and key in rows:
                note = rows[key][1]
            label = key if key.startswith("std-") else f"R{key}"
            print(f"| {label} | {secs[key][1]} | {st} | {note} |")
            if st == "supported":
                listed |= set(bad)
        print("\n## Known gaps\n")
        old = known_gaps()
        for case in sorted(listed):
            print(f"- {case}: {old.get(case, owner_of(case))}")
        return 0
    rows = table()
    gaps = known_gaps()
    errs = [f"{c}: known gap names no owner" for c, n in gaps.items() if not OWNERS.search(n)]
    errs += [f"{c}: known gap now passes; drop the line" for c in gaps if status.get(c) == "pass"]
    errs += [f"{label(k)}: no row in spec/SUBSET.md" for k in secs if k not in rows]
    errs += [f"{label(k)}: not a rule section" for k in rows if k not in secs]
    for k, (st, note) in rows.items():
        unlisted = [c for c in tally[k][1] if c not in gaps]
        if st == "supported" and unlisted:
            errs.append(f"{label(k)} is supported but {len(unlisted)} failing cases are not known gaps: "
                        + ", ".join(unlisted[:3]))
        if st == "gap" and not OWNERS.search(note):
            errs.append(f"{label(k)} is a gap with no owner in its note")
    counts = defaultdict(int)
    for st, _ in rows.values():
        counts[st] += 1
    print(f"subset: {counts['supported']} supported, {counts['documented']} documented, {counts['gap']} gap sections"
          f", {len(gaps)} known-gap cases; {len(errs)} problems")
    for e in errs:
        print(f"  {e}")
    return 1 if args.check and errs else 0


if __name__ == "__main__":
    sys.exit(main())

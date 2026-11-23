#!/usr/bin/env python3
"""Check that spec/idioms/lints.md proposes only code that exists.

Each row of the catalog is a lint not built yet: what it catches and what to
write instead (handbook/guidelines/idiomatic-code-checklist.md#r3). A
"Write instead" cell that names a function, method or type std does not
define sends a reader, the skill or a lint author to an API that is not
there. A span that starts `needs stdlib:` marks a gap on purpose and is
skipped, as are the placeholder names `foo` and `bar`. Each finding is `unknown-name <lint> <name>`: the name is neither
defined in std/ nor known to the checker (a string in crates/cpl-check/src).
The fix is to name the real API, or to route the gap to stdlib and keep the
row in the known list until it lands.

Known findings live in spec/idioms/lints-known.txt, a list that only shrinks.

  python3 scripts/idioms-catalog.py            print findings
  python3 scripts/idioms-catalog.py --check    fail on a finding not in the list
  python3 scripts/idioms-catalog.py --update   rewrite the list
"""
import argparse, glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "spec", "idioms", "lints.md")
KNOWN = os.path.join(ROOT, "spec", "idioms", "lints-known.txt")
DEF = re.compile(r"^\s*(?:@\S+\s+)*(?:pub\s+)?(?:fn|struct|enum|trait|type|const|domain|let|var)\s+(?:\([^)]*\)\s*)?([A-Za-z_]\w*)", re.M)
FIELD = re.compile(r"\b([a-z_]\w*)\s*:")
PLACEHOLDERS = {"foo", "bar"}
NAME = re.compile(r"(\.?)([A-Za-z_]\w*)(\()?")


def vocabulary():
    names = set()
    for path in glob.glob(os.path.join(ROOT, "std", "**", "*.cp"), recursive=True):
        src = open(path, errors="replace").read()
        names |= set(DEF.findall(src)) | set(FIELD.findall(src))
        names |= {m for m in re.findall(r"\b([A-Z]\w*)\b", src)}
        names |= set(re.findall(r"\|\s*([A-Z]\w*)", src))
    for path in glob.glob(os.path.join(ROOT, "crates", "cpl-check", "src", "**", "*.rs"), recursive=True):
        names |= {w for w in re.findall(r'"([A-Za-z_]\w*)"', open(path, errors="replace").read())}
    return names


def rows():
    """(lint, write-instead cell) for every catalog row with that column."""
    col = None
    for line in open(CATALOG):
        if not line.startswith("|"):
            col = None
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if "Write instead" in cells:
            col = cells.index("Write instead")
        elif col is not None and not set(line) <= set("|-: \n") and len(cells) > col:
            yield cells[0].strip("`"), cells[col]


def names(cell):
    """Identifiers a code span calls, reads as a member, or uses as a type."""
    for span in re.findall(r"`([^`]+)`", cell):
        if span.startswith("needs stdlib:"):
            continue
        span = re.sub(r'"[^"]*"', "", span)
        for module in re.findall(r"\bstd\.(\w+)", span):
            if not os.path.exists(os.path.join(ROOT, "std", module + ".cp")):
                yield "std." + module
        span = re.sub(r"\bstd\.\w+", "", span)
        for dot, name, call in NAME.findall(span):
            if len(name) == 1 or name.isupper() or name in PLACEHOLDERS:
                continue
            if dot or call or name[0].isupper():
                yield name


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--update", action="store_true")
    a = ap.parse_args(argv)
    known_names = vocabulary()
    found = sorted({f"unknown-name {lint} {n}" for lint, cell in rows() for n in names(cell) if n not in known_names})
    known = {l.strip() for l in open(KNOWN) if l.strip() and not l.startswith("#")} if os.path.exists(KNOWN) else set()
    if a.update:
        with open(KNOWN, "w") as f:
            f.write("# scripts/idioms-catalog.py findings not yet fixed; this list only shrinks.\n")
            f.writelines(x + "\n" for x in found)
        print(f"idioms-catalog: wrote {os.path.relpath(KNOWN, ROOT)} ({len(found)} findings)")
        return 0
    new = [f for f in found if f not in known]
    if not a.check:
        print("\n".join(found))
    print(f"idioms-catalog: {sum(1 for _ in rows())} rows, {len(found)} findings, {len(new)} new, {len(known - set(found))} fixed", file=sys.stderr)
    for f in new if a.check else []:
        print("  new: " + f, file=sys.stderr)
    return 1 if a.check and new else 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
# Enforces handbook/guidelines/std-api-checklist.md#r20-a-rename-ships-its-codemod-and-its-error
"""Snapshot std's public signatures, so no public API change is silent.

`spec/lib/std-api.snapshot` holds one line per public fn, `module: Owner.name(params) -> R`.
`--check` fails when std's signatures differ from it: the change shows up in the
commit that runs `--write`. `--write` refuses a public name that leaves std unless
a `cpl fix` codemod names it (a quoted string under crates/cpl-check/src) or
CHANGELOG.md mentions it in backticks.

    uv run python scripts/std-api-snapshot.py [--check | --write]
"""
import argparse, glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SNAPSHOT = os.path.join(ROOT, "spec", "lib", "std-api.snapshot")
PUB_FN = re.compile(r"^([ \t]*)(?:@\w+(?:\([^)]*\))?\s+)*pub\s+fn\s+(\w+)", re.M)
OWNER = re.compile(r"^(?:pub\s+)?(?:struct|enum|trait|impl(?:<[^>]*>)?)\s+(?:[\w<>, ]+\s+for\s+)?(\w+)", re.M)


def header(text, start):
    """The signature from `pub fn` through its return type, whitespace collapsed."""
    depth, i = 0, text.index("(", start)
    while i < len(text):
        depth += {"(": 1, ")": -1}.get(text[i], 0)
        i += 1
        if depth == 0:
            break
    rest = re.match(r"[ \t]*(->[^\n]*?)?\s*(?:=|:\s*$|$)", text[i:], re.M)
    sig = text[start:i] + (" " + rest.group(1) if rest and rest.group(1) else "")
    return re.sub(r",\s*\)", ")", re.sub(r"\(\s+", "(", re.sub(r"\s+", " ", sig)))


def entries(texts):
    """`module: Owner.name(...)` for every public fn of each (rel, text)."""
    out = []
    for rel, text in texts:
        owners = [(m.start(), m.group(1)) for m in OWNER.finditer(text)]
        for m in PUB_FN.finditer(text):
            owner = ""
            if m.group(1):
                before = [o for at, o in owners if at < m.start()]
                owner = before[-1] + "." if before else ""
            sig = header(text, m.start(2))
            out.append(f"{rel}: {owner}{sig}")
    return sorted(set(out))


def name_of(entry):
    return re.match(r"[^:]+: ([\w.]+)", entry).group(1)


def unannounced(old, new, codemods, changelog):
    """Public names in `old` and not `new` that no codemod or changelog line names."""
    gone = {name_of(e) for e in old} - {name_of(e) for e in new}
    return sorted(n for n in gone if f'"{n.split(".")[-1]}"' not in codemods
                  and f"`{n.split('.')[-1]}" not in changelog)


def live():
    paths = sorted(glob.glob(os.path.join(ROOT, "std", "*.cp")))
    return entries((os.path.relpath(p, ROOT), open(p).read()) for p in paths)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true")
    a = ap.parse_args()
    new = live()
    old = open(SNAPSHOT).read().splitlines() if os.path.exists(SNAPSHOT) else []
    if a.write:
        codemods = "".join(open(p).read() for p in glob.glob(os.path.join(ROOT, "crates/cpl-check/src/**/*.rs"), recursive=True))
        missing = unannounced(old, new, codemods, open(os.path.join(ROOT, "CHANGELOG.md")).read())
        for n in missing:
            print(f"std-api/R20: `{n}` leaves std with no `cpl fix` codemod and no CHANGELOG.md line")
        if missing:
            return 1
        with open(SNAPSHOT, "w") as f:
            f.write("\n".join(new) + "\n")
        print(f"std-api snapshot: wrote {len(new)} signatures")
        return 0
    added, removed = sorted(set(new) - set(old)), sorted(set(old) - set(new))
    for e in removed:
        print(f"- {e}")
    for e in added:
        print(f"+ {e}")
    if added or removed:
        print(f"std-api snapshot: {len(added)} added, {len(removed)} removed; "
              "review them and run `scripts/std-api-snapshot.py --write`")
        return 1
    print(f"std-api snapshot: ok ({len(new)} signatures)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

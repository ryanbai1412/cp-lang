#!/usr/bin/env python3
"""Check that the skill's prose uses the current spellings.

Rule: handbook/guidelines/idiomatic-code-checklist.md#r8. `scripts/skill-gen.py
--check` runs every code block of skills/cp-lang/ through `cpl check`; this
covers the inline code in the prose around them. Each distinct `code` span
goes through `cpl migrate --check`, and a span it would rewrite is a
finding, `removed-spelling <file>:<line> <span>`.

A span that quotes C++ on purpose is skipped: a table cell under a `C++`
heading, or a paragraph that starts `**C++ habit:**`.

  python3 scripts/skill-prose.py [--cpl PATH]   exit 1 on a finding
"""
import argparse, hashlib, json, os, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SKILL = os.path.join(ROOT, "skills", "cp-lang")
CACHE = os.path.join(ROOT, "target", "skill-prose")
FENCE = re.compile(r"^```.*?^```[ \t]*$", re.M | re.S)
SPAN = re.compile(r"`([^`\n]+)`")


def spans(path):
    """(line, span) for each inline code span that is meant as CP Lang."""
    text = FENCE.sub(lambda m: "\n" * m.group(0).count("\n"), open(path, encoding="utf-8").read())
    cpp_cols, cpp_par = set(), False
    for n, line in enumerate(text.split("\n"), 1):
        if not line.strip():
            cpp_par = False
        if line.startswith("**C++ habit:**"):
            cpp_par = True
        if line.startswith("|"):
            cells = line.strip().strip("|").split("|")
            if set(line) <= set("|-: "):
                continue
            if not any(SPAN.search(c) for c in cells):
                cpp_cols = {i for i, c in enumerate(cells) if c.strip() == "C++"}
                continue
            for i, cell in enumerate(cells):
                if i not in cpp_cols:
                    yield from ((n, s) for s in SPAN.findall(cell))
            continue
        cpp_cols = set()
        if not cpp_par:
            yield from ((n, s) for s in SPAN.findall(line))


def stale(texts, cpl):
    """The spans `cpl migrate --check` would rewrite, cached by span set and cpl."""
    key = hashlib.sha256(("\0".join(sorted(texts)) + f"\0{os.path.getmtime(cpl)}").encode()).hexdigest()[:16]
    memo = os.path.join(CACHE, key + ".json")
    if os.path.exists(memo):
        return set(json.load(open(memo)))
    work = os.path.join(CACHE, key)
    os.makedirs(work, exist_ok=True)
    names = {}
    for i, t in enumerate(sorted(texts)):
        names[f"{i}.cp"] = t
        open(os.path.join(work, f"{i}.cp"), "w", encoding="utf-8").write(t + "\n")
    r = subprocess.run([cpl, "migrate", "--check", "--json", work], capture_output=True, text=True)
    out = {names[os.path.basename(f["path"])] for f in json.loads(r.stdout)["files"] if f["changed"]}
    json.dump(sorted(out), open(memo, "w"))
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "release", "cpl"))
    a = ap.parse_args(argv)
    found = []
    for d, _, fs in os.walk(SKILL):
        for f in sorted(fs):
            if f.endswith(".md"):
                path = os.path.join(d, f)
                found += [(os.path.relpath(path, ROOT), n, s) for n, s in spans(path)]
    bad = stale({s for _, _, s in found}, a.cpl)
    hits = [f"removed-spelling {p}:{n} `{s}`" for p, n, s in found if s in bad]
    print("\n".join(hits)) if hits else None
    print(f"skill-prose: {len({s for _, _, s in found})} distinct spans, {len(hits)} findings", file=sys.stderr)
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main())

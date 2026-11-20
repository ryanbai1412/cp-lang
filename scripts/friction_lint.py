#!/usr/bin/env python3
"""Counts known friction workarounds in CP Lang 0.2.2 sources.

Each category is a spelling that a port falls back to when the language or
the standard library is missing something; the counts should only go down.

    Z(0) + a[i] * Z(2)          Mod literal wrapping (literals should coerce)
    a[i] = a[i] + x             missing compound assignment (a[i] += x)
    let t = a[i]                manual temp swap (swap(mut a[i], mut a[j]))
    a[i] = a[j]
    a[j] = t
    while head < queue.len:     manual BFS cursor loop
    [x for x in xs]             comprehension used as a clone, slice or pad
    V.first.l, ..=V.last        domain boundary boilerplate (.l / .r)
    Action<S, T>(0, ...)        positional Action fields (name them)

usage: friction_lint.py [--json] [--top N] [--list CATEGORY] [--baseline FILE] [PATH ...]

PATHs default to proto/ports and examples (every *.cp below them, minus
before/ snapshots that record code as it had to be written before a fix).
--baseline takes an earlier --json report and exits 1 if any category grew.
"""
import argparse, json, os, re, sys
from collections import Counter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_PATHS = ["proto/ports", "examples"]
SKIP_DIRS = {"before"}

IDENT = r"[A-Za-z_]\w*"
# An assignable place: x, x.f, a[i], a[i][j].f ...
PLACE = rf"{IDENT}(?:\.{IDENT}|\[[^\]\n]*\])*"


def strip_code(text):
    """Blanks out comments and string literals, keeping line structure."""
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if text.startswith("//", i):
            j = text.find("\n", i)
            i = n if j < 0 else j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append(re.sub(r"[^\n]", " ", text[i:j]))
            i = j
        elif c == '"':
            j = i + 1
            while j < n and text[j] != '"' and text[j] != "\n":
                j += 2 if text[j] == "\\" else 1
            out.append('""' + " " * max(0, j - i - 1))
            i = j + 1
        else:
            out.append(c)
            i += 1
    return "".join(out)


def mod_aliases(code):
    """Names bound to a Mod type: `type Z = Mod<P>`."""
    return set(re.findall(rf"\btype\s+({IDENT})\s*=\s*Mod\s*<", code))


def angle_end(code, i):
    """Index just past the `>` matching the `<` at code[i]."""
    depth = 0
    for j in range(i, len(code)):
        if code[j] == "<":
            depth += 1
        elif code[j] == ">":
            depth -= 1
            if depth == 0:
                return j + 1
    return -1


def line_of(code, pos):
    return code.count("\n", 0, pos) + 1


def find_mod_literals(code, lines):
    hits = []
    names = ["Mod"] + sorted(mod_aliases(code))
    for name in names:
        for m in re.finditer(rf"\b{name}\b", code):
            j = m.end()
            if name == "Mod":
                if code[j:j + 1] != "<":
                    continue
                j = angle_end(code, j)
                if j < 0:
                    continue
            lit = re.match(r"\(\s*-?\d[\d_]*\s*\)", code[j:])
            if lit:
                hits.append((line_of(code, m.start()), "Mod literal" if name == "Mod" else f"{name}(<int>)"))
    for i, line in enumerate(lines, 1):
        for m in re.finditer(r"\.v\s*(==|!=)\s*0\b", line):
            hits.append((i, f".v {m.group(1)} 0"))
    return hits


def top_level(expr):
    """`expr` with everything inside brackets blanked out."""
    out, depth = [], 0
    for c in expr:
        depth += c in "([{"
        out.append(c if depth == 0 else " ")
        depth -= c in ")]}"
    return "".join(out)


def find_self_assign(code, lines):
    """`x = x op e` that means exactly `x op= e`."""
    hits = []
    pat = re.compile(rf"(?<![\w.\]])({PLACE})\s*=\s*\1\s*([-+*])(?!=)(.*)")
    looser = re.compile(r"\?\?|[<>=!]=|(?<![-=])[<>]|\b(?:if|else|and|or)\b")
    for i, line in enumerate(lines, 1):
        for m in pat.finditer(line):
            place, op, rest = m.groups()
            rest = top_level(rest)
            if looser.search(rest) or (op in "-*" and re.search(r"[\w)\]]\s*[-+]\s", rest)) \
                    or (op == "*" and re.search(r"[*/%]", rest)):
                continue
            kind = "indexed" if "[" in place else "field" if "." in place else "variable"
            hits.append((i, f"{kind} {op}"))
    return hits


def find_temp_swaps(code, lines):
    hits = []
    let = re.compile(rf"^\s*(?:let|var)\s+({IDENT})\s*=\s*({PLACE})\s*$")
    for i in range(len(lines) - 2):
        m = let.match(lines[i])
        if not m:
            continue
        t, a = m.groups()
        m2 = re.match(rf"^\s*{re.escape(a)}\s*=\s*({PLACE})\s*$", lines[i + 1])
        if not m2 or m2.group(1) == a:
            continue
        b = m2.group(1)
        if re.match(rf"^\s*{re.escape(b)}\s*=\s*{t}\s*$", lines[i + 2]):
            hits.append((i + 1, "indexed" if "[" in a else "variable"))
    return hits


def find_cursor_loops(code, lines):
    pat = re.compile(rf"\bwhile\s+({IDENT})\s*<\s*({PLACE})\.len\s*:")
    return [(i, "while i < xs.len") for i, line in enumerate(lines, 1) if pat.search(line)]


def find_comprehension_copies(code, lines):
    hits = []
    clone = re.compile(rf"\[\s*({IDENT})\s+for\s+\1\s+in\s+[^\]]*\]")
    pad = re.compile(rf"\[\s*if\s+({IDENT})\s*<[^\]]*\bfor\s+\1\s+in\b")
    for i, line in enumerate(lines, 1):
        for m in clone.finditer(line):
            if not re.search(r"\bif\b", m.group(0)):
                hits.append((i, "[x for x in xs]"))
        for _ in pad.finditer(line):
            hits.append((i, "[if i < k: ... else: ... for i in ...]"))
    return hits


def find_domain_bounds(code, lines):
    hits = []
    pat = re.compile(r"\b([A-Z]\w*)\.(first|last)\b(\.[lr]\b)?")
    for i, line in enumerate(lines, 1):
        for m in pat.finditer(line):
            if m.group(3):
                hits.append((i, f"D.{m.group(2)}{m.group(3)}"))
            elif line[:m.start()].rstrip().endswith(("..=", "..")):
                hits.append((i, f"..D.{m.group(2)}"))
            else:
                hits.append((i, f"D.{m.group(2)}"))
    return hits


def find_positional_actions(code, lines):
    hits = []
    for m in re.finditer(r"\bAction\s*<", code):
        j = angle_end(code, m.end() - 1)
        if j < 0:
            continue
        rest = code[j:]
        call = re.match(r"\s*\(\s*", rest)
        if call and not re.match(rf"{IDENT}\s*:(?!:)", rest[call.end():]):
            hits.append((line_of(code, m.start()), "Action<...>(positional)"))
    return hits


CATEGORIES = [
    ("mod_literal_wrapping", "Mod literal wrapping", find_mod_literals),
    ("missing_compound_assign", "x = x op ... instead of x op= ...", find_self_assign),
    ("manual_temp_swap", "3-line temp swap", find_temp_swaps),
    ("manual_cursor_loop", "manual BFS cursor loop", find_cursor_loops),
    ("comprehension_copy", "comprehension as clone/slice/pad", find_comprehension_copies),
    ("domain_bound_boilerplate", "domain .first/.last boilerplate", find_domain_bounds),
    ("positional_action", "positional Action<...>(...) fields", find_positional_actions),
]


def sources(paths):
    for p in paths:
        p = os.path.join(ROOT, p) if not os.path.isabs(p) else p
        if os.path.isfile(p):
            yield p
            continue
        for d, dirs, files in os.walk(p):
            dirs[:] = sorted(x for x in dirs if x not in SKIP_DIRS)
            for f in sorted(files):
                if f.endswith(".cp"):
                    yield os.path.join(d, f)


def scan(paths):
    report = {key: {"title": title, "count": 0, "patterns": Counter(), "files": Counter(), "hits": []}
              for key, title, _ in CATEGORIES}
    files = 0
    for path in sources(paths):
        files += 1
        rel = os.path.relpath(path, ROOT)
        code = strip_code(open(path, errors="replace").read())
        lines = code.split("\n")
        for key, _, find in CATEGORIES:
            for line, pattern in find(code, lines):
                r = report[key]
                r["count"] += 1
                r["patterns"][pattern] += 1
                r["files"][rel] += 1
                r["hits"].append(f"{rel}:{line}: {pattern}")
    return files, report


def as_json(files, report, top):
    return {
        "version": "0.2.2",
        "files_scanned": files,
        "total": sum(r["count"] for r in report.values()),
        "categories": {
            key: {
                "count": r["count"],
                "files": len(r["files"]),
                "patterns": dict(r["patterns"].most_common()),
                "top_files": [[f, n] for f, n in r["files"].most_common(top)],
            }
            for key, r in report.items()
        },
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("paths", nargs="*", default=DEFAULT_PATHS)
    ap.add_argument("--json", action="store_true", help="machine-readable report")
    ap.add_argument("--top", type=int, default=5, help="offending files shown per category")
    ap.add_argument("--list", metavar="CATEGORY", help="print every hit of one category")
    ap.add_argument("--baseline", metavar="FILE", help="exit 1 if a category grew past this --json report")
    args = ap.parse_args()

    files, report = scan(args.paths)
    if args.list:
        if args.list not in report:
            ap.error(f"unknown category {args.list!r}; one of: {', '.join(report)}")
        print("\n".join(report[args.list]["hits"]))
        return 0

    data = as_json(files, report, args.top)
    if args.json:
        print(json.dumps(data, indent=2))
    else:
        print(f"{data['total']} friction workarounds in {files} files\n")
        for key, r in report.items():
            print(f"{r['count']:5}  {key}  ({r['title']}, {len(r['files'])} files)")
            for pattern, n in r["patterns"].most_common():
                print(f"{'':7}{n:5}  {pattern}")
            for f, n in r["files"].most_common(args.top):
                print(f"{'':7}{n:5}  {f}")
            print()

    if args.baseline:
        with open(args.baseline) as fh:
            base = json.load(fh)["categories"]
        grew = [(k, base[k]["count"], v["count"]) for k, v in data["categories"].items()
                if k in base and v["count"] > base[k]["count"]]
        for k, old, new in grew:
            print(f"friction_lint: {k} grew from {old} to {new}", file=sys.stderr)
        return 1 if grew else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())

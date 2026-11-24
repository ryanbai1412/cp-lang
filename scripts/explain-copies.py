#!/usr/bin/env python3
# Enforces handbook/guidelines/conformance-checklist.md#conformancer6-results-agree-across-engines-with-and-without-sanitizers
"""Check that every copy `cpl explain` lists is a copy in the generated C++.

For each `ok` conformance case, runs `cpl explain`; where it lists copies
(`[copy]` lines and `explicit copy` views), builds the judge C++ and counts
the deep copies in it with the counter explain_accuracy.rs uses (a
container initialised from a place, a by-value capture), plus a container
constructed from another (`= std::vector<T>(x);`) and `cpl::to_list(...)`. A
case whose explain lists more copies than its C++ makes is a false claim
(spec/tools/explain.md: each heap value the judge build copies, R05.3.5).

    uv run scripts/explain-copies.py [--check] [--max N] [--jobs N]

`--max` is shrink-only: lower it as tools fixes false claims.
"""
import argparse
import concurrent.futures as cf
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
COPY = re.compile(r"(?:std::vector<[^;()]*?>|std::string)\(\w+(?:\.\w+)*\)|cpl::to_list\(|cpl::sorted\(\w+(?:\.\w+)*\)")
CONTAINERS = ("std::vector<", "std::map<", "std::set<", "std::unordered_map<", "std::deque<", "std::string")
NAME = re.compile(r"[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*")
SCALARS = ("int", "ll", "bool", "double", "char", "auto", "unsigned", "__int128", "cpl::OptPos")


def scalar_decl(line):
    """The name a scalar local declares (`int f = 0;`, `for (int f = lo; ..`),
    which a lambda then captures as a scalar, not a container named `f` further out."""
    ty, _, rest = line.removeprefix("for (").partition(" ")
    if ty not in SCALARS:
        return None
    name = re.split(r"[ =;]", rest)[0]
    return name if name and re.fullmatch(r"\w+", name) else None


def is_place(text):
    text = text.strip()
    return bool(NAME.fullmatch(text)) or (text.startswith("cpl::at(") and text.endswith(")") and "::" not in text[8:])


def split_top(text):
    """`text` split at the commas outside `<>`, `()` and `{}`."""
    parts, depth, start = [], 0, 0
    for i, c in enumerate(text):
        if c in "<({":
            depth += 1
        elif c in ">)}":
            depth -= 1
        elif c == "," and depth == 0:
            parts.append(text[start:i].strip())
            start = i + 1
    parts.append(text[start:].strip())
    return parts


def tuple_copies(decl):
    """Copies in a by-value `std::tuple<..> t = place;` whose type holds a
    container, and container fields of a brace initialiser `{a.x, b}` taken
    from places."""
    depth = 0
    for i, c in enumerate(decl):
        if c == "<":
            depth += 1
        elif c == ">":
            depth -= 1
            if depth == 0:
                break
    else:
        return 0
    args, rest = split_top(decl[len("std::tuple<") : i]), decl[i + 1 :].strip()
    name, eq, init = rest.partition("=")
    name, init = name.strip(), init.strip().removesuffix(";").strip()
    if not eq or not re.fullmatch(r"\w+", name):
        return 0
    if is_place(init):
        return int(any(a.startswith(CONTAINERS) for a in args))
    brace = init.find("{")
    if brace < 0 or not init.endswith("}"):
        return 0
    items = split_top(init[brace + 1 : -1])
    return sum(1 for a, x in zip(args, items) if a.startswith(CONTAINERS) and is_place(x))


def deep_copies(cpp):
    """Mirrors deep_copies in crates/cpl-cli/tests/support/cpp_counts.rs, plus
    copies spelled as a constructor call or `cpl::to_list`, tuples holding a
    container copied from places, and a container assigned from a place."""
    held, count = set(), len(COPY.findall(cpp))
    for line in cpp.splitlines():
        line = line.strip()
        decl = line.removeprefix("const ")
        if (scalar := scalar_decl(decl)) is not None:
            held.discard(scalar)
        if decl.startswith(CONTAINERS):
            depth, end = 0, len(decl)
            for i, c in enumerate(decl):
                if c == "<":
                    depth += 1
                elif c == ">":
                    depth -= 1
                elif c in "=(;,){" and depth == 0:
                    end = i
                    break
            head = decl[:end].rstrip()
            name = re.split(r"[ &]", head)[-1]
            if name and len(name) < len(head):
                held.add(name)
                by_value = not head[: len(head) - len(name)].rstrip().endswith("&")
                rest = decl[end:]
                if by_value and rest.startswith("=") and rest.endswith(";") and is_place(rest[1:-1]):
                    count += 1
        if line.startswith("std::tuple<") and any(c in line for c in CONTAINERS) and ".first, " in line and ".second)" in line:
            count += 1
        elif decl.startswith("std::tuple<"):
            count += tuple_copies(decl)
    for line in cpp.splitlines():
        lhs, eq, rhs = line.strip().partition(" = ")
        if eq and rhs.endswith(";") and NAME.fullmatch(lhs) and lhs.rsplit(".", 1)[-1] in held and is_place(rhs[:-1]):
            count += 1
    for line in cpp.splitlines():
        open_ = line.find("= [")
        close = line.find("](", open_) if open_ >= 0 else -1
        if close < 0:
            continue
        for cap in line[open_ + 3 : close].split(", "):
            if not cap.startswith("&") and " = " in cap and cap.split(" = ", 1)[1].strip() in held:
                count += 1
    return count


def one(cpl, case, tmp):
    text = case.read_text()
    if not text.startswith("// expect: ok") or "\n// tool:" in text:
        return None
    ex = subprocess.run([cpl, "explain", str(case)], capture_output=True, text=True, timeout=120).stdout
    claimed = ex.count("[copy]") + ex.count("explicit copy")
    if not claimed:
        return None
    out = Path(tmp) / (case.stem + ".cpp")
    if subprocess.run([cpl, "build", str(case), "-o", str(out)], capture_output=True, timeout=120).returncode:
        return None
    made = deep_copies(out.read_text())
    return case.relative_to(ROOT), claimed, made


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--max", type=int, default=0, help="false claims allowed (shrink-only)")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--cpl", default=str(ROOT / "target" / "debug" / "cpl"))
    a = ap.parse_args()
    cases = sorted((ROOT / "spec" / "conformance").rglob("*.cp"))
    with tempfile.TemporaryDirectory() as tmp, cf.ThreadPoolExecutor(a.jobs) as pool:
        rows = [r for r in pool.map(lambda c: one(a.cpl, c, tmp), cases) if r]
    false = [r for r in rows if r[1] > r[2]]
    print(f"explain-copies: {len(rows)} cases list copies; {len(false)} list more than the C++ makes")
    for path, claimed, made in false:
        print(f"  {path}: explain lists {claimed}, the C++ makes {made}")
    if a.check and len(false) > a.max:
        print(f"explain-copies: {len(false)} false claims, more than the {a.max} allowed")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

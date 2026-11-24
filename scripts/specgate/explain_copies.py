"""Check that every copy `cpl explain` lists is a copy in the generated C++.

For each `ok` conformance case, runs `cpl explain` (its `[copy]` lines,
which codegen records), builds the judge C++ and counts
the deep copies in it with the counter explain_accuracy.rs uses (a
container initialised from a place, a by-value capture), plus a container
constructed from another (`= std::vector<T>(x);`) and `cpl::to_list(...)`. A
case whose explain lists more copies than its C++ makes over-lists, and (with
`--exact`) one that lists fewer hides a cost (spec/tools/explain.md: each heap value the
judge build copies, R05.3.5).

`--max` is shrink-only: lower it as tools fixes mismatches.
Enforces handbook/guidelines/conformance-checklist.md#conformancer6-results-agree-across-engines-with-and-without-sanitizers
"""
import concurrent.futures as cf
import re
import subprocess
import tempfile
from pathlib import Path

import specgate
from specgate import Problem

HELP = "`cpl explain` lists exactly the deep copies the judge C++ makes (--max mismatched cases allowed, shrink-only)"

COPY = re.compile(r"(?:(?:std::(?:vector|array|map|set|unordered_map|deque)|cpl::Grid)<[^;()]*?>|std::string)\((?:cpl::at\([^()]*\)|[A-Za-z_]\w*(?:\[[^\[\]]+\])*(?:\.\w+)*)\)(?!>)|cpl::to_list\(|cpl::to_str\(|cpl::sorted\([A-Za-z_]\w*(?:\.\w+)*\)")
CONTAINERS = ("std::vector<", "std::map<", "std::set<", "std::unordered_map<", "std::deque<", "std::string")
NAME = re.compile(r"[A-Za-z_]\w*(?:\[[^\[\]]+\])*(?:\.[A-Za-z_]\w*)*")
PARAM = re.compile(r"(?:std::vector<|std::map<|std::set<|std::unordered_map<|std::deque<|std::string)[^();{}=]*?[&\s](\w+)\s*[,)]")
ARRAY = re.compile(r"std::array<[^;()]*?>&? (\w+)\s*[=;,)]")
CAPTURES = re.compile(r"\[([^\[\]]*)\]\(")
STRUCT = re.compile(r"^struct (\w+)\s*\{")


def container_structs(cpp):
    """Structs that hold a container field, directly or through another such struct."""
    bodies, name = {}, None
    for line in cpp.splitlines():
        line = line.strip()
        if m := STRUCT.match(line):
            name = m.group(1)
            bodies[name] = []
        elif name and line.startswith("}"):
            name = None
        elif name:
            bodies[name].append(line)
    holders, grew = set(), True
    while grew:
        grew = False
        for n, lines in bodies.items():
            if n not in holders and any(l.startswith(CONTAINERS) or l.split(" ")[0].split("<")[0] in holders for l in lines):
                holders.add(n)
                grew = True
    return holders


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


def program(cpp):
    """`cpp` without the runtime's `namespace cpl { .. }`, whose own helpers
    (`std::string(sv)` in a view's `to_str`) are not the program's copies."""
    out, runtime = [], False
    for line in cpp.splitlines():
        if line.startswith("namespace cpl {"):
            runtime = True
        elif runtime and line.startswith("}  // namespace cpl"):
            runtime = False
        elif not runtime:
            out.append(line)
    return "\n".join(out)


def template_arg(rest):
    """The type inside the `<..>` that opens `rest`, and what follows it."""
    depth = 1
    for i, c in enumerate(rest):
        depth += {"<": 1, ">": -1}.get(c, 0)
        if depth == 0:
            return rest[:i], rest[i + 1 :]
    return None


DOTTED = re.compile(r"[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*")
OPTIONAL_GET = re.compile(r"\(std::get<\d+>\(([^()]*)\)\)")


def optional_copies(cpp, holders):
    """An optional of a heap value copied from a place: `std::optional<C>(xs)`
    (a `?T` argument), `std::optional<T> t0 = best;` and
    `std::optional<C>(std::get<0>(sp))`. Mirrors optional_copies in
    crates/cpl-cli/tests/support/cpp_counts.rs and optional_place_copies in
    crates/cpl-cli/tests/explain_accuracy.rs."""
    heap = lambda ty: ty.startswith(CONTAINERS + ("std::array<",)) or ty.split("<")[0] in holders
    place = lambda t: bool(DOTTED.fullmatch(t.strip())) or (t.strip().startswith("cpl::at(") and t.strip().endswith(")") and "::" not in t.strip()[8:])
    count = 0
    for line in cpp.splitlines():
        line = line.strip()
        decl = line.removeprefix("const ")
        if decl.startswith("std::optional<") and (got := template_arg(decl[len("std::optional<") :])):
            ty, rest = got
            name, eq, init = rest.removesuffix(";").partition(" = ")
            if heap(ty) and rest.endswith(";") and eq and re.fullmatch(r"\s*\w+", name) and place(init):
                count += 1
        at = 0
        while (at := line.find("std::optional<", at)) >= 0:
            at += len("std::optional<")
            if not (got := template_arg(line[at:])):
                break
            ty, rest = got
            arg = rest[1:].split(")", 1)[0] if rest.startswith("(") else None
            if arg is not None and ty in holders and place(arg):
                count += 1
            elif heap(ty) and (m := OPTIONAL_GET.match(rest)) and place(m.group(1)):
                count += 1
    return count


HEADER = re.compile(r"^(?:template <[^>]*> )?(?:inline |static )*(.+?) \w+\(.*\{$")
PART = re.compile(r"[A-Za-z_]\w*(?:\.\w+|\[[^\[\]]+\])*")
BRANCH = re.compile(r"(?:const )?(\S+) \w+ = .+ \? .+ : (.+);$")


def returned_parts(cpp, holders):
    """A part of a value given out by value: `return self.agg[t];` from a
    fn returning a heap type, and `Mat v = c ? f(..) : self.t[p];`.
    Mirrors returned_part_copies in crates/cpl-cli/tests/explain_accuracy.rs."""
    heap = lambda ty: ty.startswith(CONTAINERS + ("std::array<",)) or ty in holders
    part = lambda e: bool(PART.fullmatch(e)) and ("." in e or "[" in e)
    count, depth, scopes = 0, 0, []
    for line in cpp.splitlines():
        s = line.strip()
        lead = len(s) - len(s.lstrip("}"))
        depth -= lead
        while scopes and scopes[-1][0] >= depth:
            scopes.pop()
        if m := HEADER.match(line):
            scopes.append((depth, m.group(1).removeprefix("const ")))
        elif s.endswith("{") and "](" in s:
            scopes.append((depth, s.rpartition(") -> ")[2][:-1].strip() if ") -> " in s else None))
        depth += s.count("{") - s.count("}") + lead
        ret = scopes[-1][1] if scopes else None
        if ret and heap(ret) and not ret.endswith(("&", "*")) and s.startswith("return ") and s.endswith(";") and part(s[7:-1]):
            count += 1
        if (m := BRANCH.match(s)) and heap(m.group(1)) and part(m.group(2)):
            count += 1
    return count


def deep_copies(cpp):
    """Mirrors deep_copies in crates/cpl-cli/tests/support/cpp_counts.rs, plus
    copies spelled as a constructor call or `cpl::to_list`, tuples holding a
    container copied from places, a container assigned from a place, and
    optionals of heap values copied from places, parts returned by value;
    the runtime's own lines
    are skipped."""
    held, count = set(PARAM.findall(cpp)), len(COPY.findall(program(cpp)))
    holders = container_structs(cpp)
    count += optional_copies(program(cpp), holders) + returned_parts(program(cpp), holders)
    for line in cpp.splitlines():
        line = line.strip()
        decl = line.removeprefix("const ")
        ty, _, rest = decl.partition(" ")
        if ty in holders:
            name, eq, init = rest.partition("=")
            name = name.strip()
            if re.fullmatch(r"\w+", name):
                held.add(name)
                if eq and init.strip().endswith(";") and is_place(init.strip()[:-1]):
                    count += 1
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
            name = re.split(r"[ &>]", head)[-1]
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
    arrays = set(ARRAY.findall(program(cpp)))
    for line in cpp.splitlines():
        for caps in CAPTURES.findall(line):
            for cap in re.split(r"\s*,\s*", caps):
                name, eq, src = cap.partition("=")
                if eq and not name.strip().startswith("&") and (src.strip() in held or src.strip() in arrays):
                    count += 1
    return count


def one(cpl, case, tmp, exact):
    text = case.read_text()
    if not text.startswith("// expect: ok") or "\n// tool:" in text:
        return None
    ex = subprocess.run([cpl, "explain", str(case)], capture_output=True, text=True, timeout=120).stdout
    claimed = ex.count("[copy]")
    if not claimed and not exact:
        return None
    out = Path(tmp) / (case.stem + ".cpp")
    if subprocess.run([cpl, "build", str(case), "--full", "-o", str(out)], capture_output=True, timeout=120).returncode:
        return None
    made = deep_copies(out.read_text())
    return case.relative_to(specgate.CONFORMANCE.parent.parent), claimed, made


def args(ap) -> None:
    ap.add_argument("--max", type=int, default=0, help="cases whose copy list differs from the C++ (shrink-only)")
    ap.add_argument("--exact", action="store_true", help="also flag a case whose explain lists fewer copies than its C++ makes")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--cpl")


def run(a) -> list[Problem]:
    cpl = a.cpl or str(specgate.ROOT / "target" / "debug" / "cpl")
    cases = sorted(specgate.CONFORMANCE.rglob("*.cp"))
    with tempfile.TemporaryDirectory() as tmp, cf.ThreadPoolExecutor(a.jobs) as pool:
        rows = [r for r in pool.map(lambda c: one(cpl, c, tmp, a.exact), cases) if r]
    over = [r for r in rows if r[1] > r[2]]
    under = [r for r in rows if r[1] < r[2]] if a.exact else []
    if len(over) + len(under) <= a.max:
        return []
    return [
        Problem(str(path), f"explain {kind} copies: it lists {claimed}, the C++ makes {made}")
        for kind, wrong in (("over-lists", over), ("under-lists", under))
        for path, claimed, made in wrong
    ]

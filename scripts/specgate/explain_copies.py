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
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

import specgate
from specgate import Problem

HELP = "`cpl explain` lists exactly the deep copies the judge C++ makes (--max mismatched cases allowed, shrink-only)"

COPY = re.compile(r"(?:(?:std::(?:vector|array|map|set|unordered_map|deque)|cpl::Grid)<[^;(){}=]*?>|std::string)\((?:cpl::at\([^()]*\)|[A-Za-z_]\w*(?:\[[^\[\]]+\]|\.[A-Za-z_]\w*)*)\)(?!>)|cpl::to_list\(|cpl::to_str\(|cpl::sorted(?:<[^<>()]*>)?\((?:cpl::at\([^()]*\)|[A-Za-z_]\w*(?:\[[^\[\]]+\]|\.[A-Za-z_]\w*)*)\)")
CONTAINERS = ("std::vector<", "std::map<", "std::set<", "std::unordered_map<", "std::deque<", "std::string")
NAME = re.compile(r"[A-Za-z_]\w*(?:\[[^\[\]]+\]|\.[A-Za-z_]\w*)*")
BUILT = re.compile(r"(?<![\w:.])(\w+)\((cpl::at\([^()]*\)|[A-Za-z_]\w*(?:\[[^\[\]]+\]|\.[A-Za-z_]\w*)*)\)(?!>)")
REF_FN = re.compile(r"^(?:template <[^>]*> )?(?:inline |static )*const [^=(]*& (\w+)\(.*\{$")
BRACED = re.compile(r"(?<![\w:.])(cpl_cl\d+)\{([A-Za-z_]\w*)\}")
CALL = re.compile(r"(\w+)\(.*\)")
PARAM = re.compile(r"(?:std::vector<|std::map<|std::set<|std::unordered_map<|std::deque<|std::string)[^();{}=]*?[&\s](\w+)\s*[,)]")
ARRAY = re.compile(r"std::array<[^;()]*?>&? (\w+)\s*[=;,)]")
CAPTURES = re.compile(r"\[([^\[\]]*)\]\(")
STRUCT = re.compile(r"^struct (\w+)\s*\{")


def container_structs(cpp):
    """Structs that hold a container field, directly or through another such struct;
    a method's return type is not a field."""
    bodies, name = {}, None
    for line in cpp.splitlines():
        line = line.strip()
        if m := STRUCT.match(line):
            name = m.group(1)
            bodies[name] = []
        elif name and line.startswith("}"):
            name = None
        elif name and "(" not in line:
            bodies[name].append(line.removeprefix("mutable "))
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


KEYWORDS = ("nullptr", "true", "false", "this")
GET = re.compile(r"std::get<\d+>\((.+)\)")


def is_copied(init):
    """Whether initialising a by-value local from `init` copies a place:
    a name, a part of one, or a tuple field of one, `std::get<1>(top)`."""
    init = init.strip()
    return is_place(init) or bool((m := GET.fullmatch(init)) and is_place(m.group(1)))


def one_call(text):
    """A single call `f(..)`, whose first `(` closes at the end: not `f(a) + g(b)`."""
    depth = 0
    for i, c in enumerate(text):
        depth += (c == "(") - (c == ")")
        if c == ")" and depth == 0:
            return i == len(text) - 1
    return False


def is_place(text):
    text = text.strip()
    return (bool(NAME.fullmatch(text)) and text not in KEYWORDS) or (text.startswith("cpl::at(") and text.endswith(")") and "::" not in text[8:])


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
OPTIONAL_GET = re.compile(r"\(std::get<\d+>\((?:cpl::have\(([^()]*)\)|([^()]*))\)\)")


def optional_copies(cpp, holders):
    """An optional of a heap value copied from a place: `std::optional<C>(xs)`
    (a `?T` argument), `std::optional<T> t0 = best;` and
    `std::optional<C>(std::get<0>(sp))`. Mirrors optional_copies in
    crates/cpl-codegen/src/cpp_counts.rs and optional_place_copies in
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
            elif heap(ty) and (m := OPTIONAL_GET.match(rest)) and place(m.group(1) or m.group(2)):
                count += 1
    return count


HEADER = re.compile(r"^(?=\S)(?!.*\]\()(?:template <[^>]*> )?(?:inline |static )*(.+?) \w+\(.*\{$")
PART = re.compile(r"[A-Za-z_]\w*(?:\.\w+|\[[^\[\]]+\])*")
LAMBDA_RET = re.compile(r"\)(?: mutable)? -> (.+?) \{$")
BRANCH = re.compile(r"(?:const )?(.+?) \w+ = .+ \? .+ : (.+);$")
FIRST_ARM = re.compile(r"(?:const )?(.+?) \w+ = .+? \? ([A-Za-z_]\w*) : ")


def returned_parts(cpp, holders):
    """A part of a value given out by value: `return self.agg[t];` from a
    fn returning a heap type, `Mat v = c ? f(..) : self.t[p];` and `T v = c ? xs : f(..)`.
    Mirrors returned_part_copies in crates/cpl-cli/tests/explain_accuracy.rs."""
    heap = lambda ty: (ty.startswith(CONTAINERS + ("std::array<",)) or ty in holders) and not ty.endswith(("&", "*"))
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
            scopes.append((depth, (m.group(1) if (m := LAMBDA_RET.search(s)) else None)))
        depth += s.count("{") - s.count("}") + lead
        ret = scopes[-1][1] if scopes else None
        if ret and heap(ret) and not ret.endswith(("&", "*")) and s.startswith("return ") and s.endswith(";") and part(s[7:-1]):
            count += 1
        last = (m := BRANCH.match(s)) and heap(m.group(1)) and (part(m.group(2)) or is_place(m.group(2)))
        first = (m := FIRST_ARM.match(s)) and heap(m.group(1)) and is_place(m.group(2))
        if last or first:
            count += 1
    return count


def declared(decl):
    """`decl` split at the end of what it declares (the first `=`, `(`, `;`,
    `,`, `)` or `{` outside `<>`), and the name it declares there."""
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
    return head, re.split(r"[ &*>]", head)[-1], decl[end:]


def typed(decl):
    """The name `decl` declares and its type, without `const` and `&`."""
    head, name, _ = declared(decl.removeprefix("static ").removeprefix("const "))
    if not re.fullmatch(r"\w+", name) or len(name) == len(head):
        return None
    return name, head[: len(head) - len(name)].strip().rstrip("&*").strip()


def struct_fields(cpp):
    """Each struct's fields and their types."""
    fields, name = {}, None
    for line in cpp.splitlines():
        s = line.strip()
        if m := STRUCT.match(s):
            name = m.group(1)
            fields[name] = {}
        elif name and line.startswith("}"):
            name = None
        elif name and s.endswith(";") and "(" not in s.split(" = ")[0] and (t := typed(s)):
            fields[name][t[0]] = t[1]
    return fields


def params(header):
    """The names and types of a fn header's parameters."""
    if "(" not in header:
        return {}
    inner = header[header.index("(") + 1 :].rsplit(")", 1)[0]
    return dict(t for p in split_top(inner) if p and (t := typed(p)))


def element(ty):
    """The element type an index reads from container type `ty`, a map's value type."""
    for prefix, pick in (("std::vector<", 0), ("std::deque<", 0), ("std::array<", 0), ("cpl::Grid<", 0), ("std::map<", 1), ("std::unordered_map<", 1)):
        if ty and ty.startswith(prefix) and (got := template_arg(ty[len(prefix) :])):
            args = split_top(got[0])
            return args[pick] if pick < len(args) else None
    return None


def part_type(place, types, fields):
    """The type of a place, `w.s.xs` or `self.vals[t0]`, through its root's
    type, each field's and each index's element type. Mirrors place_type in
    crates/cpl-cli/tests/explain_accuracy.rs."""
    ty, depth, start, first = None, 0, 0, True
    for i, c in enumerate(place + "."):
        depth += {"[": 1, "]": -1}.get(c, 0)
        if c != "." or depth:
            continue
        seg, start = place[start:i], i + 1
        name = seg.split("[", 1)[0]
        if not re.fullmatch(r"[A-Za-z_]\w*", name):
            return None
        ty = types.get(name) if first else fields.get(ty, {}).get(name)
        first, d = False, 0
        for ch in seg:
            if ch == "[":
                if d == 0:
                    ty = element(ty)
                d += 1
            elif ch == "]":
                d -= 1
    return ty


def stored(ty, holders):
    """Whether a store of a `ty` value copies what it owns: a container, a
    fixed array, a struct holding either or a tuple holding a container."""
    return bool(ty) and (ty.startswith(CONTAINERS + ("std::array<", "cpl::Row<")) or ty in holders or (ty.startswith("std::tuple<") and any(c in ty for c in CONTAINERS)))


def opens_fn(line):
    """A top-level line opening a fn or lambda body, where the names
    in scope start over."""
    return line[:1] not in ("", " ", "}", "#") and line.rstrip().endswith("{") and "(" in line and not line.startswith(("struct ", "namespace "))


def deep_copies(cpp):
    """Mirrors container_copies in crates/cpl-codegen/src/cpp_counts.rs, plus
    copies spelled as a constructor call or `cpl::to_list`, tuples holding a
    container copied from places, a container assigned from a place, and
    optionals of heap values copied from places, parts returned by value;
    a closure struct braced from a held name (`cpl_cl5{ew}` owns `ew`); a
    lambda's `auto&&` params keep the types they were passed;
    the runtime's own lines are skipped. The names a fn holds as containers
    are its own: a scalar of the same name in another fn is not one."""
    count = len(COPY.findall(program(cpp)))
    holders = container_structs(cpp)
    count += sum(m.group(1) in holders for line in program(cpp).splitlines() if not line.lstrip().startswith("//") for m in BUILT.finditer(line))
    refs = {m.group(1) for line in program(cpp).splitlines() if (m := REF_FN.match(line))}
    copied = lambda init: is_copied(init) or bool((m := CALL.fullmatch(init.strip())) and m.group(1) in refs and one_call(init.strip()))
    fields = struct_fields(cpp)
    owners = holders | {n for n, fs in fields.items() if any(t.startswith(("std::array<", "cpl::Row<")) for t in fs.values())}
    heap = lambda ty: bool(ty) and (ty.startswith(CONTAINERS) or ty in holders)
    count += optional_copies(program(cpp), holders) + returned_parts(program(cpp), holders)
    arrays = set(ARRAY.findall(program(cpp)))
    held, types, scalars = set(), {}, set()
    for raw in program(cpp).splitlines():
        if opens_fn(raw):
            held, types, scalars = set(PARAM.findall(raw)), params(raw), set()
        line = raw.strip()
        decl = line.removeprefix("const ")
        if "](" in line and line.endswith("{"):
            types.update((n, t) for n, t in params(line[line.index("](") :]).items() if t != "auto")
        if t := typed(decl):
            init = decl.removesuffix(";").partition(" = ")[2]
            aliased = part_type(init, types, fields) if t[1] == "auto" and init else None
            types[t[0]] = aliased or t[1]
        ty, _, rest = decl.partition(" ")
        if ty in holders:
            name, eq, init = rest.partition("=")
            name = name.strip()
            if re.fullmatch(r"\w+", name):
                held.add(name)
                if eq and init.strip().endswith(";") and copied(init.strip()[:-1]):
                    count += 1
        if (scalar := scalar_decl(decl)) is not None:
            held.discard(scalar)
            scalars.add(scalar)
        if decl.startswith(CONTAINERS):
            head, name, rest = declared(decl)
            if name and len(name) < len(head):
                held.add(name)
                by_value = not head[: len(head) - len(name)].rstrip().endswith(("&", "*"))
                if by_value and rest.startswith("=") and rest.endswith(";") and copied(rest[1:-1]):
                    count += 1
        if line.startswith("std::tuple<") and any(c in line for c in CONTAINERS) and ".first, " in line and ".second)" in line:
            count += 1
        elif decl.startswith("std::tuple<"):
            count += tuple_copies(decl)
        lhs, eq, rhs = line.partition(" = ")
        into = part_type(lhs, types, fields)
        place = eq and (is_place(rhs[:-1]) or part_type(rhs[:-1], types, fields) is not None)
        from_ = part_type(rhs[:-1], types, fields) if place else None
        if eq and rhs.endswith(";") and NAME.fullmatch(lhs) and (stored(into, owners) and (place or copied(rhs[:-1])) or stored(from_, owners)):
            count += 1
        count += sum(m.group(1) in holders and m.group(2) in held for m in BRACED.finditer(line))
        for caps in CAPTURES.findall(line):
            for cap in re.split(r"\s*,\s*", caps):
                name, eq, src = cap.partition("=")
                if eq and not name.strip().startswith("&") and (src.strip() in held or (src.strip() in arrays and src.strip() not in scalars)):
                    count += 1
    return count


def one(cpl, case, tmp, exact, base=specgate.ROOT):
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
    return case.relative_to(base), claimed, made


def default_cache_dir() -> Path | None:
    """`cpl/explain-copies` beside the C++ cache (`~/.cache/cpl/cxx`)."""
    base = os.environ.get("XDG_CACHE_HOME") or (os.environ.get("HOME") and os.path.join(os.environ["HOME"], ".cache"))
    return Path(base) / "cpl" / "explain-copies" if base else None


class Cache:
    """Each case's row, keyed by its path and bytes under one version of the
    inputs every row depends on: the cpl binary, std/, `--exact` and the CPL_
    environment. A changed input starts a new version; the newest KEEP stay."""

    KEEP = 8

    def __init__(self, root: Path, cpl: str, exact: bool):
        h = hashlib.sha256()
        cpl_path = Path(cpl)
        h.update(cpl_path.read_bytes() if cpl_path.is_file() else cpl.encode())
        std = specgate.ROOT / "std"
        for f in sorted(std.rglob("*")) if std.is_dir() else []:
            if f.is_file():
                h.update(f.relative_to(std).as_posix().encode() + b"\0" + f.read_bytes() + b"\0")
        h.update(repr((exact, sorted((k, v) for k, v in os.environ.items() if k.startswith("CPL_")))).encode())
        self.dir = root / h.hexdigest()[:32]
        self.misses = 0
        self.dir.mkdir(parents=True, exist_ok=True)
        os.utime(self.dir)
        versions = sorted((d for d in root.iterdir() if d.is_dir()), key=lambda d: d.stat().st_mtime, reverse=True)
        for old in versions[self.KEEP:]:
            shutil.rmtree(old, ignore_errors=True)

    def row(self, case: Path, compute):
        rel = case.relative_to(specgate.CONFORMANCE.parent.parent).as_posix()
        entry = self.dir / hashlib.sha256(rel.encode() + b"\0" + case.read_bytes()).hexdigest()
        try:
            stored = json.loads(entry.read_text())
            return None if stored is None else (Path(stored[0]), stored[1], stored[2])
        except (OSError, ValueError):
            pass
        self.misses += 1
        got = compute()
        tmp = entry.with_suffix(f".{os.getpid()}.{id(got)}.tmp")
        tmp.write_text(json.dumps(None if got is None else [Path(got[0]).as_posix(), got[1], got[2]]))
        os.replace(tmp, entry)
        return got


def args(ap) -> None:
    ap.add_argument("--max", type=int, default=0, help="cases whose copy list differs from the C++ (shrink-only)")
    ap.add_argument("--exact", action="store_true", help="also flag a case whose explain lists fewer copies than its C++ makes")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--cpl")
    ap.add_argument("--cache-dir", type=Path, default=default_cache_dir(),
                    help="keep each case's result here between runs (default ~/.cache/cpl/explain-copies)")
    ap.add_argument("--no-cache", action="store_true", help="recompute every case")


def run(a) -> list[Problem]:
    cpl = a.cpl or str(specgate.ROOT / "target" / "debug" / "cpl")
    cases = sorted(specgate.CONFORMANCE.rglob("*.cp"))
    with tempfile.TemporaryDirectory() as tmp, cf.ThreadPoolExecutor(a.jobs) as pool:
        cache = None if a.no_cache or a.cache_dir is None else Cache(a.cache_dir, cpl, a.exact)
        rows = [r for r in pool.map(lambda c: cache.row(c, lambda: one(cpl, c, tmp, a.exact)) if cache
                                    else one(cpl, c, tmp, a.exact), cases) if r]
    over = [r for r in rows if r[1] > r[2]]
    under = [r for r in rows if r[1] < r[2]] if a.exact else []
    if len(over) + len(under) <= a.max:
        return []
    return [
        Problem(str(path), f"explain {kind} copies: it lists {claimed}, the C++ makes {made}")
        for kind, wrong in (("over-lists", over), ("under-lists", under))
        for path, claimed, made in wrong
    ]

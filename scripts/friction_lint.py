#!/usr/bin/env python3
"""Counts known friction workarounds in CP Lang 0.2.2 sources.

Each rule is a spelling a port falls back to; every count should only go
down. A rule has one severity:

    wa_trap     code that compiles and silently does the wrong thing
                (a negative slice bound `a[..-1]` is an empty list)
    real_gap    the language or stdlib is missing something; the hit is the
                workaround until it lands (`while head < q.len:` for a worklist)
    stale_port  a better spelling exists today; the port was never updated
                (`Z(0)` where `0` coerces, `use std.mod.{Mod}`, `a[i - 1]`)

Rules come from the original seven categories and the detector proposals in
research/friction-hunter-wave-{1,2,3}.md (`source` in the JSON says which).

usage: friction_lint.py [--json] [--top N] [--list RULE] [--baseline FILE] [PATH ...]

PATHs default to proto/ports and examples (every *.cp below them, minus
before/ snapshots that record code as it had to be written before a fix).
--baseline takes an earlier --json report and exits 1 if any rule grew.
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


def find_mod_literals(code, lines, raw):
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


def find_self_assign(code, lines, raw):
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


def find_temp_swaps(code, lines, raw):
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


def find_comprehension_copies(code, lines, raw):
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


def find_domain_bounds(code, lines, raw):
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


def find_positional_actions(code, lines, raw):
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


def line_hits(pattern, label, flags=0):
    """A finder that reports every match of `pattern` on a code line."""
    pat = re.compile(pattern, flags)

    def find(code, lines, raw):
        return [(i, label) for i, line in enumerate(lines, 1) for _ in pat.finditer(line)]
    return find


def indent(line):
    return len(line) - len(line.lstrip())


def block(lines, i):
    """Indices of the lines in the indented block opened by line i (0-based),
    plus line i itself when the block is written after its colon."""
    out, base = [i], indent(lines[i])
    for j in range(i + 1, len(lines)):
        if lines[j].strip() and indent(lines[j]) <= base:
            break
        out.append(j)
    return out


def find_cursor_loops(code, lines, raw):
    """`while i < xs.len:` as a hand-rolled worklist (W2). A file that also
    keeps in-degrees is a hand-written Kahn peel, counted by `manual_kahn`."""
    if re.search(r"\bindeg\w*\b", code):
        return []
    pat = re.compile(rf"\bwhile\s+({IDENT})\s*<\s*({PLACE})\.len\s*:")
    return [(i, "while i < xs.len") for i, line in enumerate(lines, 1) if pat.search(line)]


def find_manual_kahn(code, lines, raw):
    if not re.search(r"\bindeg\w*\b", code):
        return []
    pat = re.compile(rf"\bwhile\s+{IDENT}\s*<\s*{PLACE}\.len\s*:")
    return [(i, "Kahn loop beside indeg[]") for i, line in enumerate(lines, 1) if pat.search(line)]


def find_list_self_append(code, lines, raw):
    pat = re.compile(rf"(?<![\w.\]])({PLACE})\s*=\s*\1\s*\+\s*\[")
    return [(i, "xs = xs + [..]") for i, line in enumerate(lines, 1) if pat.search(line)]


def find_one_based_shift(code, lines, raw):
    """`a[i - 1]` inside `for i in 1..=n`: read the array as `[1..=n]T`."""
    hits = []
    loop = re.compile(rf"\bfor\s+({IDENT})\s+in\s+1\s*\.\.=")
    for i, line in enumerate(lines):
        m = loop.search(line)
        if not m:
            continue
        v = m.group(1)
        body = [(j, lines[j][m.end():] if j == i else lines[j]) for j in block(lines, i)]
        # An array also indexed by plain `i`, or grown by push, is a recurrence
        # (`f[i] = f[i - 1] ..`), not a 1-based read.
        direct = {a for _, t in body for a in re.findall(rf"\b({IDENT})\[\s*{v}\s*\]|\b({IDENT})\.push\(", t)}
        direct = {x for pair in direct for x in pair if x}
        for j, text in body:
            for a in re.findall(rf"\b({IDENT})\[\s*{v}\s*-\s*1\s*\]", text):
                if a not in direct:
                    hits.append((j + 1, "a[i - 1] over 1..=n"))
    return sorted(set(hits))


def find_dead_tail(code, lines, raw):
    """A function whose last statement is `true`/`false` right after a
    `while true:` loop it can only leave by returning."""
    hits = []
    for i, line in enumerate(lines):
        if not re.match(r"\s*(pub\s+)?fn\b.*:\s*$", line):
            continue
        body = [j for j in block(lines, i)[1:] if lines[j].strip()]
        if not body:
            continue
        top = indent(lines[body[0]])
        stmts = [j for j in body if indent(lines[j]) == top]
        if len(stmts) >= 2 and re.fullmatch(r"\s*(true|false)\s*", lines[stmts[-1]]) \
                and re.fullmatch(r"\s*while\s+true\s*:\s*", lines[stmts[-2]]):
            hits.append((stmts[-1] + 1, "dead tail after while true"))
    return hits


def find_push_after_literal(code, lines, raw):
    """`var xs = [..]` then a loop pushing onto it: join with `+`."""
    hits = []
    lit = re.compile(rf"^\s*var\s+({IDENT})\s*(?::[^=]*)?=\s*\[[^\]]*\]\s*$")
    for i in range(len(lines) - 1):
        m = lit.match(lines[i])
        if m and re.match(rf"\s*for\b.*:\s*{m.group(1)}\.push\(", lines[i + 1]):
            hits.append((i + 2, "var xs = [..]; for ..: xs.push"))
    return hits


def find_lone_sum(code, lines, raw):
    pat = re.compile(rf"^\s*for\s+({IDENT})\s+in\s+{PLACE}\s*:\s*{IDENT}\s*\+=\s*\1\s*$")
    return [(i, "for x in xs: s += x") for i, line in enumerate(lines, 1) if pat.match(line)]


def find_param_copy(code, lines, raw):
    """`var x = x` at the top of a function: a by-value parameter it wants to edit."""
    hits = []
    for i, line in enumerate(lines):
        m = re.match(r"\s*(?:pub\s+)?fn\s+\w+\s*(?:<[^>]*>)?\s*\((.*)\)", line)
        if not m:
            continue
        params = set(re.findall(rf"(?:^|,)\s*({IDENT})\s*:", m.group(1)))
        for j in block(lines, i)[1:]:
            c = re.match(rf"\s*var\s+({IDENT})\s*=\s*\1\s*$", lines[j])
            if c and c.group(1) in params:
                hits.append((j + 1, "var x = x"))
    return hits


def find_digit_loop(code, lines, raw):
    hits = []
    for i, line in enumerate(lines):
        m = re.search(rf"\bwhile\s+({IDENT})\s*>\s*0\s*:", line)
        if not m:
            continue
        div = re.compile(rf"\b{m.group(1)}\s*/=\s*10\s*$")
        hits += [(j + 1, "x /= 10 digit loop") for j in block(lines, i) if div.search(lines[j])]
    return hits


def find_where_guard(code, lines, raw):
    pat = re.compile(r"\bread\s*\(\s*\)\s+where\b|^\s*(?:pub\s+)?fn\b.*\)\s*(?:->[^:]*?)?\swhere\b.*:\s*$")
    return [(i, "removed where guard") for i, line in enumerate(lines, 1) if pat.search(line)]


def find_prototype_stamp(code, lines, raw):
    return [(i, "// cp: stamp") for i, line in enumerate(raw, 1) if re.match(r"// cp: \d", line)]


def prelude_names():
    path = os.path.join(ROOT, "stdlib", "prelude.cp")
    names = {}
    if os.path.exists(path):
        for mod, items in re.findall(r"^use std\.(\w+)\.\{([^}]*)\}", open(path).read(), re.M):
            names.setdefault(mod, set()).update(x.strip() for x in items.split(","))
    return names


PRELUDE = prelude_names()


def find_prelude_imports(code, lines, raw):
    hits = []
    for i, line in enumerate(lines, 1):
        for mod, items in re.findall(r"\buse\s+std\.(\w+)\.\{([^}]*)\}", line):
            dup = [x.strip() for x in items.split(",") if x.strip() in PRELUDE.get(mod, ())]
            if dup:
                hits.append((i, f"std.{mod}.{{{', '.join(dup)}}}"))
    return hits


def find_fact_mod(code, lines, raw):
    pat = re.compile(r"\bfn\s+pow_mod\s*\(|\b(\w+)\[(\w+)\]\s*=\s*\1\[\2\s*-\s*1\]\s*\*\s*\2\s*%")
    return [(i, "hand-rolled modular arithmetic") for i, line in enumerate(lines, 1) if pat.search(line)]


V = rf"({IDENT})"
# (key, severity, source, title, finder)
RULES = [
    # wa_trap
    ("negative_slice", "wa_trap", "wave2#16", "negative slice bound is an empty list",
     line_hits(r"\[[^\]\n]*\.\.\s*-\s*\d", "a[..-k]")),
    # real_gap
    ("manual_cursor_loop", "real_gap", "orig, wave3#28", "worklist cursor loop (W2)", find_cursor_loops),
    ("second_from_end", "real_gap", "wave2#17", "xs[xs.len - 2] (N1: last(1))",
     line_hits(rf"\b{V}\[\s*\1\.len\s*-\s*2\s*\]", "xs[xs.len - 2]")),
    ("domain_step_unwrap", "real_gap", "wave2#18", "(i + 1)! domain stepping (N2)",
     line_hits(r"\((?:l|r|m|lo|hi|i|j)\s*[+-]\s*1\)!", "(i ± 1)!")),
    ("sorted_index_list", "real_gap", "wave2#19", "sorted([i for i in ..], by:) (N4)",
     line_hits(rf"\bsorted\(\s*\[\s*{V}\s+for\s+\1\s+in\b[^\]]*\]\s*,\s*by\b", "sorted([i for i ..], by:)")),
    ("bool_list_loop", "real_gap", "wave2#20", "in [false, true] (N4)",
     line_hits(r"\bin\s+\[\s*false\s*,\s*true\s*\]", "in [false, true]")),
    ("infinite_capacity", "real_gap", "wave2#21", "INF capacity edge (N3)",
     line_hits(r"\.add\(.*\b(?:INF|inf\w*|cap_inf)\s*\)\s*$", ".add(.., INF)")),
    ("param_copy", "real_gap", "wave1#14", "var x = x for a mutable parameter", find_param_copy),
    ("bit_test", "real_gap", "wave1#15", "x >> i & 1 bit test",
     line_hits(r">>\s*\w+\s*&\s*1\b", "x >> i & 1")),
    ("digit_loop", "real_gap", "wave3#29", "x /= 10 digit loop (W3)", find_digit_loop),
    ("list_self_append", "real_gap", "wave3#30", "xs = xs + [..] (W4)", find_list_self_append),
    ("segtree_get_set", "real_gap", "wave3#31", "t.set(i, t.get(i) op ..) (W5)",
     line_hits(rf"\.set\(\s*{V}\s*,\s*{IDENT}\.get\(\s*\1\s*\)\s*[-+*]", "set(i, get(i) op ..)")),
    ("child_end", "real_gap", "wave3#32", "if parent(u) == v: u else: v (W6)",
     line_hits(rf"\.parent\(\s*{V}\s*\)\s*==\s*{V}\s*:\s*\1\s+else\s*:\s*\2\b", "parent(u) == v: u else: v")),
    ("arrow_block", "real_gap", "wave3#37", "array builder or lambda with a block body (B1)", line_hits(r"(?:[=(,]|\])\s*(?:\([\w\s,:]*\)|\w+)\s*=>\s*$", "builder/lambda => at end of line")),
    # stale_port
    ("mod_literal_wrapping", "stale_port", "orig", "Mod literal wrapping", find_mod_literals),
    ("missing_compound_assign", "stale_port", "orig", "x = x op .. instead of x op= ..", find_self_assign),
    ("manual_temp_swap", "stale_port", "orig", "3-line temp swap", find_temp_swaps),
    ("comprehension_copy", "stale_port", "orig, wave3#33", "comprehension as clone/slice/pad",
     find_comprehension_copies),
    ("domain_bound_boilerplate", "stale_port", "orig", "domain .first/.last boilerplate", find_domain_bounds),
    ("positional_action", "stale_port", "orig, wave2#26", "positional Action<...>(...) fields",
     find_positional_actions),
    ("len_minus_one", "stale_port", "wave1#1", "xs[xs.len - 1] instead of xs.last!",
     line_hits(rf"\b{V}\[\s*\1\.len\s*-\s*1\s*\]", "xs[xs.len - 1]")),
    ("nonempty_then_last", "stale_port", "wave1#2", "xs.len > 0 and xs.last!",
     line_hits(rf"\b{V}\.len\s*>\s*0\s+and\s+\1\.last!", "xs.len > 0 and xs.last!")),
    ("min_max_assign", "stale_port", "wave1#3", "x = min(x, ..) instead of x min= ..",
     line_hits(rf"^\s*({PLACE})\s*=\s*(?:min|max)\(\s*\1\s*,", "x = min/max(x, ..)")),
    ("prototype_stamp", "stale_port", "wave1#4", "// cp: prototype stamp", find_prototype_stamp),
    ("prelude_import", "stale_port", "wave1#5, wave3#34", "import of a prelude name", find_prelude_imports),
    ("sentinel_minus_one", "stale_port", "wave1#6", "List.filled(.., -1) sentinel instead of ?Int",
     line_hits(r"\bList\.filled\([^,\n]+,\s*-1\s*\)", "List.filled(n, -1)")),
    ("none_then_assign", "stale_port", "wave1#7", "if x == none: x = .. instead of x ??= ..",
     line_hits(rf"\bif\s+({PLACE})\s*==\s*none\s*:\s*\1\s*=[^=]", "if x == none: x =")),
    ("manual_ceil_div", "stale_port", "wave1#8", "(a + b - 1) / b instead of ceil_div",
     line_hits(rf"\(\s*{IDENT}\s*\+\s*{V}\s*-\s*1\s*\)\s*/\s*\1\b", "(a + b - 1) / b")),
    ("bool_to_int", "stale_port", "wave1#9", "if c: 1 else: 0 instead of Int(c)",
     line_hits(r"\bif\s+[^:\n]+:\s*1\s+else\s*:\s*0\b", "if c: 1 else: 0")),
    ("dead_tail", "stale_port", "wave1#10", "unreachable result after while true", find_dead_tail),
    ("push_after_literal", "stale_port", "wave1#11", "var xs = [..] filled by a push loop", find_push_after_literal),
    ("lone_sum", "stale_port", "wave1#12", "for x in xs: s += x instead of sum", find_lone_sum),
    ("one_based_shift", "stale_port", "wave1#13, wave3#35", "a[i - 1] over 1..=n (review, do not auto-fix)",
     find_one_based_shift),
    ("hand_modular", "stale_port", "wave2#22", "pow_mod / fact[i] % m instead of Mod, Comb", find_fact_mod),
    ("manual_kahn", "stale_port", "wave2#23", "hand-written Kahn peel instead of peel(g)", find_manual_kahn),
    ("none_check_unwrap", "stale_port", "wave2#24", "x != none and .. x! instead of if let",
     line_hits(rf"\b{V}\s*!=\s*none\s+and\b.*\b\1!", "x != none and x!")),
    ("none_or_compare", "stale_port", "wave2#25", "if x == none or y < x!: x = .. instead of min=",
     line_hits(rf"\bif\s+({PLACE})\s*==\s*none\s+or\s+.*[<>]=?\s*\1!\s*:\s*\1\s*=[^=]",
               "if x == none or v < x!: x =")),
    ("where_guard", "stale_port", "wave3#36", "where guard removed in 0.2.2 (use assert)", find_where_guard),
]

# Counted for trend only, outside the three severities and total_hits.
METRICS = [
    ("label_crossing", "wave2#27", ".to_idx / .from_idx / f(x.label ± k)",
     line_hits(r"\.(?:to_idx|from_idx)\b|\w+\(\w+\.label\s*[+-]\s", "label crossing")),
]

SEVERITIES = ("wa_trap", "real_gap", "stale_port")


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
    table = [(k, sev, src, title, f) for k, sev, src, title, f in RULES] + \
            [(k, "metric", src, title, f) for k, src, title, f in METRICS]
    report = {k: {"severity": sev, "source": src, "title": title, "count": 0,
                  "patterns": Counter(), "files": Counter(), "hits": []}
              for k, sev, src, title, _ in table}
    files = 0
    for path in sources(paths):
        files += 1
        rel = os.path.relpath(path, ROOT)
        text = open(path, errors="replace").read()
        code = strip_code(text)
        lines, raw = code.split("\n"), text.split("\n")
        for key, _, _, _, find in table:
            for line, pattern in find(code, lines, raw):
                r = report[key]
                r["count"] += 1
                r["patterns"][pattern] += 1
                r["files"][rel] += 1
                r["hits"].append(f"{rel}:{line}: {pattern}")
    return files, report


def as_json(files, report, top):
    counted = {k: r for k, r in report.items() if r["severity"] in SEVERITIES}
    total = sum(r["count"] for r in counted.values())
    by_file = Counter()
    for r in counted.values():
        by_file.update(r["files"])

    def rule(r):
        return {
            "severity": r["severity"],
            "source": r["source"],
            "title": r["title"],
            "count": r["count"],
            "files": len(r["files"]),
            "patterns": dict(r["patterns"].most_common()),
            "top_files": [[f, n] for f, n in r["files"].most_common(top)],
        }
    return {
        "version": "0.2.2",
        "files_scanned": files,
        "total_hits": total,
        "hits_per_file": round(total / files, 3) if files else 0.0,
        "files_with_hits": len(by_file),
        "severity_totals": {s: sum(r["count"] for r in counted.values() if r["severity"] == s)
                            for s in SEVERITIES},
        "rule_counts": {k: r["count"] for k, r in counted.items()},
        "rules": {k: rule(r) for k, r in counted.items()},
        "metrics": {k: rule(r) for k, r in report.items() if r["severity"] == "metric"},
        "top_files": [[f, n] for f, n in by_file.most_common(top)],
    }


def baseline_counts(base):
    """Rule counts from a --json report, old (`categories`) or new (`rules`)."""
    rules = base.get("rules") or base.get("categories") or {}
    return {k: v["count"] for k, v in rules.items()}


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("paths", nargs="*", default=DEFAULT_PATHS)
    ap.add_argument("--json", action="store_true", help="machine-readable report")
    ap.add_argument("--top", type=int, default=5, help="offending files shown per rule")
    ap.add_argument("--list", metavar="RULE", help="print every hit of one rule")
    ap.add_argument("--baseline", metavar="FILE", help="exit 1 if a rule grew past this --json report")
    args = ap.parse_args()

    files, report = scan(args.paths)
    if args.list:
        if args.list not in report:
            ap.error(f"unknown rule {args.list!r}; one of: {', '.join(report)}")
        print("\n".join(report[args.list]["hits"]))
        return 0

    data = as_json(files, report, args.top)
    if args.json:
        print(json.dumps(data, indent=2))
    else:
        sev = ", ".join(f"{n} {s}" for s, n in data["severity_totals"].items())
        print(f"{data['total_hits']} friction workarounds in {files} files "
              f"({data['hits_per_file']} per file): {sev}\n")
        for s in SEVERITIES + ("metric",):
            for key, r in report.items():
                if r["severity"] != s or (r["count"] == 0 and s != "wa_trap"):
                    continue
                print(f"{r['count']:5}  [{s}] {key}  ({r['title']}, {len(r['files'])} files)")
                for f, n in r["files"].most_common(args.top):
                    print(f"{'':7}{n:5}  {f}")
        zero = [k for k, r in report.items() if r["count"] == 0 and r["severity"] != "wa_trap"]
        if zero:
            print(f"\n    0  {', '.join(zero)}")

    if args.baseline:
        with open(args.baseline) as fh:
            base = baseline_counts(json.load(fh))
        grew = [(k, base[k], r["count"]) for k, r in report.items() if k in base and r["count"] > base[k]]
        for k, old, new in grew:
            print(f"friction_lint: {k} grew from {old} to {new}", file=sys.stderr)
        return 1 if grew else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())

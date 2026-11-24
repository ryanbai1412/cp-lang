"""Check std/*.cp, or reduce it to reference docs (every fn body `todo`).

`--audit` (doc labels only) enforces handbook/guidelines/std-docs-checklist.md#r1 (required doc labels).

std is the standard library: signatures, docs, costs and bodies.
With `--stub`, only fn bodies change: headers, docs, traits, structs and bodiless
trait methods are kept as written. `lends` fns keep their bodies, since the
checker needs the `lend place` to type the place they return.

`spec-gate.py std-docs [FILE...]` (default std/*.cp) changes nothing and fails if an
`@get` member is documented as costing more than O(1) to make: anything
that does work up front takes parentheses (decision 140), or if a fn that
mutates nothing declares no return type: the checker cannot type its result,
so every method called on it would go unchecked, or if a prelude name
appears in no core or lib chapter (core 11.3).
"""
from __future__ import annotations

import glob
import os
import re

import specgate
from specgate import Problem

HELP = "std docs: required labels, O(1) @get members, typed results, prelude names in a chapter; --stub reduces bodies to todo"

DOC_LABELS = ("cost", "traps", "example", "empty", "ties", "order")
SEQ_INPUT = re.compile(r"\b(Seq|Indexed|IndexedMut|Keyed|List<|Range<|Domain)\b|\[[^\]]*\]\w|yields")
CHOOSES = re.compile(r"^(?!is_)((arg)?(min|max)|best|first|last|find|sort|kth|nearest|lower|upper|mode|top|bsearch|search)|_(min|max)$")
PUB_FN = re.compile(r"^\s*(?:pub\s+(?:@\w+\s+)*|@get\s+pub\s+)fn\s+(\w+)")


def doc_gaps(lines):
    """(line, name, missing labels) for every pub fn whose doc lacks a required label (README)."""
    out = []
    for i, line in enumerate(lines):
        m = PUB_FN.match(line)
        if not m:
            continue
        doc, j = [], i - 1
        while j >= 0 and lines[j].strip().startswith("@"):
            j -= 1
        while j >= 0 and lines[j].strip().startswith("///"):
            doc.insert(0, lines[j].strip()[3:].strip())
            j -= 1
        labels = {d.split(":", 1)[0] for d in doc if re.match(r"^[a-z]+:", d)}
        if "```cp" in doc:
            labels.add("example")
        header = " ".join(lines[i:i + 6])
        end = re.search(r"\)\s*(?:->[^=\n]*?)?(?:\s=\s|:\s*(?:$|\s{2,}))", header)
        header = header[:end.end()] if end else header
        need = ["cost", "traps", "example"]
        if SEQ_INPUT.search(header.split("->")[0]):
            need.append("empty")
        if CHOOSES.search(m.group(1)):
            need.append("ties")
        if "->" in header and SEQ_INPUT.search(header.split("->", 1)[1]):
            need.append("order")
        missing = [k for k in need if k not in labels]
        if missing:
            out.append((i + 1, m.group(1), missing))
    return out

FN = re.compile(r"^(\s*)(?:(?:@\S+|pub|extern)\s+)*fn\b")


def code(line):
    """The line without its comment and with string contents blanked."""
    out, q, i = [], None, 0
    while i < len(line):
        c = line[i]
        if q:
            out.append(" " if c != q else c)
            if c == "\\":
                out.append(" "); i += 1
            elif c == q:
                q = None
        elif c in "\"'":
            q = c; out.append(c)
        elif line.startswith("//", i):
            break
        else:
            out.append(c)
        i += 1
    return "".join(out)


def header_end(lines, i):
    """(last header line, column of the body's `:` or `=`, kind) or None."""
    depth, seen_paren, in_where = 0, False, False
    for j in range(i, min(i + 40, len(lines))):
        s = code(lines[j])
        k = 0
        while k < len(s):
            c = s[k]
            if depth == 0 and re.match(r"\bwhere\b", s[k:]) and not s[k - 1:k].isalnum():
                in_where = True
            if c in "([{":
                depth += 1
                seen_paren |= c == "("
            elif c in ")]}":
                depth -= 1
            elif not seen_paren and c == "<":
                depth += 1
            elif not seen_paren and c == ">" and s[k - 1:k] != "-":
                depth -= 1
            elif depth == 0 and c == "=" and s[k - 1:k] not in ("=", "<", ">", "!", ".") \
                    and s[k + 1:k + 2] not in ("=", ">"):
                return j, k, "="
            elif depth == 0 and c == ":" and s[k + 1:k + 2] != ":" and s[k - 1:k] != ":" \
                    and not in_where:
                return j, k, ":"
            k += 1
        if depth == 0 and j > i and not s.strip():
            return None
        if depth == 0 and not s.rstrip().endswith((",", "(", "->")):
            nxt = lines[j + 1] if j + 1 < len(lines) else ""
            if not code(nxt).lstrip().startswith(("->", ")")):
                return None
    return None


def brackets(s):
    return sum(s.count(c) for c in "([{") - sum(s.count(c) for c in ")]}")


def indent(line):
    return len(line) - len(line.lstrip())


def stub(lines):
    out, i, changed = [], 0, 0
    while i < len(lines):
        m = FN.match(lines[i])
        end = header_end(lines, i) if m else None
        if not end:
            out.append(lines[i]); i += 1; continue
        j, col, kind = end
        base = indent(lines[i])
        head = lines[i:j] + [lines[j][:col].rstrip()]
        rest = code(lines[j])[col + 1:].strip()
        k, depth = j + 1, brackets(rest)
        while k < len(lines) and (not lines[k].strip() or indent(lines[k]) > base or depth > 0):
            depth += brackets(code(lines[k]))
            k += 1
        while k > j + 1 and not lines[k - 1].strip():
            k -= 1
        lends = " lends " in " ".join(code(l) for l in head)
        if lends or (rest == "todo" and k == j + 1):
            out.extend(lines[i:k])
        else:
            out.extend(head[:-1] + [head[-1] + " = todo"])
            changed += 1
        i = k
    return out, changed


GET = re.compile(r"@get\s+fn\s+(\w+)")
COST = re.compile(r"O\(([^)]*)\)")


def costly_getters(lines):
    """`@get` fns whose doc comment gives a first cost other than O(1)."""
    out = []
    for i, line in enumerate(lines):
        m = GET.search(line)
        if not m:
            continue
        j = i
        while j > 0 and lines[j - 1].strip().startswith("///"):
            j -= 1
        doc = " ".join(l.strip()[3:] for l in lines[j:i])
        if "O(1) to make" in doc:
            continue
        cost = COST.search(doc)
        if cost and cost.group(1).strip() != "1":
            out.append((i + 1, m.group(1), cost.group(0)))
    return out


UNIT_OK = {"prune", "cdq"}  # they act through `self` or a callback and return nothing


def untyped(lines):
    """Pub fns with no `->` and no `mut` parameter: they return something untyped."""
    out = []
    for i, line in enumerate(lines):
        m = re.match(r"\s*(?:@\S+\s+)*pub\s+(?:@\S+\s+)*fn\s+(\w+)", line)
        if not m or m.group(1) in UNIT_OK or "@test" in line + lines[i - 1]:
            continue
        text = "\n".join(lines[i:i + 40])
        k = len(line[:m.end()])
        if text[k] == "<":
            angle = 0
            for k in range(k, len(text)):
                angle += (text[k] == "<") - (text[k] == ">" and text[k - 1] != "-")
                if angle == 0:
                    break
            k += 1
        while text[k].isspace():
            k += 1
        depth = 0
        for k in range(k, len(text)):
            depth += text[k] in "([{"
            depth -= text[k] in ")]}"
            if depth == 0:
                break
        params, rest = text[:k + 1], text[k + 1:].lstrip()
        if not rest.startswith(("->", "lends")) and "mut " not in params:
            out.append((i + 1, m.group(1)))
    return out


def undocumented_prelude(root):
    """Prelude names no core or lib chapter shows in code: the prelude is the chapters'
    vocabulary (core 11.3), so each name needs a chapter row before it is imported."""
    spec = os.path.join(root, "spec")
    text = open(os.path.join(root, "std", "prelude.cp")).read()
    names = [n.strip() for group in re.findall(r"use [\w.]+\.\{([^}]*)\}", text)
             for n in group.split(",") if n.strip()]
    docs = "".join(open(f).read() for d in ("core", "lib")
                   for f in glob.glob(os.path.join(spec, d, "*.md")))
    return [n for n in names
            if not re.search(r"`[^`\n]*\b" + re.escape(n) + r"\b[^`\n]*`", docs)
            and not re.search(r"^fn\s+" + re.escape(n) + r"\b", docs, re.M)]


def label_gaps(files):
    """Every pub fn carries its required doc labels (std/README.md): no baseline, zero allowed."""
    return [Problem(f"{rel(f)}:{line}", f"`{name}` lacks {', '.join(missing)} (std/README.md)")
            for f in files for line, name, missing in doc_gaps(open(f).read().split("\n"))]


def rel(f) -> str:
    return os.path.relpath(f, specgate.ROOT)


def args(ap) -> None:
    ap.add_argument("--audit", action="store_true", help="only the required doc labels")
    ap.add_argument("--stub", action="store_true", help="rewrite every fn body to `todo`")
    ap.add_argument("files", nargs="*")


def run(a) -> list[Problem]:
    files = a.files or sorted(glob.glob(str(specgate.ROOT / "std" / "*.cp")))
    if a.audit:
        return label_gaps(files)
    if a.stub:
        for f in files:
            new, n = stub(open(f).read().split("\n"))
            if n:
                open(f, "w").write("\n".join(new))
                print(f"{rel(f)}: {n} body(ies) stubbed")
        return []
    out = []
    for f in files:
        lines = open(f).read().split("\n")
        out += [Problem(f"{rel(f)}:{line}", f"`@get {name}` costs {cost}; work up front takes parentheses")
                for line, name, cost in costly_getters(lines)]
        out += [Problem(f"{rel(f)}:{line}", f"`{name}` mutates nothing but declares no return type")
                for line, name in untyped(lines)]
    out += label_gaps(files)
    if len(files) > 1:
        out += [Problem("std/prelude.cp", f"`{name}` is in no core or lib chapter (core 11.3)")
                for name in undocumented_prelude(str(specgate.ROOT))]
    return out

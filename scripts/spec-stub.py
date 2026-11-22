#!/usr/bin/env python3
"""Reduce spec/std/*.cp to reference docs: every fn body becomes `todo`.

The spec is signatures, doc comments and costs; implementations live in
stdlib/. Only fn bodies change: headers, docs, traits, structs and bodiless
trait methods are kept as written. `lends` fns keep their bodies, since the
checker needs the `lend place` to type the place they return.

Usage: scripts/spec-stub.py [--check] [FILE...]   (default spec/std/*.cp)
--check changes nothing and exits 1 if any fn still has a body, or if an
`@get` member is documented as costing more than O(1) to make: anything
that does work up front takes parentheses (decision 140), or if a fn that
mutates nothing declares no return type: the checker cannot type its result,
so every method called on it would go unchecked, or if a prelude name
appears in no core or lib chapter (core 11.3).
"""
import glob, json, os, re, sys

DOC_LABELS = ("cost", "traps", "example", "empty", "ties", "order")
SEQ_INPUT = re.compile(r"\b(Seq|Indexed|IndexedMut|Keyed|List<|Range<|Domain)\b|\[[^\]]*\]\w|yields")
CHOOSES = re.compile(r"^(?!is_)((arg)?(min|max)|best|first|last|find|sort|kth|nearest|lower|upper|mode|top|partition_point|search)|_(min|max)$")
PUB_FN = re.compile(r"^\s*pub\s+(?:@\w+\s+)*fn\s+(\w+)")


def doc_gaps(lines):
    """(line, name, missing labels) for every pub fn whose doc lacks a required label (README)."""
    out = []
    for i, line in enumerate(lines):
        m = PUB_FN.match(line)
        if not m:
            continue
        doc, j = [], i - 1
        while j >= 0 and lines[j].strip().startswith("///"):
            doc.insert(0, lines[j].strip()[3:].strip())
            j -= 1
        labels = {d.split(":", 1)[0] for d in doc if re.match(r"^[a-z]+:", d)}
        header = " ".join(lines[i:i + 6])
        header = header[:header.find(" = todo")] if " = todo" in header else header
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
    text = open(os.path.join(spec, "std", "prelude.cp")).read()
    names = [n.strip() for group in re.findall(r"use [\w.]+\.\{([^}]*)\}", text)
             for n in group.split(",") if n.strip()]
    docs = "".join(open(f).read() for d in ("core", "lib")
                   for f in glob.glob(os.path.join(spec, d, "*.md")))
    return [n for n in names
            if not re.search(r"`[^`\n]*\b" + re.escape(n) + r"\b[^`\n]*`", docs)]


def audit(files, write):
    base_path = os.path.join(os.path.dirname(__file__), "..", "spec", "std", "doc-baseline.json")
    counts, total, fns = {}, 0, 0
    for f in files:
        lines = open(f).read().split("\n")
        gaps = doc_gaps(lines)
        fns += sum(1 for l in lines if PUB_FN.match(l))
        counts[os.path.basename(f)] = len(gaps)
        total += len(gaps)
        for line, name, missing in gaps:
            print(f"{os.path.relpath(f)}:{line}: `{name}` lacks {', '.join(missing)}")
    print(f"spec-stub audit: {total} of {fns} pub fns lack a required doc label")
    if write:
        open(base_path, "w").write(json.dumps(counts, indent=1, sort_keys=True) + "\n")
    return 0


def over_baseline(files):
    base_path = os.path.join(os.path.dirname(__file__), "..", "spec", "std", "doc-baseline.json")
    base = json.load(open(base_path)) if os.path.exists(base_path) else {}
    out = []
    for f in files:
        n = len(doc_gaps(open(f).read().split("\n")))
        if n > base.get(os.path.basename(f), 0):
            out.append(f"{os.path.relpath(f)}: {n} pub fns lack doc labels, baseline "
                       f"{base.get(os.path.basename(f), 0)} (spec/std/README.md)")
    return out


def main(argv):
    check = "--check" in argv
    flags = ("--check", "--audit", "--write-baseline")
    files = [a for a in argv if a not in flags] or sorted(glob.glob(
        os.path.join(os.path.dirname(__file__), "..", "spec", "std", "*.cp")))
    if "--audit" in argv:
        return audit(files, "--write-baseline" in argv)
    total = 0
    for f in files:
        lines = open(f).read().split("\n")
        new, n = stub(lines)
        total += n
        if n and not check:
            open(f, "w").write("\n".join(new))
        if n:
            print(f"{os.path.relpath(f)}: {n} body(ies){' left' if check else ' stubbed'}")
        if check:
            for line, name, cost in costly_getters(lines):
                print(f"{os.path.relpath(f)}:{line}: `@get {name}` costs {cost}; "
                      "work up front takes parentheses")
                total += 1
            for line, name in untyped(lines):
                print(f"{os.path.relpath(f)}:{line}: `{name}` mutates nothing but declares "
                      "no return type")
                total += 1
    if check:
        for msg in over_baseline(files):
            print(msg)
            total += 1
    if check and len(files) > 1:
        for name in undocumented_prelude(os.path.join(os.path.dirname(__file__), "..")):
            print(f"spec/std/prelude.cp: `{name}` is in no core or lib chapter (core 11.3)")
            total += 1
    print(f"spec-stub: {total} fn body(ies) {'with bodies' if check else 'stubbed'}")
    return 1 if check and total else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

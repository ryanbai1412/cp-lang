#!/usr/bin/env python3
"""Lint std's public signatures against std-api/R1 to R13 in handbook/guidelines/std-api-checklist.md.

- R1 (#r1-every-name-comes-from-the-lexicon): a public fn named by a word
  in the lexicon's "Not" column that no row's "Name" column uses.
- R4 (#r4-nothing-is-t-never-a-sentinel): a public fn returning a number
  whose doc gives a sentinel (`-1`, `INF`, "`len` when none") for "nothing".
  Orderings and signs (`cmp`, `compare`, `*_sign`), where `-1` is a value,
  are not sentinels, nor is a value the result's arithmetic uses, whose
  doc says it is "not a sentinel" (the eertree's odd-root length `-1`).
- R6 (#r6-options-are-named-arguments-with-defaults): a public fn `f_by`,
  `f_key` or `f_with` beside a public `f`: a variant where an option belongs.
- R8 (#r8-required-arguments-come-in-reading-order): two public fns of one
  name whose shared required parameters (same name, same type) come in a
  different order.
- R9 (#r9-over-a-dt-keys-keep-their-index-space): a public fn over a `[D]T`
  taking a position parameter (`i`, `at`, `l`, `r`, ...) as a bare `Int`.
- R11 (#r11-one-form-per-operation): a top-level public fn whose first
  parameter's type also has a public method of that name: both forms.
- R13 (#r13-each-types-everyday-tasks-are-one-call-each-with-a-case): a journey in the
  R13 table with no `// journey: <name>` line in its type's case,
  spec/conformance/std-journeys/<type>.cp, or a journey line the table lacks.

Shrink-only: `--max` in scripts/check.sh is today's count.

    uv run python scripts/std-api-lint.py [--max N]
"""
import argparse, glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DOC_FN = re.compile(
    r"((?:^[ \t]*///[^\n]*\n)*)[ \t]*(?:@\w+\s+)*pub\s+fn\s+(\w+)[^\n]*?"
    r"(?:->\s*([^=:\n]+?))?\s*(?:=|:\s*$)", re.M)
SENTINEL = re.compile(r"`-1`|`-?INF`|\bINF\b|`len` (?:when|if)|`n` (?:when|if) (?:none|no|not)")
ORDERING = re.compile(r"^(cmp|compare)$|_sign$")
VARIANT = re.compile(r"_(by|key|with)$")
WORDS = {"starts_with", "ends_with"}
OWNER = re.compile(r"^(?:pub\s+)?(?:struct|enum|trait|impl(?:<[^>]*>)?)\s+(?:[\w<>, ]+\s+for\s+)?(\w+)")
PUB_FN = re.compile(r"^(\s*)(?:@\w+(?:\([^)]*\))?\s+)*pub\s+fn\s+(\w+)\s*(?:<[^>]*>)?\(\s*(?:\w+\s*:\s*(?:mut\s+|lent\s+)?(\w+))?")


def both_forms(texts):
    """std-api/R11: an operation that is both a free fn and a method of its first parameter's type."""
    methods, free = {}, []
    for rel, text in texts:
        owner = None
        for line_no, line in enumerate(text.split("\n"), 1):
            o = OWNER.match(line)
            if o:
                owner = o.group(1)
                continue
            m = PUB_FN.match(line)
            if not m:
                continue
            if m.group(1):
                methods.setdefault(owner, set()).add(m.group(2))
            elif m.group(3):
                free.append((rel, line_no, m.group(2), m.group(3)))
    for rel, line_no, name, ty in free:
        if name in methods.get(ty, ()):
            yield (f"{rel}:{line_no}: std-api/R11: `{name}` is both a free fn and a method of `{ty}`; "
                   "keep one form")


CHECKLIST = os.path.join(ROOT, "handbook", "guidelines", "std-api-checklist.md")
WORD = re.compile(r"^`([A-Za-z_]\w*)(?:\(\))?`$")


SIG = re.compile(r"^[ \t]*(?:@\w+(?:\([^)]*\))?\s+)*pub\s+fn\s+(\w+)\s*(?:<[^>]*>)?\(", re.M)
POSITION = re.compile(r"^(i|j|at|pos|idx|index|l|r|lo|hi)$")


def signatures(text):
    """(line, name, [(param, type, has_default)]) of each public fn, parameters split at depth 0."""
    for m in SIG.finditer(text):
        depth, cur, params = 0, "", []
        for ch in text[m.end():]:
            if ch in "([<{":
                depth += 1
            elif ch in ")]>}":
                if depth == 0:
                    break
                depth -= 1
            if ch == "," and depth == 0:
                params.append(cur)
                cur = ""
            else:
                cur += ch
        params.append(cur)
        parsed = []
        for p in params:
            head, _, default = p.partition("=")
            name, _, ty = head.partition(":")
            if name.strip():
                parsed.append((name.strip(), " ".join(ty.split()), bool(default.strip())))
        yield text.count("\n", 0, m.start(1)) + 1, m.group(1), parsed


def reading_order(texts):
    """std-api/R8: same-named public fns whose shared required parameters come in another order."""
    seen = {}
    for rel, text in texts:
        for line, name, params in signatures(text):
            req = [(n, t) for n, t, d in params[1:] if not d]
            for orel, oline, oreq in seen.get(name, []):
                common = [p for p in req if p in oreq]
                if len(common) >= 2 and [p for p in oreq if p in common] != common:
                    yield (f"{rel}:{line}: std-api/R8: `{name}` takes "
                           f"{', '.join(n for n, _ in common)}; {orel}:{oline} takes them in another order")
            seen.setdefault(name, []).append((rel, line, req))


def index_space(texts):
    """std-api/R9: a public fn over a `[D]T` taking a position as a bare `Int`."""
    for rel, text in texts:
        for line, name, params in signatures(text):
            if not params or not re.match(r"(mut |lent )?\[", params[0][1]):
                continue
            for n, t, _ in params[1:]:
                if t == "Int" and POSITION.match(n):
                    yield (f"{rel}:{line}: std-api/R9: `{name}` takes the position `{n}` as an `Int`; "
                           f"take the sequence's `D` (or `Cut<D>`)")


def lexicon(text):
    """(own, banned): the words R1's table uses, and its bare "Not" words no row uses."""
    section = text.split("### R1.", 1)[1].split("\n### ", 1)[0]
    own, banned = set(), set()
    for row in section.split("\n"):
        cells = [c.strip() for c in row.strip().strip("|").split("|")]
        if len(cells) != 3 or not row.startswith("|") or set(cells[1]) <= set("-"):
            continue
        for code in re.findall(r"`([^`]*)`", cells[1]):
            own.update(re.findall(r"[A-Za-z_]\w*", code))
        for code in re.findall(r"`[^`]*`", cells[2]):
            w = WORD.match(code)
            if w:
                banned.add(w.group(1))
    return own, banned - own


def off_lexicon(texts, banned):
    """std-api/R1: a public fn named by a word the lexicon bans."""
    for rel, text in texts:
        for line_no, line in enumerate(text.split("\n"), 1):
            m = PUB_FN.match(line)
            if m and m.group(2) in banned:
                yield (f"{rel}:{line_no}: std-api/R1: `{m.group(2)}` is not a lexicon word; "
                       "use the lexicon's name, or add a row in std-api-checklist.md")


JOURNEY_CASES = os.path.join(ROOT, "spec", "conformance", "std-journeys")
JOURNEY = re.compile(r"^\s*// journey: (.+?)\s*$", re.M)


def journey_table(text):
    section = text.split("### R13.", 1)[1].split("\n### ", 1)[0]
    for line in section.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        ty = re.match(r"`(\w+)", cells[0]) if len(cells) == 2 else None
        if ty:
            slug = re.sub(r"(?<!^)(?=[A-Z])", "-", ty.group(1)).lower()
            yield slug, [j.strip() for j in re.sub(r"\s*\([^)]*\)", "", cells[1]).split(",")]


def journeys(text, cases):
    for slug, wanted in journey_table(text):
        rel = f"spec/conformance/std-journeys/{slug}.cp"
        found = JOURNEY.findall(cases.get(slug, ""))
        for j in wanted:
            if j not in found:
                yield f"{rel}: std-api/R13: no `// journey: {j}` case"
        for j in found:
            if j not in wanted:
                yield f"{rel}: std-api/R13: `// journey: {j}` is not in the R13 table"


def findings():
    paths = sorted(glob.glob(os.path.join(ROOT, "std", "*.cp")))
    names = {m.group(2) for p in paths for m in DOC_FN.finditer(open(p).read())}
    for path in paths:
        text = open(path).read()
        rel = os.path.relpath(path, ROOT)
        for m in DOC_FN.finditer(text):
            doc, name, ret = m.group(1), m.group(2), (m.group(3) or "").strip()
            line = text.count("\n", 0, m.start(2)) + 1
            if ret in ("Int", "I64", "F64") and not ORDERING.search(name):
                s = SENTINEL.search(doc)
                if s and "not a sentinel" not in doc:
                    yield f"{rel}:{line}: std-api/R4: `{name}` returns {s.group(0)} for nothing; return `?{ret}`"
            v = VARIANT.search(name)
            if v and name[: v.start()] in names and name not in WORDS:
                yield f"{rel}:{line}: std-api/R6: `{name}` is a variant; make the option a named argument"
    texts = [(os.path.relpath(p, ROOT), open(p).read()) for p in paths]
    yield from both_forms(texts)
    yield from reading_order(texts)
    yield from index_space(texts)
    cases = {os.path.basename(p)[:-3]: open(p).read()
             for p in glob.glob(os.path.join(JOURNEY_CASES, "*.cp"))}
    yield from journeys(open(CHECKLIST).read(), cases)
    _, banned = lexicon(open(CHECKLIST).read())
    yield from off_lexicon(((os.path.relpath(p, ROOT), open(p).read()) for p in paths), banned)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--max", type=int, default=None)
    a = ap.parse_args()
    found = sorted(set(findings()))
    for f in found:
        print(f)
    print(f"std-api lint: {len(found)} finding(s)")
    if a.max is not None and len(found) > a.max:
        print(f"over the ratchet bound {a.max}: fix the new finding above")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

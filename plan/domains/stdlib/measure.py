#!/usr/bin/env python3
"""Stdlib metrics that no other script prints. Usage: measure.py <metric>.

  todo-split   `todo` stubs in library bodies vs in `@test` fns (spec/std)
  privilege    public spec/std names the compiler also special-cases: names
               in cpl-check's prelude.rs tables, and string literals in
               cpl-ir's lower/call.rs
  cycles       import cycles among spec/std modules via explicit `use`
  uncovered    spec/std modules with no spec/conformance/std-<m>/ cases
"""
import glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
STD = os.path.join(ROOT, "spec", "std")
TODO = re.compile(r"(?:=\s*todo|^\s*todo)(?:\s*(?://.*)?$)")  # as scripts/debt-ratchet.py


def modules():
    return sorted(os.path.basename(f)[:-3] for f in glob.glob(f"{STD}/*.cp") if not f.endswith("prelude.cp"))


def text(m):
    return open(f"{STD}/{m}.cp").read()


def pub_names():
    names = set()
    for m in modules():
        names |= set(re.findall(r"^\s*pub\s+(?:fn|struct|enum|trait|type)\s+(\w+)", text(m), re.M))
    return names


def todo_split():
    body = test = 0
    for m in modules():
        in_test = False
        for line in text(m).split("\n"):
            if line.strip() == "@test":
                in_test = True
                continue
            if in_test and line.strip() and not line[0].isspace() and not line.startswith("fn "):
                in_test = False
            if TODO.search(line):
                test += in_test
                body += not in_test
    print(f"todo-split: {body} in library bodies, {test} in @test fns, {body + test} total")


def privilege():
    pub = pub_names()
    src = open(f"{ROOT}/crates/cpl-check/src/prelude.rs").read()
    tables = set()
    for t in ("TYPE_NAMES", "VALUE_NAMES", "TRAIT_NAMES"):
        tables |= set(re.findall(r'"(\w+)"', re.search(t + r": &\[&str\] = &\[(.*?)\];", src, re.S).group(1)))
    call = set(re.findall(r'"(\w+)"', open(f"{ROOT}/crates/cpl-ir/src/lower/call.rs").read()))
    a, b = sorted(tables & pub), sorted(call & pub)
    print(f"privilege: {len(set(a) | set(b))} std names special-cased "
          f"({len(a)} in cpl-check prelude.rs tables, {len(b)} literals in cpl-ir lower/call.rs)")
    print("  prelude.rs:", " ".join(a))
    print("  call.rs:", " ".join(b))


def cycles():
    g = {m: set(re.findall(r"^\s*use std\.(\w+)", text(m), re.M)) - {m} for m in modules()}
    index, low, stack, on, out = {}, {}, [], set(), []

    def visit(v):
        index[v] = low[v] = len(index)
        stack.append(v)
        on.add(v)
        for w in g[v] & g.keys():
            if w not in index:
                visit(w)
                low[v] = min(low[v], low[w])
            elif w in on:
                low[v] = min(low[v], index[w])
        if low[v] == index[v]:
            scc = []
            while True:
                w = stack.pop()
                on.discard(w)
                scc.append(w)
                if w == v:
                    break
            if len(scc) > 1:
                out.append(sorted(scc))

    for v in g:
        if v not in index:
            visit(v)
    print(f"cycles: {len(out)} among {len(g)} modules ({sum(map(len, g.values()))} explicit use edges) {out}")


def uncovered():
    missing = [m for m in modules() if not os.path.isdir(f"{ROOT}/spec/conformance/std-{m}")]
    print(f"uncovered: {len(missing)} of {len(modules())} modules have no conformance cases {missing}")


METRICS = {"todo-split": todo_split, "privilege": privilege, "cycles": cycles, "uncovered": uncovered}
if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in METRICS:
        sys.exit(__doc__)
    METRICS[sys.argv[1]]()

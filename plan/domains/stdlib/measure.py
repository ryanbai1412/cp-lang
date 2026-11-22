#!/usr/bin/env python3
"""Stdlib metrics that no other script prints. Usage: measure.py <metric>.

  todo-split   `todo` stubs in library bodies vs in `@test` fns (spec/std)
  privilege    the builtin count agreed with tech-debt: `ir_builtin_names` in
               scripts/debt-ratchet.py (distinct names string-matched in match
               arms of cpl-ir's lower/call.rs), split into public spec/std
               names (std privilege) and the rest
  cycles       import cycles among spec/std modules via explicit `use`
  uncovered    spec/std modules with no spec/conformance/std-<m>/ cases
  prelude-usage  share of solutions that use each prelude name, written to
               plan/domains/stdlib/prelude-usage.md (for plan/QUESTIONS.md
               stdlib Q1): the reviewed ports (corpus/sets/ecnerwala.txt), the
               hill-climb spec.cp (corpus/hillclimb.json), and corpus/gold
"""
import glob, json, os, re, subprocess, sys

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
    out = subprocess.run([sys.executable, f"{ROOT}/scripts/debt-ratchet.py", "--list", "ir_builtin_names"],
                         capture_output=True, text=True, check=True).stdout.split("\n")
    names = {n.strip() for n in out[1:] if n.strip()}
    std = sorted(names & pub_names())
    print(f"privilege: {len(names)} builtin names (ir_builtin_names); {len(std)} are public std names")
    print("  std:", " ".join(std))
    print("  other:", " ".join(sorted(names - set(std))))


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


def prelude_names():
    src = re.sub(r"//.*", "", open(f"{STD}/prelude.cp").read())
    out = []
    for mod, body in re.findall(r"use std\.(\w+)\.\{([^}]*)\}", src):
        for item in body.split(","):
            item = item.split(" as ")[-1].strip()
            if item:
                out.append((item, mod))
    return out


def solution_sets():
    def first(*paths):
        return next((p for p in paths if os.path.isfile(p)), None)

    probs = f"{ROOT}/corpus/problems"
    stems = [l.strip() for l in open(f"{ROOT}/corpus/sets/ecnerwala.txt") if l.strip() and not l.startswith("#")]
    ports = [first(f"{probs}/{s}/spec.cp", f"{probs}/{s}/port.cp") for s in stems]
    climb = [f"{probs}/{s}/spec.cp" for s in json.load(open(f"{ROOT}/corpus/hillclimb.json"))["problems"]]
    gold = sorted(glob.glob(f"{ROOT}/corpus/gold/*.cp"))
    return {"ports": [p for p in ports if p], "hill-climb": [p for p in climb if os.path.isfile(p)], "gold": gold}


def code(path):
    t = open(path).read()
    t = re.sub(r'"(?:\\.|[^"\\\n])*"', '""', t)
    return re.sub(r"//.*", "", t)


def uses(t, name):
    if re.search(rf"\b(?:fn|struct|enum|trait|type|let|var)\s+{name}\b", t):
        return False
    return re.search(rf"(?<![\w.]){name}(?!\w)", t) is not None


def prelude_usage(threshold=0.05):
    sets = {k: [code(p) for p in v] for k, v in solution_sets().items()}
    both = sets["ports"] + sets["hill-climb"]
    rows = []
    for name, mod in prelude_names():
        n = {k: sum(uses(t, name) for t in v) for k, v in sets.items()}
        rows.append((sum(uses(t, name) for t in both) / len(both), name, mod, n))
    rows.sort(key=lambda r: (-r[0], r[1]))
    keep = [r for r in rows if r[0] >= threshold]
    unused = [r for r in rows if r[0] == 0]
    sizes = {k: len(v) for k, v in sets.items()}
    head = (f"{len(rows)} prelude names; {len(keep)} used by >= {threshold:.0%} of the "
            f"{len(both)} ports + hill-climb solutions; {len(unused)} used by none "
            f"(ports {sizes['ports']}, hill-climb {sizes['hill-climb']}, gold {sizes['gold']})")
    lines = [
        "# Prelude usage",
        "",
        "Generated by `python3 plan/domains/stdlib/measure.py prelude-usage`; data for",
        "`plan/QUESTIONS.md` stdlib Q1. A file uses a name when the name appears outside",
        "comments and strings, not after a `.` (a method), in a file that does not declare",
        "it. Ports: `corpus/sets/ecnerwala.txt` (`spec.cp`, else `port.cp`); hill-climb:",
        "`corpus/hillclimb.json` `spec.cp`; gold: `corpus/gold/*.cp`.",
        "",
        head + ".",
        "",
        "| Name | Module | Ports + hill-climb | Ports | Hill-climb | Gold |",
        "|---|---|---|---|---|---|",
    ]
    for share, name, mod, n in rows:
        lines.append(f"| `{name}` | `{mod}` | {share:.1%} | {n['ports']} | {n['hill-climb']} | {n['gold']} |")
    open(f"{ROOT}/plan/domains/stdlib/prelude-usage.md", "w").write("\n".join(lines) + "\n")
    print("prelude-usage:", head)


METRICS = {"todo-split": todo_split, "privilege": privilege, "cycles": cycles, "uncovered": uncovered, "prelude-usage": prelude_usage}
if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in METRICS:
        sys.exit(__doc__)
    METRICS[sys.argv[1]]()

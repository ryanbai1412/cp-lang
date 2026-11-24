#!/usr/bin/env python3
"""Every std type and pub std fn has a model case: random inputs checked against a brute force.

Enforces handbook/process/quality.md#which-test-to-write (generated suites) for handbook/domains/stdlib.md's "Correct on every input, not just picked ones" row.
A case in spec/conformance/std-models/ names its types with `// model: Type` lines, reads
`seed n` from stdin, runs `n` seeded random operations (or inputs) on them and on a
brute-force model, and prints `ok` or the first operation where they differ. A
`// brute: fn` line names a std fn the case uses as its brute force. This script runs each case
for several seeds on both engines (`cpl run` and `cpl run --release`) and shrinks a failure
to the shortest failing prefix of its operation sequence. A gap is a std type with a
pub method, or a pub top-level std fn, that no case models: a fn is
covered by its own `// model:` line, by a `// brute:` line, or by the model of the
type its `self` is.

  scripts/std-models.py [--list] [--max N] [--seeds K] [--ops N] [--cpl PATH] [case ...]
"""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CASES = ROOT / "spec/conformance/std-models"
MODEL = re.compile(r"^// model: (\w+)\s*$", re.M)
BRUTE = re.compile(r"^// brute: (\w+)\s*$", re.M)
FN = re.compile(r"^(?:@\w+ )*pub fn ([a-zA-Z]\w*)(?:<[^(]*>)?\((?:mut )?(?:self: ([A-Za-z]\w*))?")
TYPE = re.compile(r"^pub (?:struct|enum) ([A-Za-z]\w*)")
IMPL = re.compile(r"^impl(?:<[^>]*>)? (?:[\w<>, ]+ for )?([A-Za-z]\w*)")
METHOD = re.compile(r"^\s+(?:@\w+ )*pub fn \w+")
ENGINES = {"interp": [], "native": ["--release"]}


def std_types(texts):
    """Public std types with a pub method, as {name: file}."""
    found = {}
    for rel, text in texts:
        public, cur = set(), None
        for line in text.splitlines():
            if m := TYPE.match(line):
                public.add(cur := m.group(1))
            elif m := IMPL.match(line):
                cur = m.group(1)
            elif line and not line[0].isspace() and not line.startswith(("///", "@", "#")):
                cur = None
            if cur in public and METHOD.match(line):
                found.setdefault(cur, rel)
    return found


def public_fns(texts):
    """Pub top-level std fns, as {name: (file, self type or None)}."""
    found = {}
    for rel, text in texts:
        for line in text.splitlines():
            if m := FN.match(line):
                found.setdefault(m.group(1), (rel, m.group(2)))
    return found


def modelled(cases):
    """{type: case} for every `// model:` line."""
    return {t: rel for rel, text in cases for t in MODEL.findall(text)}


def gaps(texts, cases):
    types, fns, have = std_types(texts), public_fns(texts), modelled(cases)
    brute = {f for _, text in cases for f in BRUTE.findall(text)}
    out = [f"{rel}: `{t}` has no model case in spec/conformance/std-models/"
           for t, rel in sorted(types.items()) if t not in have]
    out += [f"{rel}: `{f}` has no model case in spec/conformance/std-models/"
            for f, (rel, recv) in sorted(fns.items())
            if f not in have and f not in brute and recv not in have]
    out += [f"{rel}: `// model: {t}` names no std type or pub std fn"
            for t, rel in sorted(have.items()) if t not in types and t not in fns]
    return out


def shrink(fails, n):
    """The smallest prefix length in 1..n that still fails, given `fails(n)`."""
    lo, hi = 1, n
    while lo < hi:
        mid = (lo + hi) // 2
        if fails(mid):
            hi = mid
        else:
            lo = mid + 1
    return lo


def run_case(cpl, path, engine, seed, ops):
    r = subprocess.run([cpl, "run", *ENGINES[engine], str(path)], input=f"{seed} {ops}\n",
                       capture_output=True, text=True, timeout=300,
                       env={**os.environ, "CPL_STD_DIR": str(ROOT / "std")})
    out = r.stdout.strip()
    if r.returncode == 0 and out == "ok":
        return None
    err = [ln for ln in r.stderr.splitlines() if re.match(r"(error|trap)\b", ln)]
    return err[:1] or (out or r.stderr.strip()).splitlines()[-1:]


def check(cpl, path, seeds, ops):
    rel = path.relative_to(ROOT)
    for engine in ENGINES:
        for seed in range(1, seeds + 1):
            if run_case(cpl, path, engine, seed, ops) is None:
                continue
            n = shrink(lambda k: run_case(cpl, path, engine, seed, k) is not None, ops)
            why = run_case(cpl, path, engine, seed, n) or ["?"]
            yield f"{rel}: {engine} seed {seed} fails after {n} op(s): {why[0]}"
            break


def std_texts(root=ROOT):
    return [(f"std/{p.name}", p.read_text(encoding="utf-8")) for p in sorted((root / "std").glob("*.cp"))]


def case_texts(root=ROOT):
    return [(str(p.relative_to(root)), p.read_text(encoding="utf-8"))
            for p in sorted((root / "spec/conformance/std-models").glob("*.cp"))]


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--list", action="store_true", help="print the gaps and stop")
    ap.add_argument("--max", type=int, help="fail when there are more than N gaps")
    ap.add_argument("--seeds", type=int, default=20)
    ap.add_argument("--ops", type=int, default=300)
    ap.add_argument("--cpl", default=str(ROOT / "target/release/cpl"))
    ap.add_argument("cases", nargs="*", type=Path)
    a = ap.parse_args()
    found = gaps(std_texts(), case_texts())
    if a.list:
        print("\n".join(found))
        print(f"std models: {len(found)} gap(s)")
        return 0
    paths = [p.resolve() for p in a.cases] or sorted(CASES.glob("*.cp"))
    failed = [f for p in paths for f in check(a.cpl, p, a.seeds, a.ops)]
    print("\n".join(failed) if failed else "", end="\n" if failed else "")
    print(f"std models: {len(paths)} case(s), {len(failed)} failing, {len(found)} gap(s)")
    over = a.max is not None and len(found) > a.max
    if over:
        print(f"std models: {len(found)} gaps is over the limit of {a.max}; see --list")
    return 1 if failed or over else 0


if __name__ == "__main__":
    sys.exit(main())

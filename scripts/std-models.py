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

  scripts/std-models.py [--list] [--max N] [--seeds K] [--ops N] [--cpl PATH] [--cpl-next PATH] [case ...]
"""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CASES = ROOT / "spec/conformance/std-models"
MODEL = re.compile(r"^// model: (\w+)\s*$", re.MULTILINE)
BRUTE = re.compile(r"^// brute: (\w+)\s*$", re.MULTILINE)
FN = re.compile(r"^(?:@\w+ )*pub fn ([a-zA-Z]\w*)")
SELF = re.compile(r"\s*\((?:mut )?self: ([A-Za-z]\w*)")
TYPE = re.compile(r"^pub (?:struct|enum) ([A-Za-z]\w*)")
IMPL = re.compile(r"^impl(?:<[^>]*>)? (?:[\w<>, ]+ for )?([A-Za-z]\w*)")
METHOD = re.compile(r"^\s+(?:@\w+ )*pub fn \w+")
TRAIT = re.compile(r"^(?:@\w+(?:\([^)]*\))? )*pub trait ([A-Za-z]\w*)")
DEFAULT = re.compile(r"^    (?:@\w+ )*fn ([a-zA-Z]\w*)")
ENGINES = {"interp": ["run"], "native": ["run", "--release"]}
# The reference interpreter is the tests' `cpl-next-oracle`, beside `cpl-next`.
NEXT_ENGINES = {"raw": ("cpl-next-oracle", ["--raw"]), "engine": ("cpl-next", ["--engine"])}
COMPARE = re.compile(r"^// compare: stdout\s*$", re.MULTILINE)
ON_NEXT = re.compile(r"^// engines: next\s*$", re.MULTILINE)


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


def after_generics(rest):
    """`rest` past a leading `<...>`, whose bounds may hold `(..)` rows and nested `<>`."""
    if not rest.startswith("<"):
        return rest
    depth = 0
    for i, c in enumerate(rest):
        depth += {"<": 1, ">": -1}.get(c, 0)
        if depth == 0:
            return rest[i + 1:]
    return ""


def public_fns(texts):
    """Pub top-level std fns, as {name: (file, self type or None)}."""
    found = {}
    for rel, text in texts:
        for line in text.splitlines():
            if m := FN.match(line):
                recv = SELF.match(after_generics(line[m.end():]))
                found.setdefault(m.group(1), (rel, recv and recv.group(1)))
    return found


def has_body(sig):
    """Whether a trait `fn` line has a body: a block (`:`) or `= expr` past its parameter list,
    whose own defaults (`of: T -> U = x => x`) are not one."""
    depth, i = 0, sig.find("(")
    for i in range(max(i, 0), len(sig)):
        depth += {"(": 1, ")": -1}.get(sig[i], 0)
        if depth == 0 and sig[i] == ")":
            break
    rest = sig[i + 1:].rstrip()
    return rest.endswith(":") or " = " in rest


def trait_defaults(texts):
    """The default methods of pub std traits (`Seq.sum`): std code every implementing type runs."""
    found = set()
    for _, text in texts:
        trait = False
        for line in text.splitlines():
            if line and not line[0].isspace() and not line.startswith(("///", "@")):
                trait = bool(TRAIT.match(line))
            elif trait and (m := DEFAULT.match(line)) and has_body(line):
                found.add(m.group(1))
    return found


def default_gaps(texts, cases):
    """Pub std traits' default methods no `// model:` or `// brute:` line names: std code every
    implementing type runs, unchecked against a brute force (std_default_model_gaps)."""
    have = set(modelled(cases)) | {f for _, text in cases for f in BRUTE.findall(text)}
    out = []
    for rel, text in texts:
        trait = None
        for line in text.splitlines():
            if line and not line[0].isspace() and not line.startswith(("///", "@")):
                trait = (m := TRAIT.match(line)) and m.group(1)
            elif trait and (m := DEFAULT.match(line)) and has_body(line) and m.group(1) not in have:
                out.append(f"{rel}: `{trait}.{m.group(1)}` has no model case in spec/conformance/std-models/")
    return sorted(set(out))


def public_types(texts):
    """Every `pub struct`/`pub enum` name in std."""
    return {m.group(1) for _, text in texts for line in text.splitlines() if (m := TYPE.match(line))}


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
            for t, rel in sorted(have.items())
            if t not in public_types(texts) and t not in fns and t not in trait_defaults(texts)]
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


def run_once(cmd, path, flags, stdin):
    if Path(cmd[0]).name.startswith("cpl-next"):
        argv = [*cmd, str(path), *flags]
    else:
        argv = [*cmd, *flags, str(path)]
    return subprocess.run(argv, input=stdin, capture_output=True, text=True, timeout=300,
                          env={**os.environ, "CPL_STD_DIR": str(ROOT / "std")})


def failure(r):
    err = [ln for ln in r.stderr.splitlines() if re.match(r"(error|trap)\b", ln)]
    return err[:1] or (r.stdout.strip() or r.stderr.strip()).splitlines()[-1:]


def run_case(cmd, path, flags, seed, ops, compare=False):
    """None when the case passes at `seed` over `ops` operations, else why it fails."""
    if not compare:
        r = run_once(cmd, path, flags, f"{seed} {ops}\n")
        return None if r.returncode == 0 and r.stdout.strip() == "ok" else failure(r)
    std, brute = (run_once(cmd, path, flags, f"{seed} {ops} {side}\n") for side in (0, 1))
    for r in (std, brute):
        if r.returncode:
            return failure(r)
    if std.stdout == brute.stdout:
        return None
    a, b = std.stdout.splitlines(), brute.stdout.splitlines()
    i = next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), min(len(a), len(b)))
    got = lambda xs: repr(xs[i]) if i < len(xs) else "nothing"
    return [f"line {i + 1}: the std fn prints {got(a)}, the brute force {got(b)}"]


def check(cpl, cpl_next, path, seeds, ops):
    rel = path.relative_to(ROOT)
    text = path.read_text(encoding="utf-8")
    compare = bool(COMPARE.search(text))
    on_next = bool(ON_NEXT.search(text))
    engines = NEXT_ENGINES if on_next else {e: (None, f) for e, f in ENGINES.items()}
    for engine, (binary, flags) in engines.items():
        cmd = [str(Path(cpl_next).with_name(binary))] if on_next else [cpl]
        for seed in range(1, seeds + 1):
            if run_case(cmd, path, flags, seed, ops, compare) is None:
                continue
            n = shrink(lambda k: run_case(cmd, path, flags, seed, k, compare) is not None, ops)
            why = run_case(cmd, path, flags, seed, n, compare) or ["?"]
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
    ap.add_argument("--cpl-next", default=str(ROOT / "target/release/cpl-next"))
    ap.add_argument("cases", nargs="*", type=Path)
    a = ap.parse_args()
    found = gaps(std_texts(), case_texts())
    if a.list:
        print("\n".join(found))
        print(f"std models: {len(found)} gap(s)")
        return 0
    paths = [p.resolve() for p in a.cases] or sorted(CASES.glob("*.cp"))
    failed = [f for p in paths for f in check(a.cpl, a.cpl_next, p, a.seeds, a.ops)]
    print("\n".join(failed) if failed else "", end="\n" if failed else "")
    print(f"std models: {len(paths)} case(s), {len(failed)} failing, {len(found)} gap(s)")
    over = a.max is not None and len(found) > a.max
    if over:
        print(f"std models: {len(found)} gaps is over the limit of {a.max}; see --list")
    return 1 if failed or over else 0


if __name__ == "__main__":
    sys.exit(main())

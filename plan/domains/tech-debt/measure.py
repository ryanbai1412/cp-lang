#!/usr/bin/env python3
"""Measure the architecture & tech-debt metrics in plan/domains/tech-debt/metrics.toml.

Read-only; prints `name value` lines. The gate ratchets live in
scripts/debt-ratchet.py; this adds the counts that are not ratcheted yet.

Usage:
  python3 plan/domains/tech-debt/measure.py               # every metric
  python3 plan/domains/tech-debt/measure.py NAME ...   # just these
  python3 plan/domains/tech-debt/measure.py --list NAME  # the offenders behind one count
"""

from __future__ import annotations

import hashlib
import re
import subprocess
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CRATES = ROOT / "crates"
CAP = 2000

# Back ends read the IR, not the checker; the web build must not need the CLI;
# lints read facts through the checker, not the prover directly.
FORBIDDEN_EDGES = {
    ("cpl-codegen", "cpl-check"),
    ("cpl-interp", "cpl-check"),
    ("cpl-wasm", "cpl-cli"),
    ("cpl-lint", "cpl-prover"),
}
# Library types the IR, interpreter and emitter should not know by name.
# `List`, `Str`, `Map` and `Set` are the declared core intrinsics.
STD_TYPE_RE = re.compile(
    r'"(Heap|Prefix|Deque|HashMap|Bitset|Fenwick|SegTree|Segtree|Dsu|Interval|'
    r'Window|Bag|Graph|Grid|Sparse|Lazy|Tree)"'
)
EDITION_RE = re.compile(r"is_v03|CPL_EDITION|Edition::V02|edition::")
ARM_RE = re.compile(r'^\s*"[A-Za-z_][\w.]*"(?:\s*\|\s*"[A-Za-z_][\w.]*")*\s*=>')
FN_RE = re.compile(r"^(\s*)(?:pub(?:\([^)]*\))?\s+)?fn\s+(\w+)")
IGNORE_RE = re.compile(r"^\s*#\[ignore\b")


def rs_files(*dirs: Path, tests: bool = True):
    for d in dirs:
        for p in sorted(d.rglob("*.rs")):
            if "target" in p.parts:
                continue
            if not tests and ("tests" in p.parts or p.name == "tests.rs"):
                continue
            yield p


def rel(p: Path) -> str:
    return p.relative_to(ROOT).as_posix()


def lines(p: Path) -> list[str]:
    return p.read_text(encoding="utf-8").splitlines()


def rs_over_cap():
    return [f"{rel(p)} {n}" for p in rs_files(CRATES) if (n := len(lines(p))) > CAP]


def edition_refs():
    out = []
    for p in rs_files(CRATES):
        for i, line in enumerate(lines(p), 1):
            if EDITION_RE.search(line):
                out.append(f"{rel(p)}:{i}")
    return out


def rust_ignore():
    return [
        f"{rel(p)}:{i}"
        for p in rs_files(CRATES)
        for i, line in enumerate(lines(p), 1)
        if IGNORE_RE.match(line)
    ]


def ir_builtin_names():
    names = set()
    for line in lines(CRATES / "cpl-ir/src/lower/call.rs"):
        if ARM_RE.match(line):
            names.update(re.findall(r'"([^"]+)"', line.split("=>")[0]))
    return sorted(names)


def backend_std_names():
    dirs = [CRATES / c / "src" for c in ("cpl-ir", "cpl-interp", "cpl-codegen")]
    return [
        f"{rel(p)}:{i} {m.group(1)}"
        for p in rs_files(*dirs, tests=False)
        for i, line in enumerate(lines(p), 1)
        for m in STD_TYPE_RE.finditer(line)
    ]


def _fn_bodies():
    """Yield (path, line, name, normalized body lines) for every non-test fn."""
    for p in rs_files(CRATES, tests=False):
        src = lines(p)
        i = 0
        while i < len(src):
            m = FN_RE.match(src[i])
            if m:
                indent = len(m.group(1))
                j = i + 1
                while j < len(src) and not (
                    src[j].strip() == "}" and len(src[j]) - len(src[j].lstrip()) == indent
                ):
                    j += 1
                body = [s.strip() for s in src[i + 1 : j]]
                body = [s for s in body if s and not s.startswith("//")]
                yield p, i + 1, m.group(2), body
                i = j
            i += 1


def dup_fn_bodies():
    """Extra copies of identical fn bodies (>= 6 lines) in different files."""
    groups = defaultdict(list)
    for p, ln, name, body in _fn_bodies():
        if len(body) >= 6:
            groups[hashlib.sha1("\n".join(body).encode()).hexdigest()].append(
                (rel(p), ln, name, len(body))
            )
    out = []
    for g in groups.values():
        if len({x[0] for x in g}) > 1:
            first = g[0]
            for p, ln, name, n in g[1:]:
                out.append(f"{p}:{ln} {name} ({n} lines) = {first[0]}:{first[1]}")
    return out


def _crate_edges(kind="dependencies"):
    """Workspace edges of one kind: `dependencies` (shipped) or `dev-dependencies`."""
    edges = set()
    for toml in sorted(CRATES.glob("*/Cargo.toml")):
        data = tomllib.loads(toml.read_text(encoding="utf-8"))
        me = data["package"]["name"]
        tables = [data.get(kind, {})]
        tables += [t.get(kind, {}) for t in data.get("target", {}).values()]
        for t in tables:
            for dep in t:
                if dep.startswith("cpl-") and dep != me:
                    edges.add((me, dep))
    return edges


def crate_edges():
    return [f"{a} -> {b}" for a, b in sorted(_crate_edges())]


def dev_crate_edges():
    return [f"{a} -> {b}" for a, b in sorted(_crate_edges("dev-dependencies") - _crate_edges())]


def layer_violations():
    return [f"{a} -> {b}" for a, b in sorted(_crate_edges() & FORBIDDEN_EDGES)]


def tree_walker():
    """1 while the interpreter still has its tree-walking engine beside the VM."""
    src = (CRATES / "cpl-interp/src/lib.rs").read_text(encoding="utf-8")
    return ["crates/cpl-interp/src/lib.rs: fn eval(&mut self, e: &'p Expr)"] if re.search(
        r"fn eval\(&mut self, e: &'p Expr\)", src
    ) else []


def v02_port_files():
    out = subprocess.run(
        ["git", "ls-files", "corpus/problems/*/port.cp"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    return out.split()


def spec_std_refs():
    """`spec/std` mentions outside history (research/, decisions.md)."""
    out = subprocess.run(
        ["git", "grep", "-n", "spec/std", "--", ".", ":!research", ":!decisions.md",
         ":!plan/domains/tech-debt"],
        cwd=ROOT, capture_output=True, text=True,
    ).stdout
    return out.splitlines()


def direct_python():
    """`python3 ...` calls in the gate and the justfile (target: `uv run`)."""
    return [
        f"{f}:{i}"
        for f in ("scripts/check.sh", "justfile")
        for i, line in enumerate((ROOT / f).read_text(encoding="utf-8").splitlines(), 1)
        if re.search(r"(^|[\s(])python3 ", line) and not line.lstrip().startswith("#")
    ]


def stale_arch_paths():
    """Backticked repo paths in ARCHITECTURE docs that do not exist."""
    docs = [ROOT / "plan/ARCHITECTURE.md", *sorted((ROOT / "plan/domains").glob("*/ARCHITECTURE.md"))]
    out = []
    for d in docs:
        for i, line in enumerate(lines(d), 1):
            for m in re.finditer(r"`((?:crates|scripts|spec|std|stdlib|web|site|editors|corpus|plan|bench|tools|research|examples|proto|design)/[\w./-]+)`", line):
                path = m.group(1).rstrip("/.")
                if "*" in path or "<" in path:
                    continue
                if not (ROOT / path).exists():
                    out.append(f"{rel(d)}:{i} {path}")
    return out


METRICS = {
    "rs_over_cap": rs_over_cap,
    "edition_refs": edition_refs,
    "rust_ignore": rust_ignore,
    "ir_builtin_names": ir_builtin_names,
    "backend_std_names": backend_std_names,
    "dup_fn_bodies": dup_fn_bodies,
    "crate_edges": crate_edges,
    "dev_crate_edges": dev_crate_edges,
    "layer_violations": layer_violations,
    "tree_walker": tree_walker,
    "v02_port_files": v02_port_files,
    "spec_std_refs": spec_std_refs,
    "direct_python": direct_python,
    "stale_arch_paths": stale_arch_paths,
}


def main(argv: list[str]) -> int:
    show = argv[:1] == ["--list"]
    names = argv[1:] if show else argv
    for n in names or METRICS:
        if n not in METRICS:
            print(f"unknown metric {n}; one of {', '.join(METRICS)}", file=sys.stderr)
            return 2
        items = METRICS[n]()
        print(f"{n} {len(items)}")
        if show:
            for x in items:
                print(f"  {x}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""Tech-debt ratchet: architecture counts that may only go down.

Every count below is measured from the tree and compared with
`scripts/debt-ratchet.json`. A count above its baseline fails the gate; the
only way past is `--update` in the same commit plus a `decisions.md` line
saying why. Lower counts pass and print a hint to tighten the baseline.

  rs_file_size_allowlist  `.rs` files under `crates/` over `max_rs_lines` (2000);
                          an allowlisted file may not grow, a new one fails
  crate_edges             shipped `[dependencies]` between workspace crates; a
                          new edge fails until the baseline and plan/ARCHITECTURE.md
                          both list it
  stdlib_todo ...         one count per entry in COUNTERS

Usage:
  python3 scripts/debt-ratchet.py --check          # the gate
  python3 scripts/debt-ratchet.py --update         # rewrite the baseline
  python3 scripts/debt-ratchet.py --show [NAME..]  # `name count` per counter
  python3 scripts/debt-ratchet.py --list NAME      # the offenders behind one count
"""

from __future__ import annotations

import argparse
import glob
import hashlib
import json
import os
import re
import subprocess
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_RATCHET = ROOT / "scripts" / "debt-ratchet.json"
DEFAULT_MAX_RS_LINES = 2000

TODO_STUB_RE = re.compile(r"(?:=\s*todo|^\s*todo)(?:\s*(?://.*)?$)")
RUST_IGNORE_RE = re.compile(r"^\s*#\[ignore\b")
EDITION_RE = re.compile(r"is_v03|CPL_EDITION|Edition::V02|edition::")
# A match arm on string literals: `"name" | "other" =>`.
ARM_RE = re.compile(r'^\s*"[A-Za-z_][\w.]*"(?:\s*\|\s*"[A-Za-z_][\w.]*")*\s*=>')
# Library types the IR, interpreter and emitter should not know by name;
# `List`, `Str`, `Map` and `Set` are the core intrinsics.
STD_TYPE_RE = re.compile(
    r'"(Heap|Prefix|Deque|HashMap|Bitset|Fenwick|SegTree|Segtree|Dsu|Interval|'
    r'Window|Bag|Graph|Grid|Sparse|Lazy|Tree)"'
)
FN_RE = re.compile(r"^(\s*)(?:pub(?:\([^)]*\))?\s+)?fn\s+(\w+)")
ARCH_PATH_RE = re.compile(
    r"`((?:crates|scripts|spec|std|stdlib|web|site|editors|corpus|plan|bench|tools|"
    r"research|examples|proto|design)/[\w./-]+)`"
)
# Back ends read the IR, not the checker; the web build does not need the CLI;
# lints read facts through the checker, not the prover.
FORBIDDEN_EDGES = {
    ("cpl-codegen", "cpl-check"),
    ("cpl-interp", "cpl-check"),
    ("cpl-wasm", "cpl-cli"),
    ("cpl-lint", "cpl-prover"),
}
BACK_ENDS = ("cpl-ir", "cpl-interp", "cpl-codegen")
DUP_MIN_LINES = 6


def _rs_files(root: Path, *subdirs: str, tests: bool = True):
    for sub in subdirs or ("crates",):
        base = root / sub
        for dirpath, dirnames, filenames in os.walk(base):
            dirnames[:] = sorted(d for d in dirnames if d != "target" and (tests or d != "tests"))
            for fname in sorted(filenames):
                if fname.endswith(".rs") and (tests or fname != "tests.rs"):
                    yield Path(dirpath) / fname


def _rel(root: Path, p: Path) -> str:
    return p.relative_to(root).as_posix()


def _lines(p: Path) -> list[str]:
    return p.read_text(encoding="utf-8").splitlines()


def _git(root: Path, *args: str) -> list[str]:
    out = subprocess.run(["git", *args], cwd=root, capture_output=True, text=True)
    return out.stdout.splitlines() if out.returncode == 0 else []


def stdlib_todo(root: Path) -> list[str]:
    out = []
    for sub in ("spec/std", "std", "stdlib"):
        for path_str in sorted(glob.glob(str(root / sub / "*.cp"))):
            p = Path(path_str)
            out += [f"{_rel(root, p)}:{i}" for i, line in enumerate(_lines(p), 1)
                    if TODO_STUB_RE.search(line)]
    return out


def rust_ignore(root: Path) -> list[str]:
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root)
            for i, line in enumerate(_lines(p), 1) if RUST_IGNORE_RE.match(line)]


def pending_tests(root: Path) -> list[str]:
    pending = root / "crates" / "cpl-cli" / "tests" / "pending"
    return sorted(_rel(root, Path(d) / f) for d, _, fs in os.walk(pending) for f in fs)


def edition_refs(root: Path) -> list[str]:
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root)
            for i, line in enumerate(_lines(p), 1) if EDITION_RE.search(line)]


def ir_builtin_names(root: Path) -> list[str]:
    """Distinct names string-matched in lowering: stdlib privilege."""
    call = root / "crates/cpl-ir/src/lower/call.rs"
    names: set[str] = set()
    for line in _lines(call) if call.is_file() else []:
        if ARM_RE.match(line):
            names.update(re.findall(r'"([^"]+)"', line.split("=>")[0]))
    return sorted(names)


def backend_std_names(root: Path) -> list[str]:
    subdirs = [f"crates/{c}/src" for c in BACK_ENDS]
    return [f"{_rel(root, p)}:{i} {m.group(1)}" for p in _rs_files(root, *subdirs, tests=False)
            for i, line in enumerate(_lines(p), 1) for m in STD_TYPE_RE.finditer(line)]


def _fn_bodies(root: Path):
    """(path, line, name, body) for every non-test fn; body is trimmed, comment-free."""
    for p in _rs_files(root, tests=False):
        src = _lines(p)
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
                yield p, i + 1, m.group(2), [s for s in body if s and not s.startswith("//")]
                i = j
            i += 1


def dup_fn_bodies(root: Path) -> list[str]:
    """Each extra copy of an identical fn body (>= DUP_MIN_LINES) in another file."""
    groups = defaultdict(list)
    for p, ln, name, body in _fn_bodies(root):
        if len(body) >= DUP_MIN_LINES:
            key = hashlib.sha1("\n".join(body).encode()).hexdigest()
            groups[key].append((_rel(root, p), ln, name, len(body)))
    out = []
    for g in groups.values():
        if len({x[0] for x in g}) > 1:
            first = g[0]
            out += [f"{p}:{ln} {name} ({n} lines) = {first[0]}:{first[1]}" for p, ln, name, n in g[1:]]
    return out


def _crate_edges(root: Path, kind: str = "dependencies") -> set[tuple[str, str]]:
    edges = set()
    for toml in sorted((root / "crates").glob("*/Cargo.toml")):
        data = tomllib.loads(toml.read_text(encoding="utf-8"))
        me = data.get("package", {}).get("name", toml.parent.name)
        tables = [data.get(kind, {})] + [t.get(kind, {}) for t in data.get("target", {}).values()]
        edges |= {(me, dep) for t in tables for dep in t if dep.startswith("cpl-") and dep != me}
    return edges


def crate_edges(root: Path) -> list[str]:
    return [f"{a} -> {b}" for a, b in sorted(_crate_edges(root))]


def dev_crate_edges(root: Path) -> list[str]:
    """Test-only edges (informational)."""
    dev = _crate_edges(root, "dev-dependencies") - _crate_edges(root)
    return [f"{a} -> {b}" for a, b in sorted(dev)]


def layer_violations(root: Path) -> list[str]:
    return [f"{a} -> {b}" for a, b in sorted(_crate_edges(root) & FORBIDDEN_EDGES)]


def tree_walker(root: Path) -> list[str]:
    """The interpreter's tree-walking engine beside the VM."""
    lib = root / "crates/cpl-interp/src/lib.rs"
    src = lib.read_text(encoding="utf-8") if lib.is_file() else ""
    hit = re.search(r"fn eval\(&mut self, e: &'p Expr\)", src)
    return ["crates/cpl-interp/src/lib.rs: fn eval(&mut self, e: &'p Expr)"] if hit else []


def v02_port_files(root: Path) -> list[str]:
    return _git(root, "ls-files", "corpus/problems/*/port.cp")


def spec_std_refs(root: Path) -> list[str]:
    """`spec/std` mentions outside history (informational until `std/` moves)."""
    return _git(root, "grep", "-n", "spec/std", "--", ".", ":!research", ":!decisions.md",
                ":!plan/domains/tech-debt", ":!scripts/debt-ratchet.py")


def direct_python(root: Path) -> list[str]:
    """`python3 ...` calls in the gate and the justfile (target: `uv run`)."""
    out = []
    for f in ("scripts/check.sh", "justfile"):
        p = root / f
        for i, line in enumerate(_lines(p) if p.is_file() else [], 1):
            if re.search(r"(^|[\s(])python3 ", line) and not line.lstrip().startswith("#"):
                out.append(f"{f}:{i}")
    return out


def stale_arch_paths(root: Path) -> list[str]:
    """Backticked repo paths in ARCHITECTURE docs that do not exist."""
    docs = [root / "plan/ARCHITECTURE.md", *sorted((root / "plan/domains").glob("*/ARCHITECTURE.md"))]
    out = []
    for d in (d for d in docs if d.is_file()):
        for i, line in enumerate(_lines(d), 1):
            for m in ARCH_PATH_RE.finditer(line):
                path = m.group(1).rstrip("/.")
                if "*" not in path and "<" not in path and not (root / path).exists():
                    out.append(f"{_rel(root, d)}:{i} {path}")
    return out


# name -> (label, measure, ratcheted). Informational counts are shown, never gated.
COUNTERS = {
    "stdlib_todo": ("stdlib todo stubs", stdlib_todo, True),
    "rust_ignore": ("Rust #[ignore] attributes", rust_ignore, True),
    "pending_tests": ("crates/cpl-cli/tests/pending/ files", pending_tests, True),
    "edition_refs": ("dead 0.2 edition references", edition_refs, True),
    "ir_builtin_names": ("names string-matched in lower/call.rs", ir_builtin_names, True),
    "backend_std_names": ("std type names in IR/interp/codegen", backend_std_names, True),
    "dup_fn_bodies": ("copied fn bodies across files", dup_fn_bodies, True),
    "layer_violations": ("forbidden shipped crate edges", layer_violations, True),
    "tree_walker": ("second interpreter engine", tree_walker, True),
    "v02_port_files": ("v0.2 corpus port.cp files", v02_port_files, True),
    "direct_python": ("direct python3 calls in gate/justfile", direct_python, True),
    "stale_arch_paths": ("missing paths cited by ARCHITECTURE docs", stale_arch_paths, False),
    "crate_edges": ("shipped crate edges", crate_edges, False),
    "dev_crate_edges": ("test-only crate edges", dev_crate_edges, False),
    "spec_std_refs": ("spec/std references", spec_std_refs, False),
}


def rs_over_cap(root: Path, max_rs_lines: int) -> dict[str, int]:
    over = {}
    for p in _rs_files(root):
        n = len(_lines(p))
        if n > max_rs_lines:
            over[_rel(root, p)] = n
    return dict(sorted(over.items(), key=lambda kv: (-kv[1], kv[0])))


def measure(root: Path = ROOT, max_rs_lines: int = DEFAULT_MAX_RS_LINES) -> dict:
    """Current file sizes over the cap, every ratcheted count, and the crate edges."""
    data: dict = {"max_rs_lines": max_rs_lines, "rs_file_size_allowlist": rs_over_cap(root, max_rs_lines)}
    for name, (_, fn, gated) in COUNTERS.items():
        if gated:
            data[name] = len(fn(root))
    data["crate_edges"] = crate_edges(root)
    return data


def evaluate(actual: dict, baseline: dict) -> tuple[list[str], list[str], list[str]]:
    """Return (errors, warnings, notes) comparing `actual` against `baseline`.

    A counter or the edge list missing from the baseline is not checked yet.
    """
    errors: list[str] = []
    warnings: list[str] = []
    notes: list[str] = []

    max_lines = int(baseline.get("max_rs_lines", DEFAULT_MAX_RS_LINES))
    allowlist: dict[str, int] = baseline.get("rs_file_size_allowlist", {})
    for rel, lines in sorted(actual.get("rs_file_size_allowlist", {}).items()):
        cap = allowlist.get(rel)
        if cap is None:
            errors.append(f"{rel}: {lines} lines exceeds the {max_lines}-line cap (split it; the allowlist may only shrink)")
        elif lines > cap:
            errors.append(f"{rel}: grew from {cap} to {lines} lines (allowlist may only shrink)")
        elif lines < cap:
            notes.append(f"{rel}: shrank from {cap} to {lines} lines")
    for rel, cap in sorted(allowlist.items()):
        if rel not in actual.get("rs_file_size_allowlist", {}):
            notes.append(f"{rel}: now <= {max_lines} lines (was {cap}; can be removed from allowlist)")

    for name, (label, _, gated) in COUNTERS.items():
        if not gated or name not in baseline or name not in actual:
            continue
        cur, base = int(actual[name]), int(baseline[name])
        if cur > base:
            errors.append(
                f"{label}: increased from {base} to {cur} (ratchet may only go down; "
                f"`--list {name}` shows them)"
            )
        elif cur < base:
            notes.append(f"{label}: improved from {base} to {cur}")

    if "crate_edges" in baseline and "crate_edges" in actual:
        known, now = set(baseline["crate_edges"]), set(actual["crate_edges"])
        for edge in sorted(now - known):
            errors.append(
                f"new crate edge {edge}: add it to plan/ARCHITECTURE.md (A4. Crates) and run --update"
            )
        for edge in sorted(known - now):
            notes.append(f"crate edge {edge} removed")

    return errors, warnings, notes


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Check or update the tech-debt ratchet.")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="Verify against the ratchet file.")
    group.add_argument("--update", action="store_true", help="Write current measurements to the ratchet file.")
    group.add_argument("--show", nargs="*", metavar="NAME", help="Print `name count` for counters.")
    group.add_argument("--list", metavar="NAME", help="Print the offenders behind one counter.")
    parser.add_argument("--ratchet", type=Path, default=DEFAULT_RATCHET, help="Path to debt-ratchet.json.")
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root.")
    args = parser.parse_args(argv)

    max_lines = DEFAULT_MAX_RS_LINES
    if args.ratchet.is_file():
        max_lines = int(json.loads(args.ratchet.read_text(encoding="utf-8")).get("max_rs_lines", max_lines))

    if args.show is not None or args.list:
        names = [args.list] if args.list else (args.show or ["rs_over_cap", *COUNTERS])
        for name in names:
            if name == "rs_over_cap":
                items = [f"{k} {v}" for k, v in rs_over_cap(args.root, max_lines).items()]
            elif name in COUNTERS:
                items = COUNTERS[name][1](args.root)
            else:
                print(f"debt-ratchet: unknown counter {name}; one of rs_over_cap, {', '.join(COUNTERS)}",
                      file=sys.stderr)
                return 2
            print(f"{name} {len(items)}")
            if args.list:
                print("\n".join(f"  {x}" for x in items))
        return 0

    if args.update:
        data = measure(args.root, max_lines)
        args.ratchet.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
        counts = ", ".join(f"{k}={data[k]}" for k, (_, _, g) in COUNTERS.items() if g)
        print(f"debt-ratchet: wrote {args.ratchet} (rs>{max_lines}={len(data['rs_file_size_allowlist'])}, {counts})")
        return 0

    if not args.ratchet.is_file():
        print(f"debt-ratchet: missing {args.ratchet}; run --update", file=sys.stderr)
        return 1

    baseline = json.loads(args.ratchet.read_text(encoding="utf-8"))
    actual = measure(args.root, max_lines)
    errors, warnings, notes = evaluate(actual, baseline)

    for w in warnings:
        print(f"debt-ratchet: WARNING: {w}", file=sys.stderr)
    for e in errors:
        print(f"debt-ratchet: ERROR: {e}", file=sys.stderr)
    counts = ", ".join(
        f"{k}={actual[k]}/{baseline[k]}" for k, (_, _, g) in COUNTERS.items() if g and k in baseline
    )
    print(
        f"debt-ratchet: rs>{max_lines}={len(actual['rs_file_size_allowlist'])} "
        f"(allowlist {len(baseline.get('rs_file_size_allowlist', {}))}), {counts}"
    )
    if errors:
        print("debt-ratchet: a deliberate increase needs `--update` in the same commit and a decisions.md line",
              file=sys.stderr)
    if notes:
        print(f"debt-ratchet: {len(notes)} item(s) improved; run `python3 scripts/debt-ratchet.py --update` to tighten")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())

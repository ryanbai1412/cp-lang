#!/usr/bin/env python3
"""Tech-debt ratchet: architecture counts that may only go down.

Every count below is measured from the tree and compared with
`scripts/debt-ratchet.json`. A count above its baseline fails the gate, and
a baseline above main's fails too unless the same change rewrites the code
that measures it (`unexplained_bumps`): new debt is removed, never admitted.
Lower counts pass and print a hint to tighten the baseline.
Why these ratchets exist: handbook/process/quality.md#accountability.

  rs_file_size_allowlist  `.rs` files under `crates/` over `max_rs_lines` (2000);
                          an allowlisted file may not grow, a new one fails
  crate_edges             shipped `[dependencies]` between workspace crates; a
                          new edge fails until the baseline and handbook/design/architecture.md
                          both list it
  rs_near_cap             `.rs` files in the last 100 lines under the cap; a file
                          entering the band fails (one shrinking into it from over
                          the cap does not)
  stdlib_todo ...         one count per entry in COUNTERS

Usage:
  python3 scripts/debt-ratchet.py --check          # the gate
  python3 scripts/debt-ratchet.py --update         # rewrite the baseline
  python3 scripts/debt-ratchet.py --update NAME... # tighten only these (rs_over_cap, crate_edges, a counter)
  python3 scripts/debt-ratchet.py --show [NAME..]  # `name count` per counter
  python3 scripts/debt-ratchet.py --list NAME      # the offenders behind one count
"""

from __future__ import annotations

import argparse
import ast
import bisect
import difflib
import fnmatch
import functools
import glob
import hashlib
import itertools
import json
import os
import re
import subprocess
import sys
import tempfile
from typing import Iterator
import tomllib
import zlib
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_RATCHET = ROOT / "scripts" / "debt-ratchet.json"
DEFAULT_MAX_RS_LINES = 2000

TODO_STUB_RE = re.compile(r"(?:=\s*todo|^\s*todo)(?:\s*(?://.*)?$)")
DEAD_CODE_RE = re.compile(r"allow\([^)]*\bdead_code\b")
RUST_IGNORE_RE = re.compile(r"^\s*#\[ignore\b")
EDITION_RE = re.compile(r"is_v03|spec_edition|CPL_EDITION|Edition::V02|edition::")
# A match arm on string literals: `"name" | "other" =>`.
ARM_RE = re.compile(r'^\s*"[A-Za-z_][\w.]*"(?:\s*\|\s*"[A-Za-z_][\w.]*")*\s*=>')
# The same names in a tuple arm: `(Ty::Range(inner), "nth" | "at") if ... =>`.
TUPLE_ARM_RE = re.compile(
    r'^\s*\(.*?,\s*("[A-Za-z_][\w.]*"(?:\s*\|\s*"[A-Za-z_][\w.]*")*)\s*\)\s*(?:if\b.*)?(?:=>|$)')
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
ARCH_DOC = "handbook/design/architecture.md"


def crate_rules(root: Path) -> dict[str, list[str]]:
    """`### Crate rules` in architecture.md: rule (`back end`, `consumer`,
    `forbidden edge`) -> its backticked crates or `a -> b` edges."""
    doc = root / ARCH_DOC
    if not doc.is_file():
        return {}
    m = re.search(r"^### Crate rules\n(.*?)(?=^#|\Z)", doc.read_text(encoding="utf-8"), re.S | re.M)
    if not m:
        raise SystemExit(f"debt-ratchet: {ARCH_DOC} has no `### Crate rules` table")
    return {rule: re.findall(r"`([^`]+)`", cells)
            for rule, cells in re.findall(r"^\|\s*([a-z][a-z ]*?)\s*(?:\([^)]*\))?\s*\|(.*)\|\s*$", m[1], re.M)}


def forbidden_edges(root: Path) -> set[tuple[str, str]]:
    rules = crate_rules(root)
    named = {tuple(e.split(" -> ", 1)) for e in rules.get("forbidden edge", []) if " -> " in e}
    return named | layer_order_edges(rules.get("layer order", []))


def layer_order_edges(layers: list[str]) -> set[tuple[str, str]]:
    """`layer order` lists crates from the bottom up: a crate reads only the crates before it.
    `a + b` are siblings, which read neither each other nor anything after them."""
    ranks = [layer.split(" + ") for layer in layers]
    return {(a, b) for i, here in enumerate(ranks) for a in here
            for above in ranks[i:] for b in above if a != b}


DUP_MIN_LINES = 6
# Near copies: Jaccard similarity of 4-token shingles (numbers folded to `0`),
# between fns in different crates; exact copies count under dup_fn_bodies.
NEAR_DUP_MIN = 0.75
NEAR_DUP_COMMON = 30  # a shingle in more fns than this is boilerplate, not evidence
TOKEN_RE = re.compile(r'"(?:\\.|[^"\\])*"|\d[\w.]*|\w+|\S')
# Tools that read facts through cpl-check rather than recompute them.
CORE_TYPES = {"Int", "Str", "Bool", "F64", "Char", "List", "Map", "Set", "Mod"}
# Comparing text against a type name (`== "List"`, `"Int" =>`, `starts_with("Map")`).
TYPE_CMP_RE = re.compile(
    r'(?:[=!]=\s*|(?:starts_with|ends_with|strip_prefix)\(|matches!\([^,]*,\s*)"([A-Z]\w*)'
    r'|"([A-Z]\w*)"\s*(?:=>|\|)'
)
ECODE_RE = re.compile(r'"(E\d{4})')
NEAR_CAP_LINES = 100


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


def _git(root: Path, *args: str, env: dict[str, str] | None = None) -> list[str]:
    out = subprocess.run(["git", *args], cwd=root, capture_output=True, text=True,
                         env=_git_env() if env is None else env)
    return out.stdout.splitlines() if out.returncode == 0 else []


def std_gaps(root: Path) -> list[str]:
    """Each open line of `std/GAPS.md` (a std body waiting on the compiler) as `std/GAPS.md:line`."""
    p = root / "std" / "GAPS.md"
    if not p.is_file():
        return []
    return [f"std/GAPS.md:{i}" for i, line in enumerate(_lines(p), 1) if line.startswith("- ")]


def stdlib_todo(root: Path) -> list[str]:
    """Each `todo` stub as `path:line`, with ` @test` when it sits in an `@test` fn."""
    out = []
    for sub in ("std", "stdlib"):
        for path_str in sorted(glob.glob(str(root / sub / "*.cp"))):
            p = Path(path_str)
            in_test = False
            for i, line in enumerate(_lines(p), 1):
                if line.strip() == "@test":
                    in_test = True
                    continue
                if in_test and line.strip() and not line[0].isspace() and not line.startswith("fn "):
                    in_test = False
                if TODO_STUB_RE.search(line):
                    out.append(f"{_rel(root, p)}:{i}" + (" @test" if in_test else ""))
    return out


def rust_ignore(root: Path) -> list[str]:
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root)
            for i, line in enumerate(_lines(p), 1) if RUST_IGNORE_RE.match(line)]


def dead_code_allows(root: Path) -> list[str]:
    """`allow(dead_code)` attributes: dead code is deleted, not silenced."""
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root)
            for i, line in enumerate(_lines(p), 1) if DEAD_CODE_RE.search(line)]


def external_crates(root: Path) -> list[str]:
    """Third-party packages in Cargo.lock; each new one is a reviewed decision."""
    lock = root / "Cargo.lock"
    text = lock.read_text(encoding="utf-8") if lock.is_file() else ""
    return sorted(f"{m.group(1)} {m.group(2)}" for block in text.split("[[package]]")
                  if "\nsource = " in block
                  for m in [re.search(r'name = "([^"]+)"\nversion = "([^"]+)"', block)] if m)


# Where history and measured lists are kept, or the facts' own sources.
WRITTEN_TWICE_SKIP = ("plan/", "research/", "corpus/", "spec/conformance/", "decisions.md", "Cargo.",
                      "scripts/debt-ratchet.json", "scripts/test_debt_ratchet.py", "crates/cpl-cli/tests/")
# Copies a gate already compares with the source: the crate map (`crate_edges`), the std
# lint baseline (scripts/std-lint-ratchet.py), search's expected answers (the test fails
# on an answer page the site no longer has) and the per-file silent unknowns (the test
# fails unless each count equals the checker's facts).
# The ratchet names crates only in the paths its gates read (stale_arch_paths
# fails on a missing one) and in those gates' descriptions; crate rules are architecture.md's.
WRITTEN_TWICE_CHECKED = {"handbook/design/architecture.md", "scripts/std-lint-baseline.txt",
                         "web/tests/search-tasks.mjs", "scripts/debt-ratchet.py",
                         "crates/cpl-conform/silent_unknowns.txt"}


def written_twice(root: Path) -> list[str]:
    """Hand-kept files that list most of a fact a source already lists.

    Rule: handbook/design/sources-of-truth.md#never-written-twice. A fact is
    the set of lints (`spec/lints/*/`), crates (`crates/*/Cargo.toml`) or std
    modules (`std/*.cp`, named as `std.<m>` or `std/<m>`); a tracked file outside history, ratchet baselines and
    gen.toml's outputs that names at least half of one (and at least 8) is a
    second copy that goes stale. Generate it, or have it point at the source.
    """
    facts = {
        "lints": {p.name for p in (root / "spec/lints").glob("*/") if p.is_dir()},
        "crates": {p.parent.name for p in (root / "crates").glob("*/Cargo.toml")},
        "std modules": {p.stem for p in (root / "std").glob("*.cp")},
    }
    gen = root / "gen.toml"
    owned = {_rel(root, Path(g)) for e in (tomllib.loads(gen.read_text())["gen"] if gen.is_file() else [])
             for pat in e["files"] for g in glob.glob(str(root / pat), recursive=True)}
    try:
        files = subprocess.run(["git", "-C", str(root), "ls-files"], capture_output=True, text=True,
                               env=_git_env(), check=True).stdout.split()
    except (OSError, subprocess.CalledProcessError):
        files = []
    found = []
    for f in files:
        if f.startswith(WRITTEN_TWICE_SKIP) or Path(f).name.startswith("Cargo.") or f in owned \
                or f in WRITTEN_TWICE_CHECKED or "worklog" in f \
                or not re.search(r"\.(md|mdx|rs|py|toml|txt|sh|json|ts|js|mjs)$", f):
            continue
        try:
            text = (root / f).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        words = set(re.findall(r"[A-Za-z0-9_-]+", text))
        modules = set(re.findall(r"\bstd[./]([a-z0-9_]+)", text))
        for fact, names in facts.items():
            n = len((modules if fact == "std modules" else words) & names)
            if names and n >= max(8, (len(names) + 1) // 2):
                found.append(f"{f}: {n} of {len(names)} {fact}")
    return found


def pending_tests(root: Path) -> list[str]:
    pending = root / "crates" / "cpl-cli" / "tests" / "pending"
    return sorted(_rel(root, Path(d) / f) for d, _, fs in os.walk(pending) for f in fs)


def edition_refs(root: Path) -> list[str]:
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root)
            for i, line in enumerate(_lines(p), 1) if EDITION_RE.search(line)]


def _old_std(root: Path) -> Path:
    """The std the old crates read: the pinned copy once it is vendored, so a name added to
    std/ for cpl-next is not one the old pipeline knows."""
    frozen = root / FROZEN_STD
    return frozen if frozen.is_dir() else root / "std"


def ir_builtin_names(root: Path) -> list[str]:
    """Distinct names string-matched in lowering: stdlib privilege."""
    lower = root / "crates/cpl-ir/src/lower"
    files = [lower / "call.rs", *sorted((lower / "call").glob("*.rs"))]
    names: set[str] = set()
    for line in (ln for f in files if f.is_file() for ln in _lines(f)):
        if ARM_RE.match(line):
            names.update(re.findall(r'"([^"]+)"', line.split("=>")[0]))
        elif m := TUPLE_ARM_RE.match(line):
            names.update(re.findall(r'"([^"]+)"', m.group(1)))
    return sorted(names)


def prelude_privilege(root: Path) -> list[str]:
    """Public std names the checker's prelude.rs tables also know: stdlib privilege."""
    src = root / "crates/cpl-check/src/prelude.rs"
    if not src.is_file():
        return []
    text = src.read_text()
    tables: set[str] = set()
    for t in ("TYPE_NAMES", "VALUE_NAMES", "TRAIT_NAMES"):
        m = re.search(t + r": &\[&str\] = &\[(.*?)\];", text, re.S)
        if m:
            tables.update(re.findall(r'"(\w+)"', m.group(1)))
    public: set[str] = set()
    for p in sorted((_old_std(root)).glob("*.cp")):
        public.update(re.findall(r"^\s*pub\s+(?:fn|struct|enum|trait|type)\s+(\w+)", p.read_text(), re.M))
    return sorted(tables & public)


def lint_std_names(root: Path) -> list[str]:
    """String literals in lint rules that name a public std fn: lints keyed on library API."""
    public: set[str] = set()
    for p in sorted((root / "std").glob("*.cp")):
        public.update(re.findall(r"^\s*pub\s+fn\s+(\w+)", p.read_text(), re.M))
    return [f"{_rel(root, p)}:{i} {name}" for p in _rs_files(root, "crates/cpl-lint/src/rules", tests=False)
            for i, line in enumerate(_lines(p), 1) for name in re.findall(r'"([a-z_]\w*)"', line)
            if name in public]


def lint_name_matchers(root: Path) -> list[str]:
    """Lint rule lines that decide what a call is by comparing its written name
    against a std fn (`method.text == std_fn::PUSH`, a `std_fn::SORT =>` arm)
    instead of asking the checker which fn it calls (`Facts::callee`): a
    user's own `push` fires them. Names a lint writes into its help or fix
    don't count, nor a line that asks the checker (`is_std`, `Calls::is`,
    `Calls::name`, or `Calls::unresolved` for the written path of a member
    it could not resolve), nor an arm of a `match` on the checker's callee
    name.
    `lint_std_names` keeps every such name on `std_fn`."""
    facts = re.compile(r"\bis_std\(|\.is\(|\.name\(|\.unresolved\(")
    compare = re.compile(r"(==|!=)\s*std_fn::|std_fn::\w+\s*(==|!=)|contains\(std_fn::")
    arm = re.compile(r"std_fn::\w+\s*(=>|\|)|\|\s*std_fn::|matches!\(.*std_fn::")
    written = re.compile(r"\.text\b|as_str\(\)")

    def scrutinee(lines: list[str], i: int) -> str:
        """The `match` an arm on line `i` belongs to: the nearest one above
        it that is indented less."""
        indent = len(lines[i]) - len(lines[i].lstrip())
        for line in reversed(lines[:i]):
            if "match " in line and len(line) - len(line.lstrip()) < indent:
                return line
        return ""

    out = []
    for p in _rs_files(root, "crates/cpl-lint/src/rules", tests=False):
        lines = _lines(p)
        for i, line in enumerate(lines):
            if facts.search(line):
                continue
            by_arm = arm.search(line) and (
                written.search(line) or written.search(scrutinee(lines, i))
            )
            if compare.search(line) or by_arm:
                out.append(f"{_rel(root, p)}:{i + 1}")
    return out


def lint_type_syntax(root: Path) -> list[str]:
    """Lint rules reading a written type (`TypeKind::`) to decide what a value is, where
    the checker's facts know it (handbook/guidelines/lints-checklist.md, lints/R4)."""
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, "crates/cpl-lint/src/rules", tests=False)
            if not p.name.endswith("_tests.rs")
            for i, line in enumerate(_lines(p), 1) if "TypeKind::" in line]


def idioms_evidenced_unlinted(root: Path) -> list[str]:
    """Spots a real program shows to be non-idiomatic that no lint teaches: each
    `// expect: missing <pattern>` in corpus/newcomer, and each `non-idiomatic` finding
    of an acceptance run that corpus/newcomer/acceptance.txt leaves out, marks
    `missing:`, or maps to a lint that doesn't exist
    (handbook/guidelines/lints-checklist.md, lints/R26)."""
    out = [f"{_rel(root, p)}:{i} {m.group(1)}" for p in sorted((root / "corpus/newcomer").glob("*.cp"))
           for i, line in enumerate(_lines(p), 1)
           for m in re.finditer(r"missing ([\w-]+)", line.split("// expect:", 1)[1] if "// expect:" in line else "")]
    ledger = root / "corpus/newcomer/acceptance.txt"
    fate = {}
    if ledger.is_file():
        for line in _lines(ledger):
            if line.strip() and not line.startswith("#"):
                id_, _, rest = line.partition(" ")
                fate[id_] = rest.strip()
    for run in sorted((root / "research/acceptance/runs").glob("*.json")):
        for f in json.loads(run.read_text(encoding="utf-8")).get("findings", []):
            if f.get("kind") != "non-idiomatic":
                continue
            d = fate.get(f["id"], "")
            lint = d[len("lint:"):] if d.startswith("lint:") else None
            known = lint and ((root / "spec/lints" / lint).is_dir() or (root / "spec/checker-lints" / lint).is_dir())
            if not (known or d.startswith("not-lint:")):
                out.append(f"{_rel(root, run)} {f['id']} {d or 'no entry in corpus/newcomer/acceptance.txt'}")
    return sorted(set(out))


def backend_std_names(root: Path) -> list[str]:
    subdirs = [f"crates/{c}/src" for c in crate_rules(root).get("back end", [])]
    return [f"{_rel(root, p)}:{i} {m.group(1)}" for p in _rs_files(root, *subdirs, tests=False)
            for i, line in enumerate(_lines(p), 1) for m in STD_TYPE_RE.finditer(line)]


STD_PUB_NAME_RE = re.compile(r"^pub (?:fn|struct|enum|trait|type) (\w+)")
NAME_BAN_CRATES = ("cpl-check", "cpl-ir", "cpl-interp", "cpl-codegen")
# Modules outside name resolution, lowering and codegen: the build-option tables a CLI flag
# parses (`cpl build --checks all`), whose strings are flag values, not std names (decision 635).
NAME_BAN_OUT_OF_SCOPE = ("crates/cpl-ir/src/checks.rs",)


def std_names_outside_rt(root: Path) -> list[str]:
    """Public std names written as string literals in compiler source: the name-ban gate
    (architecture.md, decision 573). Only cpl-rt may name a primitive."""
    std = sorted(_old_std(root).glob("*.cp")) if (root / FROZEN_STD).is_dir() else [
        root / rel for rel in _git(root, "ls-files", "std/*.cp")]
    names = {m.group(1) for path in std for line in _lines(path) if (m := STD_PUB_NAME_RE.match(line))}
    hits: set[str] = set()
    for p in _rs_files(root, *(f"crates/{c}/src" for c in NAME_BAN_CRATES), tests=False):
        if "test" in p.name or _rel(root, p) in NAME_BAN_OUT_OF_SCOPE:
            continue
        for line in _lines(p):
            if not line.lstrip().startswith("//"):
                hits.update(n for n in re.findall(r'"(\w+)"', line) if n in names)
    return sorted(hits)


def domain_member_names(root: Path) -> list[str]:
    """Domain members std declares with `@member(name)` written as string literals in
    compiler source: lowering and checking read them off std, not a name match."""
    names = {m for p in sorted((_old_std(root)).glob("*.cp"))
             for m in re.findall(r"@member\((\w+)\)", p.read_text())}
    hits: list[str] = []
    for p in _rs_files(root, *(f"crates/{c}/src" for c in NAME_BAN_CRATES), tests=False):
        if "test" in p.name:
            continue
        for i, line in enumerate(_lines(p), 1):
            if not line.lstrip().startswith("//"):
                hits.extend(f"{_rel(root, p)}:{i} {n}" for n in re.findall(r'"(\w+)"', line) if n in names)
    return hits


@functools.lru_cache(maxsize=4)
def _fn_bodies(root: Path) -> tuple:
    return tuple(_walk_fn_bodies(root))


def _walk_fn_bodies(root: Path):
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


CP_PUB_FN = re.compile(r"^(\s*)pub fn (\w+)")
CP_FN = re.compile(r"^(\s*)(?:pub )?fn (\w+)")
CP_DUP_MIN_CHARS = 25
# Renamed copies: a std fn of at least this many tokens whose body, its lowercase
# identifiers renamed in order of first use, is NEAR_DUP_MIN alike another module's.
CP_SHAPE_MIN_TOKENS = 50
CP_KEYWORDS = frozenset("let var if elif else for in while return yield and or not fn match "
                        "unchecked assert break continue true false self mut".split())


def _cp_fn_bodies(root: Path, private: bool = False):
    """(rel, line, name, body lines) for every `pub fn` in std/*.cp (every `fn` when
    `private`): the signature ends at its depth-0 `:` (a block of deeper lines follows) or
    `=` (the expression after it); `//` lines are dropped."""
    fn_re = CP_FN if private else CP_PUB_FN
    for rel in _git(root, "ls-files", "std/**.cp", "std/*.cp"):
        src = _lines(root / rel)
        i = 0
        while i < len(src):
            m = fn_re.match(src[i])
            if not m:
                i += 1
                continue
            indent, depth, j, k, body = len(m.group(1)), 0, i, m.end(2) - len(m.group(2)) - 1, None
            while j < len(src) and body is None:
                code = src[j].split("//")[0]
                while k < len(code):
                    c = code[k]
                    if c in "([{":
                        depth += 1
                    elif c in ")]}":
                        depth -= 1
                    elif depth == 0 and c == "=" and code[k - 1] not in "=!<>-" and code[k + 1:k + 2] not in ("=", ">"):
                        body = [code[k + 1:].strip()]
                        break
                    elif depth == 0 and c == ":" and not code[k + 1:].strip():
                        body = []
                        break
                    k += 1
                if body is None:
                    j, k = j + 1, 0
            if body is None:
                break
            e = j + 1
            while e < len(src) and (not src[e].strip() or len(src[e]) - len(src[e].lstrip()) > indent):
                s = src[e].strip()
                if s and not s.startswith("//"):
                    body.append(s)
                e += 1
            yield rel, i + 1, m.group(2), [b for b in body if b]
            i = e


def _cp_dup_bodies(root: Path) -> list[str]:
    """Each extra std `pub fn` whose body (> CP_DUP_MIN_CHARS) another-named std fn has."""
    groups = defaultdict(list)
    for rel, ln, name, body in _cp_fn_bodies(root):
        text = "\n".join(body)
        if len(text) > CP_DUP_MIN_CHARS:
            groups[text].append((rel, ln, name))
    return [f"{rel}:{ln} {name} = {g[0][0]}:{g[0][1]} {g[0][2]}"
            for g in groups.values() if len({x[2] for x in g}) > 1 for rel, ln, name in g[1:]]


def _cp_near_dups(root: Path) -> list[str]:
    """std `pub fn` pairs `name` and `name_real`/`name_excl`: one algorithm written per variant."""
    where = {}
    for rel, ln, name, _ in _cp_fn_bodies(root):
        where.setdefault(name, f"{rel}:{ln}")
    return [f"{where[n + s]} {n + s} ~ {where[n]} {n}" for n in sorted(where)
            for s in ("_real", "_excl") if n + s in where]


def _cp_shape(body: list[str]) -> tuple[int, set[int]]:
    """A body's token count and its 4-token shingles once numbers fold to `0` and each
    lowercase identifier becomes `v<n>` by first use, so a copy with its names changed
    has the same shape."""
    names: dict[str, str] = {}
    toks = []
    for t in TOKEN_RE.findall(" ".join(body)):
        if t[0].isdigit():
            t = "0"
        elif (t[0].isalpha() and t[0].islower() or t[0] == "_") and t not in CP_KEYWORDS:
            t = names.setdefault(t, f"v{len(names)}")
        toks.append(t)
    return len(toks), {zlib.crc32(" ".join(toks[i : i + 4]).encode()) for i in range(len(toks) - 3)}


def _cp_renamed_dups(root: Path) -> list[str]:
    """Each pair of std fns (private ones too) in different modules with the same body shape
    whatever their names (`_cp_shape`, similarity >= NEAR_DUP_MIN): one algorithm written
    twice, where one module should call the other's."""
    fns = [(rel, ln, name, shape) for rel, ln, name, body in _cp_fn_bodies(root, private=True)
           for n, shape in [_cp_shape(body)] if n >= CP_SHAPE_MIN_TOKENS]
    out = []
    for k, (ra, la, na, sa) in enumerate(fns):
        for rb, lb, nb, sb in fns[k + 1 :]:
            if ra == rb or min(len(sa), len(sb)) < NEAR_DUP_MIN * max(len(sa), len(sb)):
                continue
            sim = len(sa & sb) / len(sa | sb)
            if sim >= NEAR_DUP_MIN:
                out.append((-sim, f"{rb}:{lb} {nb} ~ {ra}:{la} {na} ({sim:.2f})"))
    return [x for _, x in sorted(out)]


def dup_fn_bodies(root: Path) -> list[str]:
    """Each extra copy of an identical fn body (>= DUP_MIN_LINES) in another file, and of a
    std `pub fn` body under another name (Content)."""
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
    return out + _cp_dup_bodies(root)


def near_dup_fns(root: Path) -> list[str]:
    """Each pair of near-identical fns (>= DUP_MIN_LINES, similarity >= NEAR_DUP_MIN) in different
    crates, each std `name`/`name_real`/`name_excl` pair, and each pair of std fns in different
    modules with one body shape under any names (Content)."""
    fns = []
    for p, ln, name, body in _fn_bodies(root):
        if len(body) < DUP_MIN_LINES:
            continue
        toks = ["0" if t[0].isdigit() else t for t in TOKEN_RE.findall(" ".join(body))]
        shingles = {zlib.crc32(" ".join(toks[i : i + 4]).encode()) for i in range(len(toks) - 3)}
        rel = _rel(root, p)
        fns.append((rel, ln, name, rel.split("/")[1], shingles, "\n".join(body)))
    df: dict[int, int] = defaultdict(int)
    for f in fns:
        for s in f[4]:
            df[s] += 1
    # Candidates share one of each other's 8 smallest uncommon shingles (a bottom-k sketch).
    buckets = defaultdict(list)
    for i, f in enumerate(fns):
        for s in sorted(x for x in f[4] if df[x] <= NEAR_DUP_COMMON)[:8]:
            buckets[s].append(i)
    pairs = {(a, b) for ids in buckets.values() for k, a in enumerate(ids) for b in ids[k + 1 :]
             if fns[a][3] != fns[b][3]}
    out = []
    for a, b in pairs:
        (pa, la, na, _, sa, ba), (pb, lb, nb, _, sb, bb) = fns[a], fns[b]
        if ba == bb or min(len(sa), len(sb)) < NEAR_DUP_MIN * max(len(sa), len(sb)):
            continue
        sim = len(sa & sb) / len(sa | sb)
        if sim >= NEAR_DUP_MIN:
            out.append((-sim, f"{pb}:{lb} {nb} ~ {pa}:{la} {na} ({sim:.2f})"))
    return [x for _, x in sorted(out)] + _cp_near_dups(root) + _cp_renamed_dups(root)


def consumer_rederived(root: Path) -> list[str]:
    """Sites where a tool recomputes what cpl-check provides: text compared against a
    type name, or a hard-coded checker error code. Own databases count under own_databases."""
    types = set(CORE_TYPES)
    for p in sorted((root / "std").glob("*.cp")):
        types.update(re.findall(r"^\s*pub\s+(?:struct|enum|trait|type)\s+(\w+)", p.read_text(), re.M))
    out = []
    for crate in crate_rules(root).get("consumer", []):
        for p in _rs_files(root, f"crates/{crate}/src", tests=False):
            rel = _rel(root, p)
            if rel.endswith("_tests.rs"):
                continue
            for i, line in enumerate(_lines(p), 1):
                code = line.strip()
                if code.startswith("#[cfg(test)]"):
                    break
                if code.startswith("//"):
                    continue
                out += [f"{rel}:{i} type {a or b}" for a, b in TYPE_CMP_RE.findall(code) if (a or b) in types]
                out += [f"{rel}:{i} code {c}" for c in ECODE_RE.findall(code)]
    return out + site_rederived(root, types)


WEB_SOURCE_RE = re.compile(r"^web/(?!public/|tests/|content/generated/).*(?<!\.test)\.(?:ts|tsx|mjs|js)$")
WEB_TYPE_CMP_RE = re.compile(r"""[=!]==?\s*['"]([A-Z]\w*)['"]""")
WEB_ECODE_RE = re.compile(r"""['"`](E\d{4})\b""")
WEB_CAPABILITY_RE = re.compile(r"""^\s*['"]?([a-z]\w*Provider)['"]?\s*:""")
WEB_LINT_SOURCE_RE = re.compile(r"spec/(?:checker-)?lints\b|lint\.toml")


def site_rederived(root: Path, types: set[str] | None = None) -> list[str]:
    """web/ sources that re-derive tool data instead of asking the tool: text compared
    against a type name, a checker error code, a hand table of language-server
    capabilities, or a read of the lint TOML that `cpl lint --list --json` serves."""
    if types is None:
        types = set(CORE_TYPES)
        for p in sorted((root / "std").glob("*.cp")):
            types.update(re.findall(r"^\s*pub\s+(?:struct|enum|trait|type)\s+(\w+)", p.read_text(), re.M))
    out = []
    for rel in _git(root, "ls-files", "web"):
        if not WEB_SOURCE_RE.match(rel) or not (root / rel).is_file():
            continue
        for i, line in enumerate(_lines(root / rel), 1):
            code = line.strip()
            if code.startswith(("//", "*", "/*")):
                continue
            out += [f"{rel}:{i} type {t}" for t in WEB_TYPE_CMP_RE.findall(code) if t in types]
            out += [f"{rel}:{i} code {c}" for c in WEB_ECODE_RE.findall(code)]
            out += [f"{rel}:{i} capability {c}" for c in WEB_CAPABILITY_RE.findall(line)]
            out += [f"{rel}:{i} lint source"] if WEB_LINT_SOURCE_RE.search(code) else []
    return out


LSP_NESTED_KEYS = {"prepareProvider", "resolveProvider"}
LSP_METHOD_RE = re.compile(r'"((?:textDocument|workspace)/\w+)')
# The LSP spec's methods whose capability isn't `<method>Provider`.
LSP_IRREGULAR = {
    "textDocument/formatting": "documentFormatting",
    "textDocument/rangeFormatting": "documentRangeFormatting",
    "textDocument/onTypeFormatting": "documentOnTypeFormatting",
    "textDocument/prepareCallHierarchy": "callHierarchy",
    "workspace/symbol": "workspaceSymbol",
}


def lsp_capability(method: str) -> str:
    return LSP_IRREGULAR.get(method, method.partition("/")[2]) + "Provider"


def browser_lsp_gaps(root: Path) -> list[str]:
    """Capabilities cpl-lsp advertises (`capabilities()` in server.rs) that the browser
    editor (web/components/editor/) never asks for: `textDocument/fooBar` asks for
    `fooBarProvider`, but for the spec's few irregular names."""
    try:
        server = (root / "crates/cpl-lsp/src/server.rs").read_text(encoding="utf-8")
    except OSError:
        return []
    editor = "".join(p.read_text(encoding="utf-8")
                     for p in sorted((root / "web/components/editor").glob("*.ts"))
                     if not p.name.endswith(".test.ts"))
    body = server.partition("\nfn capabilities()")[2].partition("\n}\n")[0]
    advertised = set(re.findall(r'"([a-z]\w*Provider)"', body)) - LSP_NESTED_KEYS
    used = {lsp_capability(m) for m in LSP_METHOD_RE.findall(editor)}
    return sorted(advertised - used)


def site_spec_snippets(root: Path) -> list[str]:
    """Site programs the compiler doesn't accept yet: `*.spec.cp` files and `status=spec`
    fences under web/content (web/scripts/gate.mjs). Each is a checker or std bug to fix."""
    out = []
    for rel in _git(root, "ls-files", "web/content"):
        if rel.endswith(".spec.cp"):
            out.append(rel)
        elif rel.endswith((".md", ".mdx")) and (root / rel).is_file():
            out += [f"{rel}:{i}" for i, line in enumerate(_lines(root / rel), 1)
                    if re.match(r"\s*```cp\b.*\bstatus=spec\b", line)]
    return out


def _crate_edges(root: Path, kind: str = "dependencies") -> set[tuple[str, str]]:
    edges = set()
    for toml in sorted((root / "crates").glob("*/Cargo.toml")):
        data = tomllib.loads(toml.read_text(encoding="utf-8"))
        me = data.get("package", {}).get("name", toml.parent.name)
        tables = [data.get(kind, {})] + [t.get(kind, {}) for t in data.get("target", {}).values()]
        edges |= {(me, dep) for t in tables for dep in t if dep.startswith("cpl-") and dep != me}
    return edges


def a4_edges(root: Path) -> list[str]:
    """Edges handbook/design/architecture.md (Crates) documents: a crate's row naming another crate."""
    try:
        text = (root / "handbook/design/architecture.md").read_text(encoding="utf-8")
    except OSError:
        return []
    section = re.split(r"^## Crates\n", text, maxsplit=1, flags=re.M)[-1].partition("\n## ")[0] if re.search(
        r"^## Crates\n", text, re.M) else ""
    edges = set()
    for line in section.splitlines():
        cells = line.split("|")
        if len(cells) < 4:
            continue
        crates = re.findall(r"`(cpl-[\w-]+)`", cells[1])
        uses = re.findall(r"\b(cpl-[a-z]+(?:-[a-z]+)*)\b", "|".join(cells[2:]))
        edges |= {f"{a} -> {b}" for a in crates for b in uses if a != b}
    return sorted(edges)


def crate_edges(root: Path) -> list[str]:
    return [f"{a} -> {b}" for a, b in sorted(_crate_edges(root))]


def dev_crate_edges(root: Path) -> list[str]:
    """Test-only edges (informational)."""
    dev = _crate_edges(root, "dev-dependencies") - _crate_edges(root)
    return [f"{a} -> {b}" for a, b in sorted(dev)]


def layer_violations(root: Path) -> list[str]:
    return [f"{a} -> {b}" for a, b in sorted(_crate_edges(root) & forbidden_edges(root))]


TOOL_COMPILER_KINDS = ("dependencies", "dev-dependencies", "build-dependencies")


def tool_compiler_deps(root: Path) -> list[str]:
    """Ways a tool crate reaches a compiler crate without the facade: every Cargo edge
    (shipped, dev or build) from a `tool` crate to a `compiler` crate, and every code line
    in a tool crate that takes a path into one or imports it (`cpl_check::`, `use cpl_ir`,
    ...). Both
    lists are the `tool` and `compiler` rows of architecture.md's crate rules. The target is
    0 (handbook/design/next/contracts/09-facade.md, migration stage 2): a tool runs and
    checks a program through the facade, over a snapshot, and through nothing else.

    Not counted: `cpl-syntax`. Lints match on today's syntax tree about 940 times, and the
    formatter and the editor read it too; that access moves in migration stage 8, onto the
    lossless tree, not here. Not a tool: `cpl-conform` (the test harness, which may compare
    engines, migration rule 6) and `cpl-next` (the pipeline behind the facade)."""
    rules = crate_rules(root)
    tools, compiler = rules.get("tool", []), set(rules.get("compiler", []))
    if not tools or not compiler:
        return []
    edges = set()
    for kind in TOOL_COMPILER_KINDS:
        edges |= {(a, b) for a, b in _crate_edges(root, kind) if a in tools and b in compiler}
    out = [f"{a} -> {b}" for a, b in sorted(edges)]
    crates = "|".join(sorted(c.replace("-", "_") for c in compiler))
    # A path into the crate or an import of it. A bare word is not a reach: the browser
    # ABI's export `fn cpl_check()` shares the crate's name and names nothing in it.
    named = re.compile(rf"\b(?:{crates})\s*::|\b(?:use|extern\s+crate)\s+(?:{crates})\b")
    for crate in tools:
        for p in _rs_files(root, f"crates/{crate}"):
            rel = _rel(root, p)
            out += [f"{rel}:{i}" for i, line in enumerate(_lines(p), 1)
                    if not line.lstrip().startswith("//") and named.search(line)]
    return out


def facade_reexports(root: Path) -> set[str]:
    """The names `cpl-facade` hands on from a compiler crate outside its `today` module:
    each `pub use cpl_interp::{Limits, Stop}` (under its `as` name, when it has one) and
    each `pub type Size = cpl_codegen::Size`."""
    crates = "|".join(c.replace("-", "_") for c in crate_rules(root).get("compiler", []))
    if not crates:
        return set()
    used = re.compile(rf"\bpub\s+use\s+(?:{crates})\b([^;]*);")
    aliased = re.compile(rf"\bpub\s+type\s+(\w+)[^=;]*=\s*(?:{crates})\b")
    names: set[str] = set()
    for p in _rs_files(root, "crates/cpl-facade"):
        if p.name == "today.rs":
            continue
        src = "\n".join(l for l in _lines(p) if not l.lstrip().startswith("//"))
        names |= set(aliased.findall(src))
        for tree in used.findall(src):
            names |= {leaf.split()[-1].rsplit("::", 1)[-1] for leaf in re.split(r"[{},]", tree) if leaf.strip()}
    return {n for n in names if n.isidentifier() and n != "self"}


def facade_today_uses(root: Path) -> list[str]:
    """Code lines in a tool crate that read one of the compiler's own forms through the
    facade: a line naming `cpl_facade::today` (the checked file, its types and hovers, a
    run's recording), or one naming an item the facade re-exports from a compiler crate
    under its own root (`facade_reexports`: `Recording`, `Limits`, `Stop`, `Trap`, ...) in
    a file that takes that name from `cpl_facade`. They pass `tool_compiler_deps`, since the
    path goes through the facade, but each is a form that changes when the compiler behind
    the facade does. The target is 0: each becomes a query or a type of contract 9
    (handbook/design/next/contracts/09-facade.md), and the module and the re-exports go."""
    tools = crate_rules(root).get("tool", [])
    handed = facade_reexports(root)
    named = re.compile(r"\btoday\s*::|::\s*today\b|\buse\s+today\b")
    imports = re.compile(r"\buse\s+cpl_facade\s*::([^;]*);")
    by_path = re.compile(r"\bcpl_facade\s*::\s*(\w+)")
    out = []
    for crate in tools:
        for p in _rs_files(root, f"crates/{crate}"):
            rel = _rel(root, p)
            code = [("" if l.lstrip().startswith("//") else l) for l in _lines(p)]
            taken = handed & {w for tree in imports.findall("\n".join(code)) for w in re.findall(r"\w+", tree)}
            words = re.compile(r"\b(?:" + "|".join(sorted(taken)) + r")\b") if taken else None
            out += [f"{rel}:{i}" for i, line in enumerate(code, 1)
                    if named.search(line) or (words and words.search(line))
                    or any(n in handed for n in by_path.findall(line))]
    return out


def tree_walker(root: Path) -> list[str]:
    """The interpreter's tree-walking engine beside the VM."""
    lib = root / "crates/cpl-interp/src/lib.rs"
    src = lib.read_text(encoding="utf-8") if lib.is_file() else ""
    hit = re.search(r"fn eval\(&mut self, e: &'p Expr\)", src)
    return ["crates/cpl-interp/src/lib.rs: fn eval(&mut self, e: &'p Expr)"] if hit else []


def v02_port_files(root: Path) -> list[str]:
    """Corpus ports not yet re-ported in v0.3 style (absent from `corpus/sets/v03.txt`).

    Their frozen copies under `tests/v02-snapshot/` keep the regression coverage."""
    sets = root / "corpus/sets/v03.txt"
    v03 = {l.strip() for l in _lines(sets)} if sets.is_file() else set()
    return [p for p in _git(root, "ls-files", "corpus/problems/*/port.cp") if p.split("/")[2] not in v03]


def ports_unreported(root: Path) -> list[str]:
    """Corpus ports landed without their friction report or review
    (`research/frictions/<id>.md`, `research/reviews/<id>.md`; porting.md steps 4-5)."""
    have = set(_git(root, "ls-files", "research/frictions/*.md", "research/reviews/*.md"))
    return [p for p in _git(root, "ls-files", "corpus/problems/*/port.cp")
            if not {f"research/frictions/{p.split('/')[2]}.md", f"research/reviews/{p.split('/')[2]}.md"} <= have]


def reviews_uncapped(root: Path) -> list[str]:
    """Reviews scoring a port 4 or 5 when it has no dream, neither dream.cp nor a `## Dream` in
    its friction report: research/reviews/README.md caps a port with no dream at 3."""
    dreams = {f.split("/")[2] for f in _git(root, "ls-files", "corpus/problems/*/dream.cp")}
    dreams |= {Path(f).stem for f in _git(root, "ls-files", "research/frictions/*.md")
               if any(l.startswith("## Dream") for l in _lines(root / f))}
    ports = {f.split("/")[2] for f in _git(root, "ls-files", "corpus/problems/*/port.cp")}
    out = []
    for f in _git(root, "ls-files", "research/reviews/*.md"):
        name = Path(f).stem
        score = next((l for l in _lines(root / f) if l.startswith("Score:")), "")
        if name in ports and name not in dreams and re.match(r"Score:\s*[45]\s*/\s*5", score):
            out.append(f)
    return out


def reviews_pending(root: Path) -> list[str]:
    """Taste reviews whose port score is still `Score: pending` (research/reviews/README.md):
    a port lands with its cold review scored, so an unscored review is debt, not a state."""
    return [f for f in _git(root, "ls-files", "research/reviews/*.md")
            if any(l.strip() == "Score: pending" for l in _lines(root / f))]

def reviews_orphan(root: Path) -> list[str]:
    """Taste reviews with nothing to review: no `corpus/problems/<id>/port.cp` and no `dream.cp`
    (a dream-judged package's dream is its port). A deleted package takes its review with it."""
    judged = {p.split("/")[2] for p in _git(root, "ls-files", "corpus/problems/*/port.cp", "corpus/problems/*/dream.cp")}
    return [f for f in _git(root, "ls-files", "research/reviews/*.md")
            if f != "research/reviews/README.md" and Path(f).stem not in judged]

def spec_std_refs(root: Path) -> list[str]:
    """`spec/std` mentions outside history (the stdlib moved to `std/`; decision 307)."""
    return _git(root, "grep", "-n", "spec/std", "--", ".", ":!research", ":!decisions.md",
                ":!plan/domains/tech-debt", ":!scripts/debt-ratchet.py")


def direct_python(root: Path) -> list[str]:
    """`python3 ...` calls in the justfile, the scripts' shell and the workflows (target:
    `uv run`, the one interpreter with the locked dependencies)."""
    out = []
    for f in ["justfile", *_git(root, "ls-files", "scripts/*.sh", "scripts/**/*.sh", ".github/*.yml", ".github/**/*.yml")]:
        p = root / f
        for i, line in enumerate(_lines(p) if p.is_file() else [], 1):
            if re.search(r"(^|[\s(])python3 ", line) and not line.lstrip().startswith("#"):
                out.append(f"{f}:{i}")
    return out


def unrun_script_tests(root: Path) -> list[str]:
    """Tracked `test_*.py` files the gate neither names (check.sh, ci.py) nor globs in
    ci.py's SCRIPT_TESTS."""
    text = "".join(f.read_text() for f in (root / "scripts/check.sh", root / "scripts/ci.py") if f.is_file())
    m = re.search(r"^SCRIPT_TESTS = \((.*)\)$", text, re.M)
    globs = re.findall(r'"([^"]+)"', m[1]) if m else []
    return sorted(rel for rel in _git(root, "ls-files", "*test_*.py")
                  if Path(rel).name.startswith("test_") and Path(rel).name not in text
                  and not any(fnmatch.fnmatchcase(rel, g) for g in globs))


def unreferenced_scripts(root: Path) -> list[str]:
    """Tracked files under scripts/ that nothing else names, so no recipe, workflow, script,
    doc or import reaches them: dead code to delete. A script named only by its own test is
    dead too. A package module is named by `<pkg>.<module>` or, as a spec gate is, by its
    dashed name in quotes; a package's `__init__.py` lives while any of its modules does."""
    files = [f for f in _git(root, "ls-files", "scripts") if not Path(f).name.startswith("test_")]
    pats: dict[str, set[str]] = {}
    for f in files:
        p = Path(f)
        names = {p.name}
        if (root / p.parent / "__init__.py").is_file() and p.parent.name != "scripts":
            dash = p.stem.replace("_", "-")
            names |= {f"{p.parent.name}.{p.stem}", f'"{dash}"', f"'{dash}'"}
        pats[f] = names
    args = [a for n in sorted(set().union(*pats.values())) for a in ("-e", n)] if pats else []
    hits: dict[str, set[str]] = {}
    for line in _git(root, "grep", "-o", "-w", "-F", *args, "--", ".", ":!corpus") if args else []:
        src, _, m = line.partition(":")
        hits.setdefault(m, set()).add(src)

    def named(f: str) -> bool:
        own_test = f"{Path(f).parent}/test_{Path(f).stem.replace('-', '_')}.py"
        return any(src not in (f, own_test) for n in pats[f] for src in hits.get(n, ()))

    live = {f for f in files if Path(f).name != "__init__.py" and named(f)}
    return sorted(f for f in files if f not in live and not (
        Path(f).name == "__init__.py" and any(Path(g).parent == Path(f).parent for g in live)))


_RUST_TEST = re.compile(r"#\[test\]\s*((?:#\[[^\]]*\]\s*)*)(?:pub\s+)?(?:async\s+)?fn\s+(\w+)")


def tests_unrun(root: Path) -> list[str]:
    """Workspace #[test]s no CI tier runs: ignored ones, every one when ci.yml's
    hash:i/N or time:i/N partitions miss a part, a SLOW_TESTS one no ci.py step names (by name
    or its whole `--test` binary), and
    a wall-clock one when ci-slow.yml has no timing job."""
    ci = root / "scripts/ci.py"
    src = ci.read_text() if ci.is_file() else ""
    lists: dict[str, tuple[str, ...]] = {}
    for node in ast.parse(src).body if src else []:
        if isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name) \
                and node.targets[0].id in ("SLOW_TESTS", "WALL_CLOCK_TESTS"):
            lists[node.targets[0].id] = tuple(ast.literal_eval(node.value))
            src = src.replace(ast.get_source_segment(src, node) or "", "")
    yml = (root / ".github/workflows/ci.yml")
    slow_yml = (root / ".github/workflows/ci-slow.yml")
    main_ci = yml.read_text() if yml.is_file() else ""
    slow_ci = slow_yml.read_text() if slow_yml.is_file() else ""
    parts = {(int(i), int(n)) for i, n in re.findall(r"(?:hash|time):(\d+)/(\d+)", main_ci)}
    totals = {n for _, n in parts}
    partitioned = len(totals) == 1 and {i for i, _ in parts} == set(range(1, max(totals) + 1))
    full = "just check-full" in slow_ci
    timing = "just check-timing" in slow_ci
    slow = set(lists.get("SLOW_TESTS", ()))
    wall = set(lists.get("WALL_CLOCK_TESTS", ()))
    out = []
    for rel in _git(root, "ls-files", "crates/*.rs"):
        text = (root / rel).read_text(errors="replace")
        for m in _RUST_TEST.finditer(text):
            attrs, name = m[1], m[2]
            where = f"{rel}::{name}"
            if "#[ignore" in attrs:
                out.append(f"{where} (ignored)")
            elif name in wall:
                if not timing:
                    out.append(f"{where} (wall-clock, no timing job)")
            elif name in slow:
                binary = f'"--test", "{Path(rel).stem}"' if "/tests/" in rel else None
                if not (f'"{name}"' in src and full or binary and binary in src):
                    out.append(f"{where} (SLOW_TESTS, no --full step)")
            elif not partitioned:
                out.append(f"{where} (ci.yml's hash partitions miss a part)")
    return sorted(out)


def last_touched(root: Path, rel: str, n: int = 3) -> list[str]:
    """The last `n` commits to change `rel`, minus a shallow clone's boundary
    commit: it holds every file without having changed them, so naming it
    would blame whoever pushed the checked-out tip."""
    shallow = _git(root, "rev-parse", "--git-path", "shallow")
    path = Path(shallow[0]) if shallow else None
    if path is not None and not path.is_absolute():
        path = root / path
    graft = set(path.read_text().split()) if path is not None and path.is_file() else set()
    by = [l.split(" ", 1) for l in _git(root, "log", f"-{n}", "--format=%H %h %s", "--", rel)]
    return [rest for full, rest in by if full not in graft]


def _git_env() -> dict[str, str]:
    """The environment without GIT_*: a hook's GIT_DIR must not redirect `git -C`."""
    return {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}


def _git_ignored(root: Path, paths: list[str]) -> set[str]:
    """Paths `.gitignore` covers: build output, not a stale citation."""
    if not paths:
        return set()
    try:
        r = subprocess.run(["git", "-C", str(root), "check-ignore", "--no-index", "--stdin"],
                           input="\n".join(q for p in paths for q in (p, p + "/")),
                           capture_output=True, text=True, env=_git_env())
    except OSError:
        return set()
    return {p.rstrip("/") for p in r.stdout.split()} if r.returncode in (0, 1) else set()


def decision_dup_numbers(root: Path) -> list[str]:
    """Decision numbers used by more than one `## N.` heading in decisions.md."""
    path = root / "decisions.md"
    nums = re.findall(r"^##\s+(\d+)\.", path.read_text() if path.exists() else "", re.M)
    return sorted({n for n in nums if nums.count(n) > 1}, key=int)


def lint_dup_helpers(root: Path) -> list[str]:
    """Helper fns defined in more than one lint rule file (each rule's `check` excepted)."""
    where = defaultdict(list)
    for p in _rs_files(root, "crates/cpl-lint/src/rules"):
        for name in sorted({m.group(2) for m in map(FN_RE.match, _lines(p)) if m and not m.group(1)}):
            where[name].append(_rel(root, p))
    return [f"{n}: {', '.join(fs)}" for n, fs in sorted(where.items()) if len(fs) > 1 and n != "check"]


AUTHORITY = "handbook leads design; spec/ is its downstream exhaustive form"
AUTHORITY_HOMES = ("AGENTS.md", "CHANGELOG.md", "spec/README.md", "handbook/README.md",
                   "handbook/language/overview.md", "handbook/domains/spec.md", "plan/STATUS.md")
AUTHORITY_CONTRARY = re.compile(
    r"only normative source|normative source is|the normative language|spec[^.]{0,40}wins wherever"
    r"|where `?spec/`? disagrees with any other doc", re.I)


def authority_drift(root: Path) -> list[str]:
    """A home of the authority sentence without it, or a doc stating another order.

    Rule: handbook/README.md (Language design and spec move together), as
    Ryan decided. Prose wraps, so the sentence is matched across line breaks.
    """
    found = []
    for f in AUTHORITY_HOMES:
        if not (root / f).is_file():
            continue
        text = (root / f).read_text(encoding="utf-8")
        if AUTHORITY.lower() not in " ".join(text.replace("`", "").split()).lower():
            found.append(f"{f}: lacks \"{AUTHORITY}\"")
    for f in _git(root, "ls-files", "*.md", "*.mdx", "*.tsx", ":!research", ":!web/content/docs/releases.mdx"):
        if (root / f).is_file():
            text = " ".join((root / f).read_text(encoding="utf-8").split())
            for m in AUTHORITY_CONTRARY.finditer(text):
                found.append(f"{f}: {m.group(0)}")
    return found


WHY_HOMES = ("decisions.md", "handbook/language/*.md", "handbook/design/*.md",
             "spec/*.md", "spec/core/*.md", "spec/lib/*.md", "spec/tools/*.md")


def _sentences(text: str) -> list[str]:
    """Normalized prose sentences of 15+ words, outside code, tables, headings and quotes."""
    text = re.sub(r"```.*?```", "", text, flags=re.S)
    prose = " ".join(l for l in text.split("\n") if not l.lstrip().startswith(("|", "#", ">")))
    out = []
    for s in re.split(r"(?<=[.!?])\s+", prose):
        t = " ".join(re.sub(r"[*`]", "", s).lower().split())
        if len(t.split()) >= 15:
            out.append(t)
    return out


def why_written_twice(root: Path) -> list[str]:
    """A sentence of reasoning written in two homes, or a rationale file under spec/.

    Rule: handbook/design/sources-of-truth.md#never-written-twice. The why
    lives in handbook/language/ (and handbook/design/), decisions.md keeps the
    call and how to reverse it, spec/ states the rule: a sentence copied
    between two of them goes stale in one.
    """
    seen: dict[str, list[str]] = {}
    for f in _git(root, "ls-files", *WHY_HOMES):
        if (root / f).is_file():
            for s in set(_sentences((root / f).read_text(encoding="utf-8"))):
                seen.setdefault(s, []).append(f)
    found = [f"{' = '.join(sorted(fs))}: {s[:60]}" for s, fs in sorted(seen.items()) if len(fs) > 1]
    found += [f"{f}: rationale under spec/" for f in _git(root, "ls-files", "spec")
              if re.search(r"(^|/)rationale(/|\.md$)", f, re.I)]
    return found


SCOREBOARD = "research/scoreboard.json"


def _conformance_board(root: Path) -> dict | None:
    p = root / SCOREBOARD
    if not p.is_file():
        return None
    return json.loads(p.read_text()).get("suites", {}).get("conformance")


def conformance_failing(root: Path) -> list[str]:
    """Conformance cases research/scoreboard.json records as not passing.

    `cpl-conform conformance --write` writes the board; the ratchet only reads it, so
    land.sh never runs the suite. A missing board counts, so the gate can't pass blind."""
    board = _conformance_board(root)
    if board is None:
        return [f"{SCOREBOARD}: no conformance suite; run `cpl-conform conformance --write`"]
    return [f"{c}: {v}" for c, v in sorted(board.get("cases", {}).items()) if v != "pass"]


def interp_max_over_20s(root: Path) -> list[str]:
    """Corpus packages whose interpreter runs a tests.toml `--max` case past 20 s (owner: Core).

    `cpl-conform corpus --write` times each max case and records the slow packages in
    research/scoreboard.json (decision 556); a board without the list counts, so the gate
    can't pass blind."""
    p = root / SCOREBOARD
    board = json.loads(p.read_text()).get("suites", {}).get("corpus") if p.is_file() else None
    if board is None or "interp_max_over_20s" not in board:
        return [f"{SCOREBOARD}: no corpus max-case times; run `cpl-conform corpus --write`"]
    return list(board["interp_max_over_20s"])


CORPUS_MAX_SHORT = "research/corpus-max-short.json"


def _max_short_record(root: Path, key: str) -> list[str]:
    p = root / CORPUS_MAX_SHORT
    rec = json.loads(p.read_text()) if p.is_file() else {}
    if key not in rec:
        return [f"{CORPUS_MAX_SHORT}: no {key}; run `scripts/corpus.py max-short --write`"]
    return list(rec[key])


def corpus_max_short(root: Path) -> list[str]:
    """v03 ports whose gen.py --max reads a size under its Bounds (`max-short P n`, owner: the
    generator's lane). `scripts/corpus.py max-short --write` records them; ci-slow.yml fails a
    record that differs from what it measures, and a missing record counts."""
    return _max_short_record(root, "max_short")


def corpus_max_unread(root: Path) -> list[str]:
    """v03 ports whose --max sizes the max-short check can't read (no cut of the port, nor the
    port itself, reads them in time), so nothing shows they reach their Bounds (owner: Idioms)."""
    return _max_short_record(root, "max_unread")


DREAM_OVER_32KB = "research/dream-over-32kb.json"


def dream_over_32kb(root: Path) -> list[str]:
    """dream.cp files whose C++ for their judge is over 32 KB (goal 5; owner: Dreams).
    `scripts/corpus.py dream-sizes --write` records them; dreams.yml fails a record that
    differs from what it measures, and a missing record counts."""
    p = root / DREAM_OVER_32KB
    rec = json.loads(p.read_text()) if p.is_file() else {}
    if "dream_over_32kb" not in rec:
        return [f"{DREAM_OVER_32KB}: no dream_over_32kb; run `scripts/corpus.py dream-sizes --write`"]
    return list(rec["dream_over_32kb"])


TOOLS_TEST_TIMES = "research/perf/tools-test-times.json"


def tools_tests_over_30s(root: Path) -> list[str]:
    """Tests of the tools crates (cpl-cli, cpl-lsp, cpl-contest, cpl-stress, cpl-corpus)
    over 30 s (owner: Tools).

    `scripts/test-times.py --write` times them and records the slow ones in
    research/perf/tools-test-times.json; a missing record counts, so the gate can't
    pass blind."""
    p = root / TOOLS_TEST_TIMES
    if not p.is_file():
        return [f"{TOOLS_TEST_TIMES}: no test times; run `uv run scripts/test-times.py --write`"]
    return [f"{t}: {s} s" for t, s in json.loads(p.read_text())["over_limit"].items()]


def conformance_wrong_output(root: Path) -> list[str]:
    """Conformance cases research/scoreboard.json records as printing the wrong output.

    The board keeps five examples per cause, so the rest are counted by number."""
    board = _conformance_board(root)
    if board is None:
        return [f"{SCOREBOARD}: no conformance suite; run `cpl-conform conformance --write`"]
    for entry in board.get("causes", []):
        if entry.get("cause") == "wrong output":
            named = list(entry.get("examples", []))
            n = max(int(entry.get("count", 0)), len(named))
            return named + [f"wrong output {i + 1} of {n} (unnamed in {SCOREBOARD})" for i in range(len(named), n)]
    return []


def plan_delta_lines(root: Path) -> list[str]:
    """Lines of working notes under plan/domains/*/delta, plus each such directory on disk:
    findings become conformance cases, history goes under research/design-history/."""
    out = [f"{f}:{i}" for f in _git(root, "ls-files", "plan/domains/*/delta/*")
           for i, _ in enumerate(_lines(root / f), 1)]
    return out + [f"{d.relative_to(root)}/" for d in sorted(root.glob("plan/domains/*/delta"))]


def _code_lines(p: Path) -> int:
    """Lines that are neither blank nor a `//` comment: trimming comments can't game the cap."""
    return sum(1 for l in _lines(p) if l.strip() and not l.lstrip().startswith("//"))


def _near_cap(root: Path, max_rs_lines: int = DEFAULT_MAX_RS_LINES) -> dict[str, int]:
    out = {}
    for p in _rs_files(root, "crates"):
        n = _code_lines(p)
        if max_rs_lines - NEAR_CAP_LINES < n <= max_rs_lines:
            out[_rel(root, p)] = n
    return dict(sorted(out.items()))


def rs_near_cap(root: Path) -> list[str]:
    """`.rs` files within 100 lines under the cap: split them on a seam before they reach it.
    A file entering the band fails the check (see `evaluate`)."""
    return [f"{rel} {n}" for rel, n in _near_cap(root).items()]


# Every route from the VM into the tree-walker, as a source call site: the VM
# compiler's `Fallback*` emits, the VM's own `eval`/`stmt`/`block`/`feed`
# calls, and each place a whole body is left to the walker.
WALKER_ROUTE_RES = (
    ("crates/cpl-interp/src/vm/compile/", re.compile(r"(?<!fn )\bemit_fallback_(?:stmt|expr_i|expr_v)\(")),
    ("crates/cpl-interp/src/vm/", re.compile(r"\bself\.(?:eval|stmt|block|feed)\(")),
    ("crates/cpl-interp/src/", re.compile(r"\bself\.block\(&(?:program\.(?:main|setup)|f\.body|def\.body)\)")),
)


def vm_fallbacks(root: Path) -> list[str]:
    """Each source call site that routes a run from the VM into the tree-walker (decision 396's
    count, static): what deleting the walker has left. The runtime kinds over tests/programs are
    cpl-cli's trace_goldens list, `vm_fallbacks.txt`."""
    out = []
    for p in _rs_files(root, "crates/cpl-interp/src", tests=False):
        rel = _rel(root, p)
        prefix, rx = next((pre, rx) for pre, rx in WALKER_ROUTE_RES if rel.startswith(pre))
        out += [f"{rel}:{i}" for i, line in enumerate(_lines(p), 1)
                if rx.search(line) and not line.lstrip().startswith("//")]
    return out


OWN_DATABASE_RE = re.compile(r"\bDatabase::(?:new|with_loader|default)\b|\bDatabase\s*\{")


NEXT_TESTS = "crates/cpl-next/tests"


def next_wrong(root: Path) -> list[str]:
    """Cases the new pipeline (cpl-next) answers against their header: accepted, rejected or
    run to another result, a crash, or its three runs disagreeing. `tests/programs.rs` keeps
    `wrong.tsv` equal to what it finds, so this is the count, and it reaches 0 by fixes."""
    wrong = root / NEXT_TESTS / "wrong.tsv"
    return [l.split("\t")[0] for l in _lines(wrong) if l.strip()] if wrong.is_file() else []


def end_state_unmeasured(root: Path) -> list[str]:
    """Suites of the end state (`plan/end-state.toml`) that still say `missing`: nothing the
    gate measures counts them yet. One leaves by naming ratchet counters for all of it."""
    path = root / "plan" / "end-state.toml"
    if not path.is_file():
        return []
    suites = tomllib.loads(path.read_text(encoding="utf-8"))["suite"]
    return [f"suite {s['id']}: {s['name']}: {s['missing']}" for s in suites if s.get("missing")]


def end_state_broken(root: Path, ref: str = "origin/main") -> list[str]:
    """What `scripts/end-state.py --check` finds wrong with the end state: a malformed suite,
    a stale `plan/END-STATE.md`, a suite of `ref` that is gone, or one retargeted with no
    `changed` note. A target is then never weakened by an edit nobody had to write down."""
    script = root / "scripts" / "end-state.py"
    if not script.is_file() or not (root / "plan" / "end-state.toml").is_file():
        return []
    out = subprocess.run([sys.executable, str(script), "--check", "--against", ref], cwd=root,
                         capture_output=True, text=True, env=_git_env())
    return [l for l in (out.stdout + out.stderr).splitlines() if l.strip()] if out.returncode else []


def next_cases_removed(root: Path, ref: str = "origin/main") -> list[str]:
    """Cases of `ref` that are gone: a conformance case or a new-pipeline test deleted or
    renamed away. Deleting a wrong case would lower `next_wrong` for free; one leaves only
    by a line in `spec/conformance/REMOVED.md` naming it and why."""
    pats = ["spec/conformance", NEXT_TESTS + "/programs"]
    # From the fork point, so a case `ref` gained after it is not read as this tree deleting it.
    base = (_git(root, "merge-base", ref, "HEAD") or [ref])[0]
    gone = [l.split("\t")[1] for l in _git(root, "diff", "--name-status", "--diff-filter=DR", "--no-renames",
                                           base, "--", *pats) if l.endswith(".cp")]
    removed = root / "spec" / "conformance" / "REMOVED.md"
    named = removed.read_text(encoding="utf-8") if removed.is_file() else ""
    return [g for g in gone if f"`{g}`" not in named]


def next_masked(root: Path) -> list[str]:
    """Cases the new pipeline refuses although it also gave a coded verdict on the program:
    the refusal hides whether that verdict is right. `tests/programs.rs` keeps `masked.txt`
    equal to what it finds; it reaches 0 as refused constructs stop hiding verdicts beside them."""
    masked = root / NEXT_TESTS / "masked.txt"
    return [l.strip() for l in _lines(masked) if l.strip()] if masked.is_file() else []


def next_lowered(root: Path) -> list[str]:
    """Programs cases whose error only lowering finds: the checker passed what lowering then
    refuses, so the checker is not the one source of the program's errors (suite 5).
    `tests/programs.rs` fails on a case missing from `lowered.txt`."""
    listed = root / NEXT_TESTS / "lowered.txt"
    return [l.strip() for l in _lines(listed) if l.strip()] if listed.is_file() else []


OLD_PARSER_ACCEPTS = "tests/old-parser-accepts.txt"
OLD_PARSER_ACCEPTS_CODES = ("E0503", "E0504")


def _old_parser_accept_rows(root: Path) -> list[tuple[str, str]]:
    """(case, decision cited) per row of the list: `core-01/x.cp  # decision 662`."""
    listed = root / OLD_PARSER_ACCEPTS
    rows = []
    for line in _lines(listed) if listed.is_file() else []:
        case, _, note = line.partition("#")
        if case.strip():
            m = re.search(r"decision (\d+)", note)
            rows.append((case.strip(), m.group(1) if m else ""))
    return rows


def _old_parser_accepts(root: Path) -> list[str]:
    return [case for case, _ in _old_parser_accept_rows(root)]


def next_old_parser_accepts(root: Path) -> list[str]:
    """Rejected-form cases cpl-next accepts only because it still reads the old parser, which
    keeps a form only the new parser reports (coordinator 18:03; a ruled form is an error only in
    the new parser, 19:39). Reported, not gated: the gate is `next_old_parser_accepts_unruled`.
    They leave `next_not_passing` and `next_refused`, not `next_wrong_accepts`. The list is
    deleted with the old parser in stage 5's switching landing (handbook/design/next/migration.md)."""
    return _old_parser_accepts(root)


def next_old_parser_accepts_unruled(root: Path) -> list[str]:
    """Listed old-parser accepts that cite no decision, or whose case expects no new-parser
    syntax error (E0503 or E0504), so the list can't hold a real wrong-accept."""
    out = [f"{case}: cites no decision" for case, decision in _old_parser_accept_rows(root) if not decision]
    for case in _old_parser_accepts(root):
        path = root / "spec/conformance" / case
        expect = next((m.group(1) for l in _lines(path) if (m := re.match(r"//\s*expect:\s*(\S+)", l))),
                      None) if path.is_file() else None
        if expect not in OLD_PARSER_ACCEPTS_CODES:
            out.append(f"{case}: expects {expect}")
    return out


def next_error_typed(root: Path) -> list[str]:
    """Programs cases that link an instance holding an error type: the checker passed a program
    whose instances it did not type, where it should have refused or resolved them (contract 5).
    `tests/programs.rs` fails on a case missing from `error-typed.txt`."""
    listed = root / NEXT_TESTS / "error-typed.txt"
    return [l.strip() for l in _lines(listed) if l.strip()] if listed.is_file() else []


def next_std_unchecked(root: Path) -> list[str]:
    """Std modules the new pipeline does not compile: checking every fn of the module, not
    only those a program reaches, gives a diagnostic in it. `tests/std_modules.rs` keeps
    `std-unchecked.tsv` equal to what it finds; it reaches 0 when std is a program (suite 4)."""
    listed = root / NEXT_TESTS / "std-unchecked.tsv"
    rows = [l.split("\t") for l in _lines(listed) if l.strip()] if listed.is_file() else []
    return [f"{m}: {n.strip()} diagnostics" for m, n in rows if n.strip() != "0"]


def next_core_privilege(root: Path) -> list[str]:
    """Checks in the new pipeline that give core's module a power no other library has
    (`is_library`): core is a library like any other, so 0 is when its types and traits live
    in std/ with markers any library may use."""
    hits = []
    for f in sorted(root.glob("crates/cpl-next*/src/**/*.rs")):
        for i, l in enumerate(_lines(f), 1):
            if "is_library(" in l and "fn is_library" not in l:
                hits.append(f"{f.relative_to(root)}:{i}")
    return hits


NEXT_WHOLE_PROGRAM_TYPES = ("Decls", "Typed", "Loaded", "Whole", "Generic")
NEXT_STRUCT_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+(\w+)")
NEXT_IMPL_RE = re.compile(r"^\s*impl\b(?:\s*<[^{]*?>)?\s+(?:[\w:<>,' &]+?\s+for\s+)?(?:\w+::)*(\w+)\b")
NEXT_FN_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+|async\s+|unsafe\s+)*fn\s+\w+")


def _block(lines: list[str], start: int) -> int:
    """The index of the line closing the `{` block opened on `lines[start]`, or `start` for an
    item that ends at its `;` first."""
    depth = 0
    for j in range(start, len(lines)):
        text = lines[j].split("//")[0]
        if depth == 0 and "{" not in text and text.rstrip().endswith(";"):
            return j
        depth += text.count("{") - text.count("}")
        if depth <= 0 and "{" in "".join(lines[start:j + 1]):
            return j
    return len(lines) - 1


def _whole_res(types: tuple[str, ...], traits: tuple[str, ...] = ()) -> tuple[re.Pattern, re.Pattern]:
    names = r"(?:\w+::)*(?:%s)" % "|".join(types)
    if traits:
        names = r"(?:%s|dyn\s+(?:\w+::)*(?:%s))" % (names, "|".join(traits))
    return (re.compile(r"&(?:'\w+\s+)?(?:mut\s+)?%s\b" % names),
            re.compile(r"\b\w+\s*(?<!:):(?!:)\s*(?:[\w:]+<\s*)*%s\b(?!\s*(?:::|\{|\())" % names))


NEXT_TRAIT_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?trait\s+(\w+)(?:<[^{:]*>)?\s*(?::\s*([^{]+))?\{")
# A view method's key: an item id, a `Sym`, an impl or call index (lane I's rule), or a
# concrete `Ty` by value, which belongs to no item and is compared by structure, never an
# arena index or an interner's first-asked order (coordinator ruling 20:52, link's `impl_for`).
NEXT_VIEW_KEY_RE = re.compile(r"^&?(?:'\w+\s+)?(?:\w+::)*(?:\w+Id|FuncKey|Sym|Ty|usize|u32|u16|u8)$")
NEXT_VIEW_TABLE_RE = re.compile(r"\b(?:str|String|HashMap|BTreeMap|HashSet|Vec|Decls|Typed|Loaded|Whole|Generic|Deferred|SetAdds)\b|\[")


def _params(sig: str) -> list[str]:
    """The types of a fn signature's parameters after `self`, split at top-level commas."""
    start = sig.find("(")
    depth, cur, out = 0, "", []
    for ch in sig[start + 1:]:
        if ch in "(<[":
            depth += 1
        elif ch in ")>]":
            if depth == 0 and ch == ")":
                break
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    out.append(cur)
    types = []
    for p in (x.strip() for x in out):
        if not p or re.fullmatch(r"&?(?:'\w+\s+)?(?:mut\s+)?self", p):
            continue
        at = re.search(r"(?<!:):(?!:)", p)
        types.append(p[at.end():].strip() if at else p)
    return types


def _view_method_ok(sig: str) -> bool:
    """Lane I's rule for one view method: it takes at least one key, every key is an id, a
    `Sym` or a concrete `Ty` (never text or a table), and it answers that key alone, never
    every item of a kind."""
    name = re.search(r"fn\s+(\w+)", sig).group(1)
    if re.match(r"(?:all|every)(?:_|$)", name) or name.endswith("_all"):
        return False
    keys = _params(sig)
    ret = sig.split("->", 1)[1] if "->" in sig else ""
    return bool(keys) and all(NEXT_VIEW_KEY_RE.match(k) and not NEXT_VIEW_TABLE_RE.search(k) for k in keys) \
        and not re.search(r"\b(?:%s)\b" % "|".join(NEXT_WHOLE_PROGRAM_TYPES), ret)


def _view_traits(files: dict[str, list[str]]) -> dict[str, bool]:
    """Each cpl-next* trait and whether a `&dyn` holder of it goes uncounted: every method of
    it and of its supertraits passes `_view_method_ok`."""
    found: dict[str, tuple[list[str], bool]] = {}
    for lines in files.values():
        for i, line in enumerate(lines):
            if not (m := NEXT_TRAIT_RE.match(line)):
                continue
            j, ok, k = _block(lines, i), True, i + 1
            while k <= j:
                if NEXT_FN_RE.match(lines[k]):
                    sig = ""
                    for l in lines[k:j + 1]:
                        sig += l.split("//")[0]
                        if "{" in l or l.rstrip().endswith(";"):
                            break
                    ok = ok and _view_method_ok(sig)
                k += 1
            supers = [x.strip().split("<")[0].split("::")[-1] for x in (m.group(2) or "").split("+")]
            found[m.group(1)] = ([x for x in supers if x[:1].isupper()], ok)
    def passes(t: str, seen: frozenset = frozenset()) -> bool:
        if t not in found or t in seen:
            return True
        supers, ok = found[t]
        return ok and all(passes(x, seen | {t}) for x in supers)
    return {t: passes(t) for t in found}


def next_whole_program_inputs(root: Path) -> list[str]:
    """Each place the new pipeline holds a whole-program value (shortcut audit S2, condition 2
    of the 15:00 review): `Decls`, `Typed`, `Loaded`, `Whole` or `Generic` by reference
    anywhere, or owned as a fn parameter, closure parameter or struct field; a field or
    parameter whose type is a struct holding one (transitively); and each method taking `self`
    of such a struct, which reaches the whole program through `self`, except a builder that
    returns its own struct and takes none: a snapshot setter of a plain setting (coordinator
    ruling on audit #14's S16). The five types' own
    methods are the value itself, not a reader of it, and a fn returning one produces it. The
    incremental core (handbook/design/next/incremental.md) calls every layer per item over
    small views, so this reaches 0: a context that holds views instead stops counting. A
    `&dyn T` view is such a value unless every method of T and its supertraits takes an id,
    `Sym` or concrete `Ty` key and answers that key alone (`_view_method_ok`, lane I's rule)."""
    return _holding_lines(root, NEXT_WHOLE_PROGRAM_TYPES, "", views=True)


def _holding_lines(root: Path, types: tuple[str, ...], within: str, views: bool = False) -> list[str]:
    """`next_whole_program_inputs`'s rule for the values `types`, counted in files under
    `within`; holders are found in every cpl-next* crate. With `views`, a `&dyn` of a trait
    failing lane I's view rule is such a value too."""
    files = {}
    for p in _rs_files(root, *_next_srcs(root), tests=False):
        files[_rel(root, p)] = list(_lines(p))
    failing = tuple(sorted(t for t, ok in _view_traits(files).items() if not ok)) if views else ()
    by_ref, owned = _whole_res(types, failing)
    def code(line: str) -> str:
        return "" if line.lstrip().startswith("//") else line.split("->")[0]
    structs, impls = [], []
    for rel, lines in files.items():
        for i, line in enumerate(lines):
            if line.lstrip().startswith("//"):
                continue
            if m := NEXT_STRUCT_RE.match(line):
                structs.append((m.group(1), rel, i, _block(lines, i)))
            elif m := NEXT_IMPL_RE.match(line):
                impls.append((m.group(1), rel, i, _block(lines, i)))
    holders: dict[str, set[str]] = {}
    crates: dict[str, set[str]] = {}
    # Each file's `use` items, as (words, globs, pub); a multi-line `use a::{b, c};` is one.
    uses = {rel: [(set(re.findall(r"\w+", u)), "*" in u, bool(v))
                  for v, u in re.findall(r"^\s*(pub(?:\([^)]*\))?\s+)?use\s+([^;]*);", "\n".join(lines), re.M)]
            for rel, lines in files.items()}
    local = {"crate", "self", "super"}
    def seen(name: str, rel: str) -> bool:
        """Whether a bare `name` in `rel` can be the holder: its crate defines one, or a `use`
        item imports it (or globs) from a path naming the holder's module or crate, directly or
        through a `pub use` re-export of it in this crate."""
        krate = rel.split("/")[1]
        if krate in crates[name]:
            return True
        where = holders[name] - local
        def brings(items, only_pub=False):
            return [words for words, glob, pub in items
                    if (name in words or glob) and (pub or not only_pub)]
        if any(words & where for words in brings(uses[rel])):
            return True
        return any(words & local for words in brings(uses[rel])) and any(
            words & where for r, items in uses.items() if r.split("/")[1] == krate
            for words in brings(items, only_pub=True))
    def paths(rel: str) -> set[str]:
        """The path segments that may qualify a struct defined in `rel`: its file's module, its
        crate's name and the short name the pipeline imports it by."""
        krate = rel.split("/")[1].replace("-", "_")
        return {Path(rel).stem, krate, krate.removeprefix("cpl_next_"), "crate", "self", "super"}
    def holds(line: str, rel: str) -> bool:
        text = code(line)
        if by_ref.search(line) and text.strip() or owned.search(text):
            return True
        at = re.search(r"(?<!:):(?!:)", text)
        if not holders or not at:
            return False
        names = "|".join(sorted(holders))
        return any(m.group(1) in holders[m.group(2)] if m.group(1) else seen(m.group(2), rel)
                   for m in re.finditer(r"(?:(\w+)::)?\b(%s)\b(?!\s*(?:::|\{|\())" % names,
                                        text[at.end():]))
    def tuple_holds(first: str, rel: str) -> bool:
        if "(" not in first or "{" in first:
            return False
        inner = first[first.index("(") + 1:first.rindex(")") if ")" in first else None]
        return any(holds("x: " + f.strip().removeprefix("pub "), rel) for f in inner.split(","))
    grew = True
    while grew:
        grew = False
        for name, rel, i, j in structs:
            if name in holders or name in types:
                continue
            if any(holds(line, rel) for line in files[rel][i + 1:j + 1]) or tuple_holds(files[rel][i], rel):
                holders.setdefault(name, set()).update(paths(rel))
                crates.setdefault(name, set()).add(rel.split("/")[1])
                grew = True
    own = {(rel, k) for name, rel, i, j in structs + impls if name in types
           for k in range(i, j + 1)}
    out = {(rel, k) for rel, lines in files.items() for k, line in enumerate(lines)
           if (rel, k) not in own and holds(line, rel)}
    out |= {(rel, i) for name, rel, i, j in structs if name in holders and tuple_holds(files[rel][i], rel)}
    held = sorted(set(types) | set(holders))
    for name, rel, i, j in impls:
        # A builder returning its own struct with one more plain setting reads nothing.
        handed = re.compile(r"\b(?:%s)\b" % "|".join(t for t in held if t != name))
        # An impl is of the struct its own crate defines under that name.
        if rel.split("/")[1] not in crates.get(name, ()):
            continue
        lines = files[rel]
        for k in range(i + 1, j + 1):
            if not NEXT_FN_RE.match(lines[k]):
                continue
            sig = ""
            for line in lines[k:j + 1]:
                sig += "" if line.lstrip().startswith("//") else line.split("//")[0]
                if "{" in line or line.rstrip().endswith(";"):
                    break
            head = sig.split("{")[0]
            if not re.search(r"\bself\b", head.split("->")[0]):
                continue
            ret = head.split("->", 1)[1] if "->" in head else ""
            builds = re.fullmatch(r"\s*(?:Self|%s)(?:<[^>]*>)?\s*" % name, ret)
            if any(handed.search(t) for t in _params(head)) or not builds:
                out.add((rel, k))
    return [f"{rel}:{k + 1}" for rel, k in sorted(out) if rel.startswith(within)]


def _next_file_lines(root: Path, rel: str) -> list[str]:
    p = root / rel
    if not p.is_file():
        return []
    return [f"{rel}:{i}" for i, l in enumerate(_lines(p), 1) if l.strip() and not l.lstrip().startswith("//")]


def next_reference_evaluator_lines(root: Path) -> list[str]:
    """Code lines of the new pipeline's reference interpreter: one evaluator (the engine)
    serves constant evaluation, the prover and runs, so this reaches 0 once the engine agrees
    on every case."""
    src = root / "crates/cpl-next-interp/src"
    return [l for f in _rs_files(root, "crates/cpl-next-interp/src", tests=False) if src.is_dir()
            for l in _next_file_lines(root, _rel(root, f))]


# A verdict comes from the pipeline, never from the harness (shortcut audit S14).
HARNESS_REFUSAL_RE = re.compile(r'\("refused",')


def next_harness_refusals(root: Path) -> list[str]:
    """Sites in the new pipeline's programs harness that answer `refused` themselves, for a
    header or exit code the harness does not judge, rather than from the pipeline's `Unbuilt`.
    0 is every refusal the pipeline's own."""
    rel = NEXT_TESTS + "/programs.rs"
    if not (root / rel).is_file():
        return []
    return [f"{rel}:{i}" for i, l in enumerate(_lines(root / rel), 1) if HARNESS_REFUSAL_RE.search(l)]


# The tests' own harness binary: `--raw`, `--canonical` and `--all` run the oracle for them.
NEXT_ORACLE_HOST = "crates/cpl-next/src/bin/cpl-next-oracle.rs"


def next_oracle_callers(root: Path) -> list[str]:
    """Non-test lines naming the reference interpreter (`cpl_next_interp`) outside its crate and
    the tests' harness binary: it is the tests' oracle, and constant evaluation, the prover, runs
    and traces all go through the engine (11:35 ruling). Counts every line naming it, so moving
    a call into another fn cannot pay it off. 0 is one evaluator for everything but the tests."""
    out = []
    for f in _rs_files(root, tests=False):
        rel = _rel(root, f)
        if rel.startswith("crates/cpl-next-interp/") or rel == NEXT_ORACLE_HOST:
            continue
        for i, l in enumerate(_lines(f), 1):
            if "cpl_next_interp" in l and not l.lstrip().startswith("//"):
                out.append(f"{rel}:{i}")
    return out



NEXT_REPR_HOME = ("crates/cpl-next-ir/src/prog.rs", "crates/cpl-next-ir/src/ty.rs")


NEXT_REPR_NAMES = {"unlent", "plain", "plain_in", "stores"}
NEXT_REPR_FN_RE = re.compile(r"^\s*(pub(\([a-z]+\))? )?fn (?P<name>\w+)[^{]*&C?Ty\b[^{]*->\s*(bool|Holds)\b")
NEXT_REPR_ANY_FN_RE = re.compile(r"^\s*(pub(\([a-z]+\))? )?fn \w+")
# One match arm that puts fn, closure or storage types beside scalars: a representation answer.
NEXT_REPR_ARM_RE = re.compile(r"\bC?Ty::(Int|Bool|Char|U64|F64)\b[^=]*?\|\s*C?Ty::(Fn|Clo|Raw)\b"
                              r"|\bC?Ty::(Fn|Clo|Raw)\b[^=]*?\|\s*C?Ty::(Int|Bool|Char|U64|F64)\b")


def _indent(l: str) -> int:
    return len(l) - len(l.lstrip())


NEXT_INTR_RE = re.compile(r"\bIntr::(\w+)")
# The back ends that hold the machine primitives' arms (decision 653); the C++ takes every
# operation's text from `ops`.
NEXT_OP_BACKENDS = ("crates/cpl-next-engine/src/", "crates/cpl-next-interp/src/")


def _machine_ops(root: Path) -> set[str]:
    ops = root / "crates/cpl-next-ir/src/ops.rs"
    if not ops.is_file():
        return set()
    text = ops.read_text()
    m = re.search(r"pub const MACHINE: \[Intr; \d+\] = \[(.*?)\];", text, re.S)
    return set(NEXT_INTR_RE.findall(m.group(1))) if m else set()


def _intr_arm(lines: list[str], i: int, at: int) -> bool:
    """Whether the `Intr::` at column `at` of line `i` (1-based) is in a match pattern: in a
    `matches!`, before an arm's `=>` on its line, or one of a pattern's `|` alternatives."""
    code = lines[i - 1].split("//")[0]
    before, after = code[:at], code[at:]
    if "matches!(" in before or ("=>" in after and "=>" not in before):
        return True
    if "=>" in code:
        return False
    s = code.strip()
    nxt = next((x.strip() for x in lines[i:] if x.strip()), "")
    return s.startswith("|") or s.endswith("|") or nxt.startswith("|")


def next_op_arms_outside_ir(root: Path) -> list[str]:
    """`Intr::` match arms outside `cpl-next-ir` that are not a back end's machine primitive
    (`ops::MACHINE`, decision 653), and machine primitives a back end has no arm for. Every
    other operation's meaning is `ops` data that each back end reads, so a back end that
    matches one has a meaning of its own. Counts match patterns, not fn names. 0 is
    condition 6: the back ends are generated from one op table."""
    if not (root / "crates/cpl-next-ir").is_dir():
        return []
    machine, out = _machine_ops(root), []
    if not machine:
        out.append("crates/cpl-next-ir/src/ops.rs: no `MACHINE` table")
    seen = {b: set() for b in NEXT_OP_BACKENDS}
    for p in _rs_files(root, *_next_srcs(root), tests=False):
        rel = _rel(root, p)
        if rel.startswith("crates/cpl-next-ir/"):
            continue
        lines = _lines(p)
        for i, l in enumerate(lines, 1):
            if l.strip() == "#[cfg(test)]" and i < len(lines) and NEXT_TEST_MOD_RE.match(lines[i]):
                break
            if l.lstrip().startswith("//"):
                continue
            for m in NEXT_INTR_RE.finditer(l):
                if not _intr_arm(lines, i, m.start()):
                    continue
                b = next((b for b in NEXT_OP_BACKENDS if rel.startswith(b)), None)
                if b and m.group(1) in machine:
                    seen[b].add(m.group(1))
                else:
                    out.append(f"{rel}:{i} Intr::{m.group(1)}")
    for b, ops in seen.items():
        out += [f"{b}: no arm for Intr::{o}" for o in sorted(machine - ops)]
    return out


def next_repr_second_defs(root: Path) -> list[str]:
    """Definitions of a value-representation predicate (`unlent`, `plain`, `plain_in`, `stores`)
    outside the IR's one classifier (`Program::holds` and its readers, `CTy::unlent`,
    `Ty::unlent`), in the new pipeline's non-test code: every pass and backend reads the one
    answer (1 PM audit). Structural: a fn over a type returning `bool` or `Holds` with a match arm
    putting fn, closure or storage types beside scalars counts under any name; the names count too."""
    out = []
    for f in _rs_files(root, tests=False):
        rel = _rel(root, f)
        if not rel.startswith("crates/cpl-next") or rel in NEXT_REPR_HOME or f.name == "tests.rs":
            continue
        lines = _lines(f)
        for i, l in enumerate(lines):
            m = NEXT_REPR_FN_RE.match(l)
            if not m:
                continue
            body = []
            for b in lines[i + 1:]:
                if NEXT_REPR_ANY_FN_RE.match(b) and _indent(b) <= _indent(l):
                    break
                body.append(b)
            body = "\n".join(body)
            if m.group("name") in NEXT_REPR_NAMES or NEXT_REPR_ARM_RE.search(body):
                out.append(f"{rel}:{i + 1} {m.group('name')}")
    return out

NEXT_LAYOUT_READERS = ("crates/cpl-next-cpp/src/", "crates/cpl-next-engine/src/", "crates/cpl-next-interp/src/")


def next_layout_outside_table(root: Path) -> list[str]:
    """Lines in a new-pipeline backend (the engine, the C++ emitter) that read a struct's fields
    or a union's members straight from `Program::structs`/`Program::sums`: the field order, the
    members and where the tag goes come from the IR's one layout table
    (`cpl_next_ir::layout::Layouts`), so the two backends cannot lay a value out differently."""
    pat = re.compile(r"\.(structs|sums)\b")
    out = []
    for f in _rs_files(root, tests=False):
        rel = _rel(root, f)
        if not rel.startswith(NEXT_LAYOUT_READERS):
            continue
        for i, l in enumerate(_lines(f), 1):
            if pat.search(l.split("//")[0]):
                out.append(f"{rel}:{i}")
    return out


NEXT_STANDIN_RE = re.compile(r"stand-in|stands in|until .* (stage|lands|loads)")


def next_standin_notes(root: Path) -> list[str]:
    """Non-test lines in the new pipeline that say the code beside them is a stand-in for a
    designed component (rule 11, b9d4cd026): each goes when the component it waits on lands."""
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, *_next_srcs(root), tests=False) if p.name != "tests.rs"
            for i, l in enumerate(_lines(p), 1) if NEXT_STANDIN_RE.search(l)]


def next_source_text_guards(root: Path) -> list[str]:
    """`include_str!` of a Rust source file in the new pipeline: an architecture guard that
    reads code as text passes vacuously once a marker moves; it becomes a structural (IR-level
    or data-table) check."""
    pat = re.compile(r'include_str!\("[^"]*\.rs"\)')
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, *_next_srcs(root))
            for i, l in enumerate(_lines(p), 1) if pat.search(l)]


def next_synthetic_name_tests(root: Path) -> list[str]:
    """A test of a name's text for the `$` that marks a synthetic local or domain in cpl-next's
    checker: whether a name is synthetic is a flag the item carries, not a spelling read back
    (coordinator ruling; lane D's `Local.synthetic`), so this goes to 0 and stays there."""
    pat = re.compile(r"""starts_with\(\s*(?:'\$'|"\$")\s*\)""")
    src = root / "crates/cpl-next-check/src"
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, src)
            for i, l in enumerate(_lines(p), 1) if pat.search(l)] if src.is_dir() else []


DECLS_OP_ARM_RE = re.compile(r"\bTokenKind::(?:Plus|Minus|Star|StarStar|Slash|SlashSlash|Percent|Shl|Shr|"
                             r"Amp|Pipe|Caret|Tilde)\b[^=]*=>")


def decls_const_op_arms(root: Path) -> list[str]:
    """Per-operator `TokenKind::` match arms in cpl-next-decls: a constant in a type (a range's
    end, an array length) is evaluated by ir's operator table, the one the run uses, so no layer
    keeps a second copy of `+`'s meaning (coordinator ruling; lane D's tyconv.rs `const_value`)."""
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, "crates/cpl-next-decls/src", tests=False)
            for i, l in enumerate(_lines(p), 1) if DECLS_OP_ARM_RE.search(l)]


def next_core_standin_lines(root: Path) -> list[str]:
    """Lines of `crates/cpl-next/std.cp`, the private stand-in for core: std loads like any
    library through the module loader, so this reaches 0 when core lives in std/."""
    p = root / "crates/cpl-next/std.cp"
    return [f"crates/cpl-next/std.cp:{i}" for i, l in enumerate(_lines(p), 1) if l.strip()] if p.is_file() else []


NEXT_RAW_OPEN_RE = re.compile(r'\br(#*)"')
# C++ kept in its own file and pulled in whole is hand-written runtime too.
NEXT_INCLUDED_CPP_RE = re.compile(r'include_str!\(\s*"[^"]*\.(?:h|hpp|cpp|cc|inc)"')


NEXT_TEST_MOD_RE = re.compile(r"^\s*mod \w+ \{")


def next_cpp_runtime_lines(root: Path) -> list[str]:
    """Lines of C++ written by hand inside the new emitter (its raw-string runtime): each runtime
    operation is a CP std body or an `ops` table entry, so C++ cannot disagree with the engine."""
    out, inside, close = [], False, ""
    for p in _rs_files(root, *_next_srcs(root), tests=False):
        rel = _rel(root, p)
        lines = _lines(p)
        for i, l in enumerate(lines, 1):
            # Comments and a file's `#[cfg(test)] mod` hold no emitter C++ (ca568afe1 counted
            # cpl-next-syntax's lexer comments and its tests' raw strings).
            if not inside and l.strip() == "#[cfg(test)]" and i < len(lines) and NEXT_TEST_MOD_RE.match(lines[i]):
                break
            if not inside and l.lstrip().startswith("//"):
                continue
            # A raw string of any hash count (`r"`, `r#"`, `r##"`), not only `r#"` (shortcut audit).
            m = None if inside else NEXT_RAW_OPEN_RE.search(l)
            if inside or m:
                if l.strip():
                    out.append(f"{rel}:{i}")
                if m:
                    close = '"' + m.group(1)
                inside = close not in (l[m.end():] if m else l)
            if NEXT_INCLUDED_CPP_RE.search(l):
                out.append(f"{rel}:{i}: include_str")
    return out


def next_wrong_accepts(root: Path) -> list[str]:
    """Cases the new pipeline accepts that their header says it must reject: a soundness hole,
    worse for a contestant than a wrong reject, so it gets its own floor beside `next_wrong`."""
    return _wrong_ids(root, "wrong-accept")


# Codes link and own may say: none is a user code, since the checker owns every verdict
# (building.md rule 5). E9999 is an internal error, counted by `next_wrong`.
NEXT_LATE_CODES_ALLOWED = {"E9999"}
NEXT_LATE_REJECT_RE = re.compile(r"^(link|own): .*?\b(E\d{4})\b")


def next_late_rejects(root: Path) -> list[str]:
    """Cases the spec accepts that the new pipeline rejects with a user error code from link or
    from ownership after it (`wrong-reject` rows saying `link:` or `own:`). Whole-program steps
    choose representations and do not reject (decision 31): a user error comes per unit, and a
    feature not built yet is an `Unbuilt` refusal, never the contestant's error. Reaches 0."""
    wrong = _wrong_rows_path(root)
    out = []
    for l in _lines(wrong) if wrong.is_file() else []:
        cols = l.split("\t")
        m = len(cols) >= 3 and cols[1] == "wrong-reject" and NEXT_LATE_REJECT_RE.match(cols[2])
        if m and m.group(2) not in NEXT_LATE_CODES_ALLOWED:
            out.append(f"{cols[0]}: {m.group(1)} {m.group(2)}")
    return out


def next_split(root: Path) -> list[str]:
    """Cases whose three runs disagree (`split` in `wrong.tsv`): the interpreter, the engine and
    the C++ program answer differently, the one verdict that falsifies "one source of semantics".
    It is counted apart from `next_wrong` so a split is never filed as one more wrong reject."""
    return _wrong_ids(root, "split")


def _wrong_rows_path(root: Path) -> Path:
    return _run_list_path(root, "wrong.tsv")


def _run_list_path(root: Path, name: str) -> Path:
    """The list `name` to read: the full programs run's own (`target/cpl-next/<name>`) when it
    is newer than HEAD and the committed list, else the committed list. A run is the evidence;
    the committed list is only what someone last copied, and gating on it alone let a swap sit
    in the baseline (09:00 trend audit). A focused run writes `target/cpl-next/only/` instead."""
    committed, run = root / NEXT_TESTS / name, root / "target/cpl-next" / name
    if not run.is_file():
        return committed
    head = _git(root, "log", "-1", "--format=%ct")
    newer_than_head = bool(head) and head[0].isdigit() and run.stat().st_mtime >= int(head[0])
    newer_than_list = not committed.is_file() or run.stat().st_mtime >= committed.stat().st_mtime
    return run if newer_than_head and newer_than_list else committed


def _wrong_ids(root: Path, kind: str) -> list[str]:
    wrong = _wrong_rows_path(root)
    return [l.split("\t")[0] for l in _lines(wrong)
            if l.count("\t") >= 1 and l.split("\t")[1] == kind] if wrong.is_file() else []


def next_wrong_outputs(root: Path) -> list[str]:
    """Cases that run and print what the reference interpreter and C++ agree on, but not the
    expected output, a `has:`/`lacks:` note or a trap note: the answer a contestant submits is
    wrong, so each case is gated like a wrong accept."""
    return _wrong_ids(root, "wrong-output")


def next_listed_twice(root: Path) -> list[str]:
    """Cases on two of the committed passing, wrong and masked lists. One run writes them, so a
    case is on one at most; a second listing is a hand edit or a merge, and the lists then say
    two things about it."""
    seen: dict[str, str] = {}
    out = []
    for name in ("passing.txt", "wrong.tsv", "masked.txt"):
        path = root / NEXT_TESTS / name
        for l in _lines(path) if path.is_file() else []:
            i = l.split("\t")[0].strip()
            if i and i in seen and seen[i] != name:
                out.append(f"{i}: {seen[i]} and {name}")
            seen.setdefault(i, name)
    return out


# Counters gated by case, not count: one fixed and one newly broken keep the count,
# and for a soundness hole or a split run that swap is the regression (07:00 trend audit).
ID_SETS = {"next_wrong_accepts_ids": next_wrong_accepts, "next_split_ids": next_split,
           "next_wrong_outputs_ids": next_wrong_outputs}


def next_core_checks(root: Path) -> list[str]:
    """Places where new-pipeline code singles out core or a std module: the `CORE` id or name in
    any use (`== CORE`, `CORE.0`, `fn_named(CORE, ..)`), core's stand-in refusals, a module
    named `"core"`, and std source built into the binary. core
    and std are ordinary libraries, so nothing may treat them differently (S6)."""
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, *_next_srcs(root), tests=False)
            for i, l in enumerate(_lines(p), 1)
            if not l.lstrip().startswith("//") and NEXT_CORE_USE_RE.search(l) and not NEXT_CORE_DEF_RE.match(l)]


# Any use of the core module's id, not just `== CORE`: `decls::CORE`, `CORE.0`,
# `fn_named(CORE, ..)` and `matches!(m, CORE)` single it out the same way (shortcut audit).
# Core's stand-in refusals, core known by its name, and library source built into the binary
# single it out too; dotted std module names are next_std_names'.
NEXT_CORE_USE_RE = re.compile(r'\bCORE\b|\bCoreStandIn|"core"|include_str!\([^)]*\.cp"')
NEXT_CORE_DEF_RE = re.compile(r"\s*(?:pub(?:\([^)]*\))?\s+)?const\s+CORE\b")


def next_ignored_settings(root: Path) -> list[str]:
    """Settings the new pipeline accepts and then ignores: each source name in a match arm whose
    body is empty, as `Some("ranges" | "bounds") if on => {}` for `with checks(..)` fields that
    mean something only under a release profile (shortcut audit S15). The target is 0: each
    becomes a build input every backend honours, or is refused until it is one."""
    arm = re.compile(r'^\s*(?:Some\()?((?:"\w+"\s*\|?\s*)+)\)?(?:\s+if\s+[^=]+)?\s*=>\s*\{\}\s*,?\s*$')
    return [f"{_rel(root, p)}:{i} {n}" for p in _rs_files(root, *_next_srcs(root), tests=False)
            for i, l in enumerate(_lines(p), 1) if (m := arm.match(l))
            for n in re.findall(r'"(\w+)"', m.group(1))]


def next_hidden_const_evaluators(root: Path) -> list[str]:
    """Constant evaluators outside the consts layer (audit F4, consts.md): each `impl .. Consts
    for` and each call of the engine's `eval` in a cpl-next* crate other than `cpl-next-consts`
    and the engine itself. Such an evaluator runs lower, own and the engine from wherever it
    sits, over whole-program `Decls`/`Loaded`; the `const_value(SourceId)` query in
    `cpl-next-consts` replaces it. Target 0."""
    pat = re.compile(r"\bimpl\b.*\bConsts\s+for\b|\bcpl_next_engine::eval\(")
    skip = tuple(f"crates/{c}/" for c in ("cpl-next-consts", "cpl-next-engine"))
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, *_next_srcs(root), tests=False)
            if not _rel(root, p).startswith(skip)
            for i, l in enumerate(_lines(p), 1) if pat.search(l)]


def event_labels(root: Path) -> list[str]:
    """New-pipeline events named by a label string: `Stmt::Event(format!(..), ..)`, or a backend
    matching one apart as `Event(label, ..)`. An event is a typed point (kind, fn, site) that the facade
    renders; a label is a second, textual answer to which event fired (events.md). Target 0."""
    pat = re.compile(r'Event\(\s*(?:format!|"|String::|label\b)')
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, *_next_srcs(root), tests=False)
            for i, l in enumerate(_lines(p), 1) if pat.search(l)]


def trace_old_interp_uses(root: Path) -> list[str]:
    """Tool and facade lines naming `cpl_interp::trace`, the old pipeline's step log. Trace and
    the debugger read the new pipeline's event stream instead; the count reaches 0 at the switch
    that deletes `cpl-interp` (events.md, migration rule 13)."""
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, tests=False)
            if not _rel(root, p).startswith("crates/cpl-interp/")
            for i, l in enumerate(_lines(p), 1) if "cpl_interp::trace" in l]


def next_name_compares(root: Path) -> list[str]:
    """Places new-pipeline code compares source text against a fixed identifier: `== "print"`,
    a `"Some" =>` match arm, `matches!(.., "todo")`, `.eq("on")`. Each is the compiler knowing a
    name, decorator or variant instead of an id or a declared role; `next_std_names` only saw
    literals equal to std declarations (11:00 blind audit). Intrinsic ids and primitive type
    names count too until they are table data; the target is a role table std fills."""
    # A match arm with a fixed name anywhere in its pattern (`Some("x") =>`, `("x", _) =>`),
    # and a keyed lookup by one (`.get("x")`), are the same comparison (shortcut audit).
    pat = re.compile(r'(?:==|!=)\s*"[A-Za-z_]\w*"|^\s*(?:\|\s*)?[^=|]*"[A-Za-z_]\w*"[^=]*(?:\|.*)?=>'
                     r'|matches!\([^)]*"[A-Za-z_]\w*"|\.eq\(\s*"[A-Za-z_]\w*"\s*\)'
                     r'|\.(?:get|get_mut|contains_key|remove|entry)\(\s*"[A-Za-z_]\w*"\s*\)')
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, *_next_srcs(root), tests=False)
            for i, l in enumerate(_lines(p), 1)
            if not l.lstrip().startswith("//") and pat.search(l)]


def next_library_gates(root: Path) -> list[str]:
    """Places new-pipeline code asks whether a module is a library (`is_library`), or wraps that
    question (`items_only`): a decorator or rule only std may use is std privilege under a new
    name (09:00 blind audit). Each call counts, so renaming the gate or hiding it in a helper keeps
    the count; it drops only as callers go."""
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, *_next_srcs(root), tests=False)
            for i, l in enumerate(_lines(p), 1)
            if not l.lstrip().startswith("//") and re.search(r"\b(?:is_library|items_only)\b", l)]


NEXT_OWN_DECIDERS_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))? )?fn (\w*(?:lend|drop)\w*)")


def _built_at(text: str, at: int) -> bool:
    """Whether the `Name(` at `at` builds a value rather than matching one: the parens close
    without a `=>`, `|`, `if` or `@` after them, and the line holds no `matches!`."""
    depth, k = 0, text.index("(", at)
    while k < len(text):
        depth += {"(": 1, ")": -1}.get(text[k], 0)
        k += 1
        if depth == 0:
            break
    start = text.rfind("\n", 0, at) + 1
    before, after = text[start:at], text[k:text.find("\n", at)]
    rest = after.lstrip()
    return ("matches!" not in before and not NEXT_OWN_SAME_KIND_RE.search(before) and "=>" not in after and not re.search(r"\blet\b[^=]*$", before)
            and not re.match(r"(\||if\b|@)", rest))


NEXT_OWN_BUILDS_RE = re.compile(r"\b(Stmt::Drop|Arg::View|Arg::Lend)\(")
NEXT_OWN_FIELD_RE = re.compile(r"\.drops(\.push\(|\.insert\(|\s*=[^=])")
# A node rebuilt from one of its own kind (a substitution, or a written `mut` lowered to a lend).
NEXT_OWN_SAME_KIND_RE = re.compile(r"::(Drop|View|Lend|Mut)\([^)]*\)\s*=>")


def next_ownership_outside_own(root: Path) -> list[str]:
    """Places outside `cpl-next-own` that build an ownership decision into the IR: a
    `Stmt::Drop`, `Arg::View` or `Arg::Lend` built (not matched, and not rebuilt from a node of
    its own kind), or `Func::drops` written. Ownership is decided once, in the ownership pass (contract 04); every later layer,
    the IR crate included, only reads it. Structural, so a rename cannot pay it off."""
    out = []
    for p in _rs_files(root, *_next_srcs(root), tests=False):
        rel = _rel(root, p)
        if rel.startswith("crates/cpl-next-own/") or p.name == "tests.rs":
            continue
        text = "\n".join(_lines(p))
        for m in NEXT_OWN_BUILDS_RE.finditer(text):
            if _built_at(text, m.start()):
                out.append(f"{rel}:{text.count(chr(10), 0, m.start()) + 1} {m.group(1)}")
        for i, l in enumerate(_lines(p), 1):
            if NEXT_OWN_FIELD_RE.search(l.split("//")[0]):
                out.append(f"{rel}:{i} Func field")
    return out


NEXT_OWNS_READ_RE = re.compile(r"\bowns\(|\.owns\[")
# Code that turns IR into execution: where a read of "does this layout own" places a drop or retain.
NEXT_PLACERS = ("crates/cpl-next-engine/src", "crates/cpl-next-cpp/src", "crates/cpl-next-interp/src",
                "crates/cpl-next-lower/src", "crates/cpl-next-link/src")


def next_backend_owns_reads(root: Path) -> list[str]:
    """Reads of whether a layout owns (`lays.owns[..]`, `owns(ty)`) in code that turns IR into
    execution: each one is a drop or retain placed by the backend rather than read from an
    ownership node (contract 04). The whole engine counts, its exec and peephole passes too: since
    812e28ddd neither reads an owns bit. Counts reads, not fn names, so a rename
    cannot pay it off. 0 is every drop and copy an IR node from cpl-next-own."""
    out = []
    for d in NEXT_PLACERS:
        q = root / d
        files = [q] if q.is_file() else _rs_files(root, d, tests=False) if q.is_dir() else []
        for f in files:
            for i, l in enumerate(_lines(f), 1):
                t = l.lstrip()
                if t.startswith("//") or re.match(r"(pub(\([^)]*\))?\s+)?fn\s+owns\(", t):
                    continue
                if NEXT_OWNS_READ_RE.search(l):
                    out.append(f"{_rel(root, f)}:{i}")
    return out


FAST_TYPES = ("Int", "U64")


def fast_unchecked(root: Path) -> list[str]:
    """`@fast` fns of std that `fast_ops_agree_with_their_bodies` (crates/cpl-next/tests/
    programs.rs) cannot call, by its rule: one parameter of type `Int`, `U64` or a `W: Word`
    whose every `impl Word` type is one of those. A fn means its CP body; its op is only a
    faster way to compute it, so an op nothing compares with its body may differ from it."""
    files = sorted((root / "std").rglob("*.cp"))
    srcs = [(p, _lines(p)) for p in files]
    words = [m.group(1) for _, ls in srcs for l in ls if (m := re.match(r"impl Word for (\w+):$", l))]
    out = []
    for p, ls in srcs:
        for i, l in enumerate(ls):
            if not l.startswith("@fast("):
                continue
            sig = next((x[len("pub fn "):] for x in ls[i:] if x.startswith("pub fn ")), None)
            m = sig and re.match(r"(\w+)(?:<(\w+): Word>)?\(\w+: (\w+)\)", sig)
            at = [] if not m else words if m.group(3) == m.group(2) else [m.group(3)]
            if not at or any(t not in FAST_TYPES for t in at):
                name = re.match(r"\w*", sig).group(0) if sig else "no fn"
                out.append(f"{_rel(root, p)}:{i + 1} {name}")
    return out


NEXT_AFTER_OWN = ("cpl-next-lower", "cpl-next-link", "cpl-next-cpp", "cpl-next-engine", "cpl-next-interp")


def next_lent_outside_own(root: Path) -> list[str]:
    """Places a layer other than the checker or `cpl-next-own` makes a type lent (`Ty::Lent(Box::new(..))`):
    whether a value is lent or moved is an ownership decision, made once in the ownership pass
    (contract 04); lowering, linking and the engines only read it. Function names miss this
    (lowering's `lent_targs`), so the constructor is what is counted. Lowering runs before
    ownership marks anything, so any test there of a lent or reference type counts too."""
    dirs = [f"crates/{c}/src" for c in NEXT_AFTER_OWN if (root / "crates" / c / "src").is_dir()]
    out = []
    for p in _rs_files(root, *dirs, tests=False):
        ls = _lines(p)
        lower = "cpl-next-lower" in p.parts
        for i, l in enumerate(ls, 1):
            if l.lstrip().startswith("//"):
                continue
            if lower and NEXT_LENT_TY_RE.search(l):
                out.append(f"{_rel(root, p)}:{i}")
                continue
            if not NEXT_MAKES_LENT_RE.search(l):
                continue
            # An arm of a `match` on a type it matched lent (`Ty::Lent(e) => match f(e) {`).
            if any(NEXT_LENT_MATCH_RE.search(b) for b in ls[max(0, i - 5):i - 1]):
                continue
            out.append(f"{_rel(root, p)}:{i}")
    return out


# `Ty::Lent(Box::new(..))` built from a type not already lent: a match arm that rebuilds a lent
# type it matched (`Lent(e) => Lent(Box::new(f(e)))`) keeps a decision, it does not make one.
NEXT_MAKES_LENT_RE = re.compile(r"^(?!.*(?:Lent|Ref)\([^)]*\)\)?\s*=>.*Ty::(?:Lent|Ref)\(Box::new).*Ty::(?:Lent|Ref)\(Box::new")
NEXT_LENT_TY_RE = re.compile(r"\bTy::(?:Lent|Ref)\b")
NEXT_LENT_MATCH_RE = re.compile(r"\bTy::(?:Lent|Ref)\(\w+\)\s*=>\s*match\b")


NEXT_WEAK_EXPECT_RE = re.compile(r"^// expect: (exit 1|ok)\s*$")


def next_weak_oracle_passes(root: Path) -> list[str]:
    """Passing cases whose oracle does not pin the behaviour: `// expect: exit 1` (any rejection
    passes) or `// expect: ok` with no `// output:` or `// last:` (it only has to run). Each should name the
    code and line it rejects with, or the output it prints."""
    out = []
    listed = root / NEXT_TESTS / "passing.txt"
    for c in _lines(listed) if listed.is_file() else []:
        c = c.strip()
        f = root / (NEXT_TESTS + "/programs/" + c.removeprefix("next/") if c.startswith("next/") else "spec/conformance/" + c)
        head = list(itertools.takewhile(lambda l: l.startswith("//"), _lines(f))) if f.is_file() else []
        weak = [m.group(1) for l in head if (m := NEXT_WEAK_EXPECT_RE.match(l))]
        if "exit 1" in weak or ("ok" in weak and not any(l.startswith(("// output:", "// last:")) for l in head)):
            out.append(c)
    return out


SYNTAX_SRCS = ("crates/cpl-syntax/src", "crates/cpl-next-syntax/src")


def syntax_std_names(root: Path) -> list[str]:
    """String literals in the lexer and parser (`cpl-syntax`, `cpl-next-syntax`) equal to a type std declares: the parser
    turning `Bounds` into `Cut` knows the library by name below every layer `next_std_names` scans."""
    names = set()
    for f in sorted((root / "std").glob("**/*.cp")):
        names |= {n for n in NEXT_STD_DECL_RE.findall(f.read_text(encoding="utf-8")) if n[:1].isupper()}
    return [f"{_rel(root, p)}:{i}: {n}" for p in _rs_files(root, *SYNTAX_SRCS, tests=False)
            if not p.name.endswith("_tests.rs")
            for i, l in enumerate(_lines(p), 1) if not l.lstrip().startswith("//")
            for n in RUST_STR_RE.findall(l) if n in names]


NEXT_ASKS_RE = re.compile(r"^// (copies):")


def next_asks_refused(root: Path) -> list[str]:
    """Conformance cases the new pipeline's harness can refuse for a header it does not act on
    yet: `// copies:`, as ownership reports no copy count (tests/programs.rs; `judge:`,
    `before:` and `timeout:` are judged). 0 is every header judged."""
    out = []
    for r in _git(root, "ls-files", "spec/conformance/*.cp", NEXT_TESTS + "/programs/*.cp"):
        lines = _lines(root / r) if (root / r).is_file() else []
        if any(NEXT_ASKS_RE.match(l) for l in itertools.takewhile(lambda l: l.startswith("//"), lines)):
            out.append(r)
    return out


# `cpl-syntax` is counted on its own, by `next_syntax_old_refs`.
# `cpl-prim` is the spec's primitive set (R12.14): ids and numbers, no tool or build fact.
NEXT_LAYER_MAY_USE = ("cpl-syntax", "cpl-diag", "cpl-config", "cpl-contract", "cpl-prim")


def next_layer_foreign_deps(root: Path) -> list[str]:
    """Dependencies of any cpl-next crate (each layer, the driver, the engines and C++) on a
    crate that is neither one of them nor the diagnostics and config they are built on: a tool
    or build fact (a judge profile, the environment, JSON) decided inside the compiler instead of
    arriving in the snapshot (contract 0). Hard zero at the switch (audit S21)."""
    out = []
    for toml in sorted((root / "crates").glob("cpl-next*/Cargo.toml")):
        c = toml.parent.name
        deps = tomllib.loads(toml.read_text(encoding="utf-8")).get("dependencies", {})
        out += [f"{c} -> {d}" for d in sorted(deps) if not d.startswith("cpl-next-") and d.startswith("cpl-")
                and d not in NEXT_LAYER_MAY_USE]
    return out


OLD_SYNTAX_USE_RE = re.compile(r"\buse\s+(?:::)?cpl_syntax\b[^;]*;", re.S)
OLD_SYNTAX_ANALYSES = ("decorators", "fall_through", "visit", "narrow", "const_int", "order", "planned")


def _use_tree(text: str, path: tuple[str, ...]) -> list[tuple[str, tuple[str, ...]]]:
    """The names a `use` tree brings in, each with the path it names: `ast::{self, Expr as E}`
    under `cpl_syntax` is `ast` (cpl_syntax, ast) and `E` (cpl_syntax, ast, Expr)."""
    text = text.strip()
    if text.endswith("}") and "{" in text:
        head, body = text[:text.index("{")], text[text.index("{") + 1:-1]
        prefix = path + tuple(x for x in head.split("::") if x.strip())
        parts, depth, cur = [], 0, ""
        for c in body:
            depth += (c == "{") - (c == "}")
            if c == "," and depth == 0:
                parts.append(cur)
                cur = ""
            else:
                cur += c
        parts.append(cur)
        return [n for part in parts if part.strip() for n in _use_tree(part, prefix)]
    name, _, alias = text.partition(" as ")
    full = path + tuple(x.strip() for x in name.split("::") if x.strip())
    if full[-1] == "self":
        full = full[:-1]
    if full[-1] == "*":
        return []
    return [(alias.strip() or full[-1], full)]


def _old_syntax_uses(root: Path, files, only: tuple[str, ...] = ()) -> list[str]:
    """Each use of the old syntax crate in `files`: every `cpl_syntax` token, and every use of a
    name a `use cpl_syntax::..` brings in (a module, type, fn or alias), so a grouped import
    counts each use behind it. `only` keeps the uses that reach those modules of it."""
    out = []
    for p in files:
        text = p.read_text(encoding="utf-8")
        code = re.sub(r"//[^\n]*", lambda m: " " * len(m.group(0)), text)
        names, spans = {}, []
        for m in OLD_SYNTAX_USE_RE.finditer(code):
            spans.append((m.start(), m.end()))
            tree = re.sub(r"\s+", " ", m.group(0)[m.group(0).index("cpl_syntax"):-1])
            for name, full in _use_tree(tree, ()):
                if name != "cpl_syntax":
                    names[name] = full
        if only:
            names = {n: f for n, f in names.items() if len(f) > 1 and f[1] in only}
        words = [re.escape(n) for n in names]
        alts = ["cpl_syntax::(?:%s)\\b" % "|".join(only)] if only else ["cpl_syntax"]
        pat = re.compile(r"\b(?:%s)" % "|".join(alts + ([r"(?:%s)\b" % "|".join(words)] if words else [])))
        starts = [0] + [i + 1 for i, c in enumerate(code) if c == "\n"]
        for m in pat.finditer(code):
            if any(a <= m.start() < b for a, b in spans) and not m.group(0).startswith("cpl_syntax"):
                continue
            line = bisect.bisect_right(starts, m.start())
            out.append(f"{_rel(root, p)}:{line}:{m.start() - starts[line - 1] + 1} {m.group(0)}")
    return out


def _next_old_syntax_files(root: Path):
    return [p for d in sorted((root / "crates").glob("cpl-next*")) if d.name != "cpl-next-syntax"
            for p in _rs_files(root, _rel(root, d))]


FACADE_TYPES = ("Session", "Snapshot")


def _facade_queries(files: list[Path]) -> set[str]:
    """Each `pub fn` signature in an `impl Session`/`impl Snapshot` block, whitespace folded."""
    out = set()
    for f in files:
        ty, sig = None, None
        for line in f.read_text(encoding="utf-8").splitlines():
            if m := re.match(r"impl (\w+) \{", line):
                ty = m[1] if m[1] in FACADE_TYPES else None
            elif line == "}":
                ty = None
            elif ty and sig is None and line.startswith("    pub fn "):
                sig = ""
            if ty and sig is not None:
                sig += " " + line
                if line.endswith("{") or line.lstrip().startswith("where"):
                    out.add(f"{ty}::" + re.sub(r"\s+", " ", sig.split(" where")[0].rstrip("{")).strip())
                    sig = None
    return out


def next_facade_split(root: Path) -> list[str]:
    """Contract 9 has one API (audit S9): each `Session`/`Snapshot` query of cpl-facade with no
    identical signature on cpl-next's facade, each cpl-next query of the same name with
    another signature, and each second `pub struct Session` or `Snapshot` outside cpl-facade.
    A cpl-next query with no old-side half is the one facade's own, not a split (coordinator
    ruling on audit #14's S16). Hard zero at the switch, when cpl-next's facade is
    cpl-facade's implementation."""
    old = _facade_queries(sorted((root / "crates/cpl-facade/src").glob("*.rs")))
    new_file = root / "crates/cpl-next/src/facade.rs"
    new = _facade_queries([new_file] if new_file.is_file() else [])
    def named(q: str) -> tuple[str, str] | str:
        m = re.match(r"(\w+::)pub fn (\w+)", q)
        return m.group(1, 2) if m else q
    old_names = {named(q) for q in old}
    out = [f"cpl-facade {q}" for q in sorted(old - new)] + \
        [f"cpl-next {q}" for q in sorted(new - old) if named(q) in old_names]
    for rs in sorted((root / "crates").glob("*/src/**/*.rs")):
        rel = rs.relative_to(root).as_posix()
        if rel.startswith("crates/cpl-facade/"):
            continue
        for n, line in enumerate(rs.read_text(encoding="utf-8").splitlines(), 1):
            if re.match(r"pub struct (Session|Snapshot)\b", line):
                out.append(f"{rel}:{n}")
    return out


NEXT_RUN_CONTRACTS = ("00-snapshot.md", "07-execution.md", "08-events.md")


def _members(text: str, kinds: str = "struct|enum") -> dict[str, set[str]]:
    """Each `struct`/`enum` body's top-level member names: a field before `:`, or a bare
    name before `,`, `{`, `(` or the end (an enum's variant, a field the contract leaves
    untyped). Comments are dropped first."""
    text = re.sub(r"/\*.*?\*/|//[^\n]*", "", text, flags=re.S)
    out: dict[str, set[str]] = {}
    for m in re.finditer(rf"\b(?:{kinds})\s+(\w+)(?:<[^>{{]*>)?\s*\{{", text):
        depth, i, body = 1, m.end(), []
        while i < len(text) and depth:
            c = text[i]
            depth += (c in "{(<[") - (c in "})>]")
            body.append(c if depth == 1 else (" " if c not in "{(" else c))
            i += 1
        names = re.findall(r"(?:^|[,|{])\s*(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_]\w*)\s*(?=[:,{(|]|$)",
                           "".join(body[:-1]))
        out.setdefault(m[1], set()).update(n for n in names if n not in ("pub",))
    return out


def next_run_api_unbuilt(root: Path) -> list[str]:
    """Audit S10: each field or variant of a type in contracts 0, 7 and 8 (the run input,
    limits, entry roots, I/O, outcome and events) with no member of that name in a
    same-named cpl-next type. Hard zero at the switch."""
    built: dict[str, set[str]] = {}
    for rs in sorted((root / "crates").glob("cpl-next*/src/**/*.rs")):
        for ty, ms in _members(rs.read_text(encoding="utf-8")).items():
            built.setdefault(ty, set()).update(ms)
    out = []
    for doc in NEXT_RUN_CONTRACTS:
        path = root / "handbook/design/next/contracts" / doc
        if not path.is_file():
            continue
        code = "\n".join(re.findall(r"```rust\n(.*?)```", path.read_text(encoding="utf-8"), flags=re.S))
        for ty, ms in sorted(_members(code).items()):
            have = {m.lower() for m in built.get(ty, set())}
            out += [f"{doc} {ty}.{m}" for m in sorted(ms) if m.lower() not in have]
    return out


def next_syntax_old_refs(root: Path) -> list[str]:
    """Each use of the old parser's crate in the new pipeline: every `cpl_syntax` token and
    every use of a name imported from it (`ast::Expr`, `Expr` after `use cpl_syntax::ast::Expr`,
    `visit::`, `Mark::`, an alias) in a `cpl-next*` crate, tests included, and each
    `cpl-syntax` dependency in its manifest. Each reader moves onto `cpl-next-syntax`'s lossless
    tree and deletes its old uses (handbook/design/next/syntax.md); at 0, `cpl-syntax` serves only
    the old pipeline it is deleted with."""
    out = []
    for d in sorted((root / "crates").glob("cpl-next*")):
        toml = d / "Cargo.toml"
        if toml.is_file():
            parsed = tomllib.loads(toml.read_text(encoding="utf-8"))
            out += [f"{_rel(root, toml)}: {kind} {dep}" for kind in TOOL_COMPILER_KINDS
                    for dep in sorted(parsed.get(kind, {})) if dep == "cpl-syntax"]
    return out + _old_syntax_uses(root, _next_old_syntax_files(root))


def next_old_syntax_analyses(root: Path) -> list[str]:
    """Uses in the new pipeline of the semantic decisions that live in the old syntax crate:
    its decorator set, fall-through analysis, mention and pattern-name walks, narrowing, constant
    folding, ordering and planned features (OLD_SYNTAX_ANALYSES). Each moves into the layer that
    owns it (the decorator set into the declarations' role table, fall-through onto the typed
    tree), not into `cpl-next-syntax`; at 0 none of them dies with `cpl-syntax`."""
    return _old_syntax_uses(root, _next_old_syntax_files(root), OLD_SYNTAX_ANALYSES)


def tool_old_syntax_uses(root: Path) -> list[str]:
    """Each use of the old syntax crate in a `tool` crate (architecture.md's crate rules):
    lints, the formatter and the editor reading today's tree, which `tool_compiler_deps` leaves
    out on purpose. They move onto the lossless tree in migration stage 8; every lint added on
    the old tree before then is one more to port."""
    tools = crate_rules(root).get("tool", [])
    files = [p for t in tools if (root / "crates" / t).is_dir() for p in _rs_files(root, f"crates/{t}")]
    return _old_syntax_uses(root, files)


def wrong_to_refused(root: Path, ref: str) -> list[str]:
    """Cases `ref` lists wrong that this tree lists neither wrong nor passing: a wrong verdict
    turned into a refusal, which lowers `next_wrong` without fixing the case. Each is refused
    unless a numbered decisions.md decision names it (`case`), saying why the refusal is the honest verdict."""
    def ids(where: str, name: str) -> set[str]:
        lines = _git(root, "show", f"{where}:{NEXT_TESTS}/{name}") if where else \
            _lines(root / NEXT_TESTS / name) if (root / NEXT_TESTS / name).is_file() else []
        return {l.split("\t")[0].strip() for l in lines if l.strip()}
    was_wrong = ids(ref, "wrong.tsv")
    if not was_wrong:
        return []
    now = ids("", "wrong.tsv") | ids("", "passing.txt")
    cases = {str(p.relative_to(root / "spec" / "conformance")) for p in (root / "spec" / "conformance").rglob("*.cp")}
    cases |= {"next/" + p.name for p in (root / NEXT_TESTS / "programs").glob("*.cp")}
    text = (root / "decisions.md").read_text(encoding="utf-8") if (root / "decisions.md").is_file() else ""
    # Only a numbered decision (`## N.`) names a case: a mention anywhere else is not a ruling.
    named = "".join(re.split(r"(?m)^(?=## \d+\.)", text)[1:]) if re.search(r"(?m)^## \d+\.", text) else ""
    return sorted(c for c in was_wrong - now if c in cases and f"`{c}`" not in named)


def regressed_named(root: Path, ref: str) -> set[str]:
    """Case ids a list-regeneration commit since `ref` names in a `Regressed: <case> <sha>`
    trailer, with `sha` already on `ref`: a run that finds a regression records it, so the
    lists never hold one back. A commit touching anything but the lists (and plan/END-STATE.md,
    which `scripts/end-state.py` writes from them) names nothing."""
    lists = {f"{NEXT_TESTS}/{n}" for n in ("passing.txt", "wrong.tsv", "masked.txt", "lowered.txt", "std-unchecked.tsv")}
    lists.add("plan/END-STATE.md")
    base = (_git(root, "merge-base", ref, "HEAD") or [ref])[0]
    given = []
    for sha in _git(root, "rev-list", f"{base}..HEAD"):
        files = set(_git(root, "diff-tree", "--no-commit-id", "--name-only", "-r", sha))
        if files and files <= lists:
            given += _git(root, "log", "-1", "--format=%(trailers:key=Regressed,valueonly,separator=%x0a)", sha)
    # The commit being made: pre-commit runs before its message exists, so CPL_REGRESSED names
    # its trailers (commit-msg writes them in) when what it stages is only the lists.
    staged = set(_git(root, "diff", "--cached", "--name-only"))
    if staged and staged <= lists:
        given += os.environ.get("CPL_REGRESSED", "").replace(";", "\n").splitlines()
    named = set()
    for t in given:
        case, _, cause = t.strip().partition(" ")
        if case and cause.strip() and subprocess.run(["git", "merge-base", "--is-ancestor", cause.strip(), ref], cwd=root,
                                                     env=_git_env(), capture_output=True).returncode == 0:
            named.add(case if case.endswith(".cp") else case + ".cp")
    return named


def set_swaps(root: Path, ref: str) -> list[str]:
    """Case ids new to a bad set (`next_wrong`, `next_masked`, `next_wrong_accepts`) that `ref`
    had in a better one: a count holds when one case is fixed and another breaks, so each set is
    compared by id, and a case leaving `passing.txt` for any bucket is one, read from a fresh run
    when there is one (`_run_list_path`), so a lost pass the lists never recorded still fails;
    the run gives a listed pass that timed out under load one run alone. A wrong verdict on a
    case `ref` refused is an exposure (`unhidden_wrong`), a
    case `ref` lacked is a case added, and neither is a swap; a new wrong-accept always is. A
    regeneration's `Regressed:` trailer admits exactly the cases it names (`regressed_named`)."""
    def rows(where: str, name: str) -> dict[str, str]:
        here = _run_list_path(root, name)
        lines = _git(root, "show", f"{where}:{NEXT_TESTS}/{name}") if where else \
            _lines(here) if here.is_file() else []
        return {l.split("\t")[0].strip(): (l.split("\t") + [""])[1] for l in lines if l.strip()}
    was_wrong, was_passing, was_masked = rows(ref, "wrong.tsv"), rows(ref, "passing.txt"), rows(ref, "masked.txt")
    if not (was_wrong or was_passing):
        return []
    wrong, masked, now_passing = rows("", "wrong.tsv"), rows("", "masked.txt"), rows("", "passing.txt")
    out = [(c, f"passes on {ref}, wrong here ({v})") for c, v in wrong.items()
           if c in was_passing and c not in now_passing]
    out += [(c, f"{'passes' if c in was_passing else 'wrong'} on {ref}, masked here") for c in masked
            if c not in was_masked and (c in was_passing or c in was_wrong)]
    out += [(c, f"a wrong-accept {ref} does not have") for c, v in wrong.items()
            if v == "wrong-accept" and was_wrong.get(c) != "wrong-accept" and c not in was_passing]
    def exists(c: str) -> bool:
        return (root / "spec/conformance" / c).is_file() or (root / NEXT_TESTS / "programs" / c.removeprefix("next/")).is_file()
    out += [(c, f"passes on {ref}, refused here") for c in was_passing
            if c not in now_passing and c not in wrong and c not in masked and exists(c)]
    named = regressed_named(root, ref) if out else set()
    return sorted(f"{c}: {m}; a list regeneration may name it in `Regressed: <case> <sha>`"
                  for c, m in out if c not in named)

def next_not_passing(root: Path) -> list[str]:
    """Cases the new pipeline does not pass yet: wrong ones and the ones it refuses (says it
    does not implement). 0 is the day `cpl` switches to it and what it replaces is deleted.
    `tests/programs.rs` keeps `passing.txt` equal to what passes."""
    listed = root / NEXT_TESTS / "passing.txt"
    if not listed.is_file():
        return []
    passing = {l.strip() for l in _lines(listed)}
    cases = [r[len("spec/conformance/"):] for r in _git(root, "ls-files", "spec/conformance/*.cp")]
    cases += ["next/" + r.rsplit("/", 1)[-1] for r in _git(root, "ls-files", NEXT_TESTS + "/programs/*.cp")]
    accepted = set(_old_parser_accepts(root))
    return [c for c in cases if c not in passing and c not in accepted]


def next_refused(root: Path) -> list[str]:
    """Cases the new pipeline refuses: not passing, and on neither `wrong.tsv` nor `masked.txt`.
    Gated apart from `next_not_passing`, so lifting one refusal never pays for another."""
    def ids(name: str) -> set[str]:
        path = root / NEXT_TESTS / name
        return {l.split("\t")[0].strip() for l in _lines(path) if l.strip()} if path.is_file() else set()
    answered = ids("wrong.tsv") | ids("masked.txt")
    return [c for c in next_not_passing(root) if c not in answered]


def own_databases(root: Path) -> list[str]:
    """A `Database` built outside cpl-check, tests included: only a `cpl_check::Session`
    builds an analysis, so no tool keeps a front end that could disagree with the checker."""
    out = []
    for p in _rs_files(root, "crates", tests=True):
        rel = _rel(root, p)
        if rel.startswith("crates/cpl-check/"):
            continue
        out += [f"{rel}:{i}" for i, line in enumerate(_lines(p), 1) if OWN_DATABASE_RE.search(line)]
    return out


IR_REINFER_RE = re.compile(r"\.probe_expr(?:_ty)?\(")


def ir_reinfer(root: Path) -> list[str]:
    """Lowering lowering an expression only to learn its type: `probe_expr` calls in
    cpl-ir. The checker's facts already say it (`Facts::type_of`)."""
    out = []
    for p in _rs_files(root, "crates/cpl-ir/src", tests=False):
        rel = _rel(root, p)
        out += [f"{rel}:{i}" for i, line in enumerate(_lines(p), 1) if IR_REINFER_RE.search(line)]
    return out


NEXT_STD_DECL_RE = re.compile(r"^\s*(?:pub\s+)?(?:fn|struct|trait|enum|type)\s+([A-Za-z_]\w*)", re.M)
RUST_STR_RE = re.compile(r'"([A-Za-z_]\w*)"')
NEXT_STD_DECORATOR_RE = re.compile(r"^\s*@([A-Za-z_]\w*)", re.M)
NEXT_STD_MODULE_RE = re.compile(r'"(std(?:\.[A-Za-z_]\w*)+)"')


def _next_srcs(root: Path) -> list[str]:
    """The pipeline and the per-layer crates it is split into (`cpl-next-syntax`, `cpl-next-ir`,
    `cpl-next-engine`)."""
    return sorted(_rel(root, d / "src") for d in (root / "crates").glob("cpl-next*") if (d / "src").is_dir())


# The grammar's one spelling table: keywords and contextual words (`by`, `max`) are spelled
# there and nowhere else, so it alone is not a name the compiler knows a std declaration by.
NEXT_GRAMMAR_TABLE = "crates/cpl-next-syntax/src/grammar.rs"


def next_std_names(root: Path) -> list[str]:
    """Places where the new pipeline (cpl-next and its `cpl-next-*` crates) knows a library
    declaration by its name: a string literal in its compiler code equal to a name its std declares (`"List"`, `"get"`,
    `"read"`), and every use of a lookup that wraps one: a `const` holding such a name, or a fn of
    at most NEXT_WRAPPER_LINES lines whose body names one (`list_struct()`, `is_str(t)`). The design
    has no privileged std: the library says what a declaration is (a marker or a trait), and the
    compiler reads that, so this reaches 0."""
    std = root / "crates/cpl-next/std.cp"
    sources = [*([std] if std.is_file() else []), *sorted((root / "std").glob("**/*.cp"))]
    names = {"print", "write", "read"}
    for f in sources:
        text = f.read_text(encoding="utf-8")
        # Decorators std declares or uses (`@derive`) are its names as much as its decls.
        names |= set(NEXT_STD_DECL_RE.findall(text)) | set(NEXT_STD_DECORATOR_RE.findall(text))
    # `cpl-prim` holds the `@intrinsic` ids the IR maps to its operations, so it is scanned too.
    files = {_rel(root, p): _lines(p) for p in _rs_files(root, *_next_srcs(root), "crates/cpl-prim/src", tests=False)
             if _rel(root, p) != NEXT_GRAMMAR_TABLE}
    # An id table's row binds an id std writes as `@intrinsic("id")` or `@fast(id)`; a row std writes no
    # binding for is a name the compiler knows, and counts.
    ids = {a or b for f in sources for a, b in NEXT_INTRINSIC_ID_RE.findall(f.read_text(encoding="utf-8"))}
    # The check-name table's rows are R09.3's names, read from the spec itself.
    bound = {rel: _spec_check_names(root) if rel == NEXT_CHECK_NAMES[0] else ids for rel in NEXT_INTRINSIC_TABLES}
    out: list[str] = []
    wrappers: dict[str, tuple[str, int]] = {}
    for rel, lines in files.items():
        table = None
        for i, line in _ungated_lines(lines):
            if line.lstrip().startswith("//"):
                continue
            if (t := NEXT_INTRINSIC_TABLES.get(rel)) and re.match(rf"\s*pub const {t}\b", line):
                table = t
            hits = [n for n in RUST_STR_RE.findall(line) if n in names and not (table and n in bound[rel])]
            if table and line.strip().startswith("];"):
                table = None
            # A display arm naming its own variant (`Stage::Check => "check"`) is the compiler's label.
            if (arm := NEXT_LABEL_ARM_RE.match(line)) and hits == [arm.group(1).lower()]:
                hits = []
            # A query's name (`const NAME: &'static str = "parse";`) labels its recompute count.
            if NEXT_QUERY_NAME_RE.match(line):
                hits = []
            out += [f"{rel}:{i}: {n}" for n in hits]
            # A std module by its dotted name (`"std.prelude"`).
            out += [f"{rel}:{i}: {n}" for n in NEXT_STD_MODULE_RE.findall(line)]
            # A name matched by its start (`starts_with("Tuple")`) or built (`format!("Tuple{}")`).
            parts = ("".join(t) for t in NEXT_NAME_PART_RE.findall(line))
            out += [f"{rel}:{i}: {n}*" for n in parts if len(n) > 2 and n not in names and any(m.startswith(n) for m in names)]
            if hits:
                const = NEXT_CONST_RE.match(line)
                w = const.group(1) if const else _short_fn_around(lines, i)
                if w:
                    wrappers.setdefault(w, (rel, i))
    for rel, lines in files.items():
        for i, line in enumerate(lines, 1):
            if line.lstrip().startswith("//"):
                continue
            for w, at in wrappers.items():
                if at == (rel, i) or ((d := NEXT_FN_RE.match(line)) and d.group(2) == w):
                    continue
                use = rf"\b{w}\b" if w.isupper() else rf"\b{w}\("
                out += [f"{rel}:{i}: {w}()"] * len(re.findall(use, line))
    return out


NEXT_WRAPPER_LINES = 8
# An id std binds: a body `= @intrinsic("id")`, or `@fast(id)` on a CP body the op computes.
NEXT_INTRINSIC_ID_RE = re.compile(r'@intrinsic\("(\w+)"\)|@fast\((\w+)\)')
# The one table of `@intrinsic` ids (in build.rs, of R09.3 check names) each file keeps, by its `const`'s name.
NEXT_CHECK_NAMES = ("crates/cpl-next-ir/src/build.rs", "ALL")
NEXT_INTRINSIC_TABLES = {"crates/cpl-next-ir/src/ops.rs": "INTRINSICS", "crates/cpl-prim/src/lib.rs": "IDS",
                         NEXT_CHECK_NAMES[0]: NEXT_CHECK_NAMES[1]}
NEXT_CHECK_NAMES_SPEC = "spec/core/09-contexts-checks.md"
NEXT_CHECK_NAMES_RE = re.compile(r"Each row has one check name(.*?)This is the one list", re.S)


def _spec_check_names(root: Path) -> set[str]:
    """R09.3's check names (`bounds`, `div`, ..), the words `with checks(..)` takes."""
    spec = root / NEXT_CHECK_NAMES_SPEC
    m = NEXT_CHECK_NAMES_RE.search(spec.read_text(encoding="utf-8")) if spec.is_file() else None
    return set(re.findall(r"`(\w+)`", m.group(1))) if m else set()

NEXT_QUERY_NAME_RE = re.compile(r'^\s*const NAME: &\'static str = "[a-z_]+";\s*$')
NEXT_LABEL_ARM_RE = re.compile(r'^\s*[A-Z]\w*::([A-Z]\w*)\s*=>\s*"[a-z_]+",?\s*$')
NEXT_FN_RE = re.compile(r"^(\s*)(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?fn\s+([a-z_]\w*)")
NEXT_CONST_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+([A-Z_][A-Z0-9_]*)\s*:")


def _short_fn_around(lines: list[str], i: int) -> str | None:
    """The fn line `i` (1-based) is in, when that fn is at most NEXT_WRAPPER_LINES lines long."""
    for j in range(i - 1, -1, -1):
        m = NEXT_FN_RE.match(lines[j])
        if not m:
            continue
        close = m.group(1) + "}"
        end = next((k for k in range(j, len(lines)) if lines[k] == close or (k == j and lines[k].rstrip().endswith("}"))), None)
        if end is not None and end + 1 >= i and end - j + 1 <= NEXT_WRAPPER_LINES:
            return m.group(2)
        return None
    return None


NEXT_UNBUILT = "crates/cpl-next-ir/src/unbuilt.rs"
NEXT_REFUSAL_RE = re.compile(r'"slice: ((?:[^"\\]|\\.)*)"')


def next_refusal_kinds(root: Path) -> list[str]:
    """The kinds of feature the new pipeline (cpl-next and its `cpl-next-*` crates) refuses: the
    variants of `Unbuilt`, and any refusal still written as `slice:` text. A refusal says the spec
    has a feature this pipeline has not built, so the list only shrinks; a new kind is a visible raise."""
    out = []
    enum = root / NEXT_UNBUILT
    if enum.is_file():
        inside = False
        for i, line in enumerate(_lines(enum), 1):
            if line.startswith("pub enum Unbuilt"):
                inside = True
            elif inside and line.startswith("}"):
                break
            elif inside and (m := re.match(r"    ([A-Z]\w*)\b", line)):
                out.append(f"{NEXT_UNBUILT}:{i}: {m.group(1)}")
    for p in _rs_files(root, *_next_srcs(root), tests=False):
        rel = _rel(root, p)
        for i, line in enumerate(_lines(p), 1):
            if line.lstrip().startswith("//"):
                continue
            out += [f"{rel}:{i}: slice: {t.strip()}" for t in NEXT_REFUSAL_RE.findall(line) if t.strip()]
    return sorted(out)


NEXT_NAME_PART_RE = re.compile(r'(?:starts_with|strip_prefix|ends_with|strip_suffix)\(\s*"([A-Za-z_]\w*)"|format!\(\s*"([A-Za-z_]\w*)\{')


def old_pipeline_lines(root: Path) -> list[str]:
    """Code lines in today's checker, lowering, interpreter and C++ emitter, which stage 5 deletes.
    They are frozen (migration rule 9): a fix goes to a conformance case the new pipeline must pass,
    so this only falls, and reaches 0 in the switching landing."""
    out = []
    for crate in OLD_PIPELINE_CRATES:
        for p in _rs_files(root, f"crates/{crate}/src"):
            rel = _rel(root, p)
            out += [f"{rel}:{i}" for i, l in enumerate(_lines(p), 1) if l.strip() and not l.lstrip().startswith("//")]
    return out


def old_std_copy_lines(root: Path) -> list[str]:
    """Code lines of `FROZEN_STD`, the pinned copy of std the old pipeline reads (7fd97e1e2).
    Apart from `old_pipeline_lines` so that the switch, which deletes it with the old crates,
    shows it going on its own (coordinator, 18:47)."""
    out = []
    for p in sorted((root / FROZEN_STD).rglob("*.cp")):
        rel = _rel(root, p)
        out += [f"{rel}:{i}" for i, l in enumerate(_lines(p), 1) if l.strip() and not l.lstrip().startswith("//")]
    return out


# The old pipeline: its checker, lowering, interpreter and C++ emitter, its parser (cpl-syntax,
# whose lexer is cpl-next-syntax's, the one shared lexer) and cpl-facade over them (audit #18
# (3)). All six are frozen, and old_pipeline_lines counts them.
OLD_PIPELINE_CRATES = ("cpl-check", "cpl-ir", "cpl-interp", "cpl-codegen", "cpl-syntax", "cpl-facade")
FROZEN_CRATES = OLD_PIPELINE_CRATES
# The std the old pipeline reads, a copy of std/ at 6a31a3747^ (migration rule 9): stage 5 deletes it.
FROZEN_STD = "crates/cpl-check/frozen/std"
# `git rev-parse 6a31a3747^:std`: the tree FROZEN_STD has to stay.
FROZEN_STD_TREE = "918845f0fb5e8d8e69b121774d22d5b10ff88bba"


def frozen_std_drift(root: Path, tree: str = FROZEN_STD_TREE) -> list[str]:
    """Files of the old pipeline's std that differ from std/ at the pin, staged or not:
    the copy is read-only, so a std change can't reach the frozen pipeline."""
    if not (root / FROZEN_STD).is_dir():
        return []
    index = _git(root, "write-tree")
    out = [f"{l[3:]}: {l[:2].strip()}" for l in
           _git(root, "status", "--porcelain", "--untracked-files=all", "--", FROZEN_STD) if l[1] != " "]
    if index and _git(root, "rev-parse", f"{index[0]}:{FROZEN_STD}") != [tree]:
        out += [f"{FROZEN_STD}/{l.split(chr(9))[-1]}: differs from the pin" for l in
                _git(root, "diff-tree", "-r", "--name-only", "--format=", tree, f"{index[0]}:{FROZEN_STD}")
                if l] or [f"{FROZEN_STD}: not the pinned tree {tree[:9]}"]
    return out




def old_parser_lines(root: Path) -> list[str]:
    """Code lines of `cpl-syntax`'s parser, which `cpl-next-syntax`'s parser replaces: its step 2
    turns `parse_file` and `parse_item` into a lowering of the lossless tree and deletes these in the
    same landing (handbook/design/next/syntax.md), so there is never a second parser."""
    files = [root / "crates/cpl-syntax/src/parser.rs", *sorted((root / "crates/cpl-syntax/src/parser").glob("*.rs"))]
    return [f"{_rel(root, p)}:{i}" for p in files if p.is_file()
            for i, l in enumerate(_lines(p), 1) if l.strip() and not l.lstrip().startswith("//")]


STRING_KEYED_MAP_RE = re.compile(r"HashMap<\s*(?:String|&(?:'\w+\s+)?str|Box<str>|Arc<str>)\s*,")


def string_keyed_maps(root: Path) -> list[str]:
    """A `HashMap` keyed by a name in cpl-check: each one builds the hash table's code
    again for its value type. `crate::names::NameMap<V>` shares one index for all of them."""
    out = []
    for p in _rs_files(root, "crates/cpl-check/src", tests=False):
        rel = _rel(root, p)
        if rel == "crates/cpl-check/src/names.rs":
            continue
        out += [f"{rel}:{i}" for i, line in enumerate(_lines(p), 1) if STRING_KEYED_MAP_RE.search(line)]
    return out


def stress_hand_json(root: Path) -> list[str]:
    """JSON assembled by hand in the stress engine and its browser entry; `cpl_json` values render it."""
    out = []
    for p in _rs_files(root, "crates/cpl-stress", "crates/cpl-wasm/src"):
        out += [f"{_rel(root, p)}:{i}" for i, line in enumerate(_lines(p), 1)
                if '{{\\"' in line or "escape_string(" in line]
    return out


# Variables the toolchain may hand a program it runs: stress seeds a built `@gen` fn.
HELPER_ENV_ALLOWED = {"CPL_SEED"}


def helper_env_vars(root: Path) -> list[str]:
    """`CPL_*` side channels to a compiled program or helper beyond `CPL_SEED`: one a generated runtime
    `getenv`s, or one stress or the judge sets on a program it runs. Helpers talk through stdin/stdout."""
    found: dict[str, str] = {}
    for p in _rs_files(root, tests=False):
        rel = _rel(root, p)
        helper = rel.startswith(("crates/cpl-stress/", "crates/cpl-judge/", "crates/cpl-cli/src/profile.rs"))
        for i, line in enumerate(_lines(p), 1):
            names = re.findall(r'getenv\(\\?"(CPL_[A-Z0-9_]+)', line)
            if helper:
                names += re.findall(r'(?:\.env\(|\(\(?)"(CPL_[A-Z0-9_]+)"(?:\.to_string\(\))?\s*,', line)
            for name in names:
                if name not in HELPER_ENV_ALLOWED:
                    found.setdefault(name, f"{name}: {rel}:{i}")
    return sorted(found.values())


def _header_counts(root: Path, key: str,
                   bases: tuple[str, ...] = ("corpus/problems", "spec/conformance")) -> list[str]:
    """The `// <key>: N` header lines of the programs `read_only_copies` builds (the
    `.cp` files one directory down from corpus/problems and spec/conformance), one
    item per unit: the test holds each program's C++ to its own header, this sums them."""
    out = []
    for base in bases:
        for path in sorted((root / base).glob("*/*.cp")):
            for line in path.read_text().splitlines():
                rest = line.strip()
                if not rest.startswith("//"):
                    break
                m = re.fullmatch(rf"//\s*{key}:\s*(\d+)\s*", rest)
                if m:
                    rel = path.relative_to(root)
                    out += [f"{rel} {i}" for i in range(int(m.group(1)))]
    return out


def cpp_deep_copies(root: Path) -> list[str]:
    """Deep copies in shipped programs (corpus ports, dreams and specs): each one is avoidable."""
    return _header_counts(root, "copies", ("corpus/problems",))


def spec_required_copies(root: Path) -> list[str]:
    """Deep copies a spec/conformance case exists to require (decision 458): shrink-only, so a
    header raised to hide a regression, or a new required copy, is a visible baseline bump."""
    return _header_counts(root, "copies", ("spec/conformance",))


def cpp_optional_copies(root: Path) -> list[str]:
    return _header_counts(root, "optional_copies")


def cpp_std_functions(root: Path) -> list[str]:
    return _header_counts(root, "std_functions")


def cpp_sunk_copies(root: Path) -> list[str]:
    return _header_counts(root, "sunk_copies")


def explain_copy_mismatches(root: Path) -> list[str]:
    """Ok conformance cases whose `cpl explain` lists more or fewer copies than their judge C++
    makes (owner: tools). Rule: costs are visible (spec/tools/explain.md): explain lists each
    copy the C++ makes, no more. The same check as `spec-gate.py explain-copies --exact`."""
    cases = sorted((root / "spec" / "conformance").rglob("*.cp"))
    if not cases:
        return []
    cpl = str(_built_cpl(root).resolve())
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import concurrent.futures
    from specgate import explain_copies
    # cpl embeds std, so its bytes, the counter's rules and a case's text decide the row:
    # land.sh's re-measures after each lost push race rerun a cpl they already ran.
    binary = hashlib.sha256(Path(cpl).read_bytes() + Path(explain_copies.__file__).read_bytes()).hexdigest()[:16]
    # In the common git dir, so the scratch worktree that measures main reads every land's rows.
    common = _git(root, "rev-parse", "--path-format=absolute", "--git-common-dir", env=_git_env())
    cache_path = (Path(common[0]) if common else root / "target") / "explain-copies-cache" / f"{binary}.json"
    try:
        cache = json.loads(cache_path.read_text())
    except (OSError, ValueError):
        cache = {}
    key = lambda c: hashlib.sha256(c.read_bytes()).hexdigest()[:16]

    def row(case):
        k = key(case)
        if k not in cache:
            r = explain_copies.one(cpl, case, tmp, True, root)
            cache[k] = r and [r[1], r[2]]
        return cache[k] and (str(case.relative_to(root)), *cache[k])

    with tempfile.TemporaryDirectory() as tmp, concurrent.futures.ThreadPoolExecutor(os.cpu_count() or 4) as pool:
        rows = [r for r in pool.map(row, cases) if r]
    if cache_path.parent.parent.is_dir():
        cache_path.parent.mkdir(exist_ok=True)
        cache_path.write_text(json.dumps(cache))
    return sorted(f"{path}: explain lists {claimed}, the C++ makes {made}"
                  for path, claimed, made in rows if claimed != made)


_RUNS_CODE = re.compile(r"ExprKind::(Call|CallExpr)\(\.\.\)|ExprKind::Closure\(_\)")
_EFFECTS_DIRECT = re.compile(r"cpl_ir::effects|walk::Effects\b|walk::\{[^}]*\bEffects\b")
_PURITY_FN = re.compile(r"\bfn ([a-z_]*_)?(pure|impure|trap_free|effects?|writes_(nothing|own)|may_touch|"
                        r"safe_place|quiet|runs_nothing|side_effects?)(_[a-z_]*)?\b")
_FLAG_SET = re.compile(r"[|&]=|= (true|false)\b")


def codegen_effect_guesses(root: Path) -> list[str]:
    """Codegen deciding what code can do outside the Uses table (crates/cpl-codegen/src/uses/):
    reaching cpl_ir::effects or walk::Effects directly, defining a purity helper, or a walk that
    flags any call or closure as "may do anything"."""
    out = []
    src = root / "crates/cpl-codegen/src"
    for p in _rs_files(root, "crates/cpl-codegen/src", tests=False):
        rel = _rel(root, p)
        if p.is_relative_to(src / "uses") or p.name.endswith("_tests.rs"):
            continue
        lines = _lines(p)
        for i, line in enumerate(lines):
            if line.lstrip().startswith("//"):
                continue
            if _EFFECTS_DIRECT.search(line) or _PURITY_FN.search(line):
                out.append(f"{rel}:{i + 1}")
                continue
            if not re.search(r"\bdeep_(expr|exprs|stmts)\(", line) or line.rstrip().endswith(");"):
                continue
            indent, body = len(line) - len(line.lstrip()), [line]
            for nxt in lines[i + 1:]:
                body.append(nxt)
                if nxt.strip().startswith("})") and len(nxt) - len(nxt.lstrip()) <= indent:
                    break
            if any(_RUNS_CODE.search(b) and _FLAG_SET.search(" ".join(body[k:k + 4]))
                   for k, b in enumerate(body)):
                out.append(f"{rel}:{i + 1}")
    return out


def stale_arch_paths(root: Path) -> list[str]:
    """Backticked repo paths in ARCHITECTURE docs, and the ratchet's own `crates/` paths,
    that do not exist (a gate reading a moved file would count 0).

    Skips gitignored build output and the "Known debt" section, which may
    name a missing file on purpose.
    """
    docs = [root / "handbook/design/architecture.md", root / "plan/rearchitecture.md",
            *sorted((root / "plan/domains").glob("*/ARCHITECTURE.md"))]
    found = []
    for d in (d for d in docs if d.is_file()):
        in_debt = False
        for i, line in enumerate(_lines(d), 1):
            if line.startswith("#"):
                in_debt = line.lstrip("#").strip().lower().startswith("known debt")
            if in_debt:
                continue
            for m in ARCH_PATH_RE.finditer(line):
                path = m.group(1).rstrip("/.")
                if "*" not in path and "<" not in path and not (root / path).exists():
                    found.append((f"{_rel(root, d)}:{i}", path))
    me = root / "scripts/debt-ratchet.py"
    for i, line in enumerate(_lines(me) if me.is_file() else [], 1):
        if '"deleted: ' in line:
            continue  # a LISTS row for a list that is gone on purpose
        for m in re.finditer(r'"(crates/[\w./-]+)"', line):
            path = m.group(1).rstrip("/.")
            if "*" not in path and not (root / path).exists():
                found.append((f"scripts/debt-ratchet.py:{i}", path))
    ignored = _git_ignored(root, sorted({p for _, p in found}))
    return [f"{where} {p}" for where, p in found if p not in ignored]


SKIP_NAME_RE = re.compile(r"SKIP|XFAIL|KNOWN|ALLOW|NOT_|DIFFER|PENDING|IGNORE|WORSE|EXEMPT|FAIL|GAP")
SKIP_CONST_RE = re.compile(
    r"^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:const|static)\s+([A-Z][A-Z0-9_]*)\s*:\s*&?(?:'static\s+)?\[[^=]*=\s*&?\[",
    re.M)
SKIP_FILE_RE = re.compile(r"(gaps|failing|allow|xfail|skip)[^/]*\.txt$")


def _array_items(text: str) -> int:
    """Top-level items of a Rust array literal whose `[` was just consumed."""
    depth, items, pending, i = 0, 0, False, 0
    while i < len(text):
        c = text[i]
        if text.startswith("//", i):
            i = text.find("\n", i) % (len(text) + 1)
            continue
        if c == '"':
            j = i + 1
            while j < len(text) and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            i, pending = j + 1, True
            continue
        if c in "([{":
            depth, pending = depth + 1, True
        elif c in ")]}":
            if depth == 0:
                return items + pending
            depth -= 1
        elif c == "," and depth == 0:
            items, pending = items + pending, False
        elif not c.isspace():
            pending = True
        i += 1
    return items + pending


def corpus_no_max(root: Path) -> list[str]:
    """Non-interactive problem packages with a port.cp but no `gen.py --max`.

    Rule: problem-packages/R6 (handbook/guidelines/problem-packages-checklist.md#r6):
    every package has an input at the statement's bounds, which `scripts/corpus.py
    verify` (`max`) judges; an interactive package's gen.py prints the interactor's data.
    """
    problems = root / "corpus" / "problems"
    found = []
    for d in sorted(problems.iterdir()) if problems.is_dir() else ():
        if not (d / "port.cp").is_file() or (d / "interactor.py").is_file():
            continue
        g = d / "gen.py"
        if not g.is_file() or "--max" not in g.read_text(encoding="utf-8", errors="replace"):
            found.append(d.relative_to(root).as_posix())
    return found


def corpus_no_limits(root: Path) -> list[str]:
    """Problem packages whose port.cp has no `// limits: T ms, M MB` header line (owner: Content).

    Rule: problem-packages/R6 (handbook/guidelines/problem-packages-checklist.md#r6): the
    `over-limit` gate and `cpl bench` judge a port against its statement's limits, so a port
    without the line is silently judged against the 2000 ms, 256 MB default.
    """
    problems = root / "corpus" / "problems"
    limits = re.compile(r"^//\s*limits:\s*\d+\s*ms,\s*\d+\s*MB", re.M)
    return [d.relative_to(root).as_posix() for d in (sorted(problems.iterdir()) if problems.is_dir() else ())
            if (d / "port.cp").is_file()
            and not limits.search((d / "port.cp").read_text(encoding="utf-8", errors="replace"))]


def corpus_no_tests(root: Path) -> list[str]:
    """Problem packages with no tests.toml (owner: Content, idioms).

    Rule: problem-packages/R9 (handbook/guidelines/problem-packages-checklist.md#r9):
    every package's cases are statement samples inline plus gen.py seeds and `--max`,
    made by `scripts/corpus.py make` with answers from the accepted C++.
    """
    problems = root / "corpus" / "problems"
    return [d.relative_to(root).as_posix() for d in sorted(problems.iterdir()) if problems.is_dir()
            and d.is_dir() and (d / "meta.json").is_file() and not (d / "tests.toml").is_file()] \
        if problems.is_dir() else []


class MeasureError(RuntimeError):
    """A counter that can't measure, say because the tool it runs isn't built: never a count."""


@functools.cache
def _corpus():
    import importlib.util
    spec = importlib.util.spec_from_file_location("corpus", Path(__file__).with_name("corpus.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _built_cpl(root: Path) -> Path:
    """`corpus.current_cpl`: a stale cpl measures the code it was built from, so with none
    current it raises, naming the build command, and never counts."""
    cpl, why = _corpus().current_cpl(root)
    if why and not os.environ.get("CPL") and not os.environ.get("CI"):
        # A stale cpl after a pull or a rebase is the usual case: an incremental build
        # (cargo's lock serialises the forked counters) brings it current.
        subprocess.run(["cargo", "build", "-q", "-p", "cpl-cli"], cwd=root, capture_output=True)
        cpl, why = _corpus().current_cpl(root)
    if why:
        raise MeasureError(f"cpl: {why}")
    return cpl


def corpus_fmt_dirty(root: Path) -> list[str]:
    """Corpus port.cp, dream.cp and spec.cp files `cpl fmt --check` would change (owner: Content, idioms).

    Rule: idiomatic-code/R9 (handbook/guidelines/idiomatic-code-checklist.md#r9): published code
    is in the canonical style; `scripts/corpus.py verify` (`lint`) reports each as `fmt-dirty`.
    Runs $CPL, else the newer of target/debug/cpl and target/release/cpl (`_built_cpl`).
    """
    problems = root / "corpus" / "problems"
    files = [f"corpus/problems/{d.name}/{n}" for d in (sorted(problems.iterdir()) if problems.is_dir() else ())
             for n in ("port.cp", "dream.cp", "spec.cp") if (d / n).is_file()]
    if not files:
        return []
    cpl = _built_cpl(root)
    r = subprocess.run([str(cpl.resolve()), "fmt", "--check", "--json", *files], cwd=root, capture_output=True, text=True)
    try:
        out = json.loads(r.stdout)["files"]
    except (ValueError, KeyError):
        raise MeasureError(f"{cpl}: `cpl fmt --check --json` printed no report: {r.stderr.strip()[-300:]}")
    return sorted(f["path"] for f in out if f["changed"] or f.get("diagnostics") or f.get("error"))


def corpus_no_statement(root: Path) -> list[str]:
    """Problem packages whose tests.toml has no statement sample inline (owner: Content, idioms).

    Rule: problem-packages/R9 (handbook/guidelines/problem-packages-checklist.md#r9): a
    package's cases start with its statement's samples as `input` (or `input_file`) cases. A package whose
    statement can't be found has none, and can't have valid bounds or tests either, so it
    is deleted (git history keeps it) rather than listed. A statement with no sample at all is
    recorded in meta.json as `"statement": {"url", "sha256", "samples": 0}`, pinning the
    document that was read, and is not listed.
    """
    import tomllib
    problems = root / "corpus" / "problems"
    found = []
    for d in sorted(problems.iterdir()) if problems.is_dir() else []:
        if not (d / "meta.json").is_file():
            continue
        t = d / "tests.toml"
        try:
            cases = tomllib.loads(t.read_text(encoding="utf-8")).get("case", []) if t.is_file() else []
        except tomllib.TOMLDecodeError:
            cases = []
        if not any("input" in c or "input_file" in c for c in cases) and not _no_sample_statement(d / "meta.json"):
            found.append(d.relative_to(root).as_posix())
    return found


def _no_sample_statement(meta: Path) -> bool:
    """meta.json pins a statement (url and sha256) that has no samples."""
    try:
        st = json.loads(meta.read_text(encoding="utf-8")).get("statement")
    except (OSError, ValueError, AttributeError):
        return False
    return (isinstance(st, dict) and bool(st.get("url")) and bool(st.get("sha256"))
            and type(st.get("samples")) is int and st["samples"] == 0)


def corpus_test_files(root: Path) -> list[str]:
    """Literal test data committed under corpus/problems/*/tests/, samples included.

    Rule: problem-packages/R9 (handbook/guidelines/problem-packages-checklist.md#r9):
    tests come from each problem's gen.py and tests.toml. `scripts/check.sh --commit`
    refuses a newly added file there.
    """
    return [f for f in _git(root, "ls-files", "corpus/problems") if "/tests/" in f]


def skip_lists(root: Path) -> list[str]:
    """Entries in lists that exempt something from a check.

    Rust `const`/`static` arrays named like SKIP, XFAIL, KNOWN_*, NOT_BUILT or
    *_DIFFERS, and tracked `*gaps*.txt`, `*failing*.txt`, `*allow*.txt`,
    `*xfail*.txt` and `*skip*.txt` files (one entry per non-comment line).
    """
    found = []
    for p in _rs_files(root):
        text = p.read_text(encoding="utf-8")
        for m in SKIP_CONST_RE.finditer(text):
            if SKIP_NAME_RE.search(m.group(1)):
                found += [f"{_rel(root, p)} {m.group(1)}"] * _array_items(text[m.end():])
    tracked = _git(root, "ls-files", "*.txt")
    for rel in (r for r in tracked if SKIP_FILE_RE.search(r) and not r.startswith("research/")):
        lines = [l for l in _lines(root / rel) if l.strip() and not l.lstrip().startswith("#")]
        found += [rel] * len(lines)
    return found


def bench_lists(root: Path) -> list[str]:
    """Entries in tracked `bench/**/*.txt` files: every such file exempts programs from a bench
    gate, so the bench gates hold only while there are none."""
    found = []
    for rel in _git(root, "ls-files", "bench/*.txt"):
        found += [rel] * sum(1 for l in _lines(root / rel) if l.strip() and not l.lstrip().startswith("#"))
    return found


REGISTER_SPAN_RE = re.compile(r"`([^`]+)`")
REGISTER_COMMANDS = {"cargo", "cpl", "cpl-conform", "just"}
REGISTER_CARGO_GATES = {"test", "nextest", "clippy", "fuzz", "mutants"}


def _register_command(root: Path, words: list[str]) -> tuple[bool, str | None]:
    """Whether a command span is a gate that can fail, and the span if it names a command that doesn't exist.

    Rule: handbook/process/quality.md#accountability. `cargo test`-like runs,
    `cpl-conform` suites, `just` recipes and `cpl ... --check` can fail; a
    command that only produces something (`cpl doc`, `cpl check`) is a
    feature, not its check.
    """
    tool, args = words[0], words[1:]
    sub = next((a for a in args if not a.startswith("-")), None)
    span = " ".join(words)
    if tool == "cargo":
        if not args or args[0] not in REGISTER_CARGO_GATES:
            return False, None
        for flag, exists in (("-p", lambda n: (root / "crates" / n / "Cargo.toml").exists()),
                             ("--test", lambda n: any(root.glob(f"crates/*/tests/{n}.rs")))):
            i = args.index(flag) + 1 if flag in args else 0
            if 0 < i < len(args) and not exists(args[i]):
                return False, span
        return True, None
    if tool == "cpl-conform":
        lib = root / "crates/cpl-conform/src/lib.rs"
        suites = set(re.findall(r'"([a-z][a-z-]*)"[^=\n]*=> Some\(Suite', lib.read_text(encoding="utf-8"))) \
            if lib.is_file() else set()
        return (True, None) if sub is None or sub in suites | {"all", "fast", "fuzz"} else (False, span)
    if tool == "just":
        jf = root / "justfile"
        recipes = set(re.findall(r"^([a-z][\w-]*)[^:\n=]*:", jf.read_text(encoding="utf-8"), re.M)) \
            if jf.is_file() else set()
        return (True, None) if sub in recipes else (False, span)
    lib = root / "crates/cpl-cli/src/lib.rs"
    subs = set(re.findall(r'"([a-z][a-z-]*)"\)?\s*(?:if [^\n]*)?=>', lib.read_text(encoding="utf-8"))) \
        if lib.is_file() else set()
    if sub is None:
        return False, None
    if sub not in subs:
        return False, span
    return "--check" in args, None


def _register_check(root: Path, check: str) -> tuple[bool, list[str]]:
    """Whether a register cell names a check, and the paths it names that are gone."""
    named, gone = False, []
    for span in REGISTER_SPAN_RE.findall(check):
        words = span.split()
        if CHECKLIST_ID_RE.fullmatch(span):
            continue
        if words[0] == "python3" and len(words) > 1:
            words = words[1:]
        elif words[0] in REGISTER_COMMANDS:
            gate, unknown = _register_command(root, words)
            named |= gate
            gone += [unknown] if unknown else []
            continue
        elif len(words) == 1 and words[0] in COUNTERS:
            named = True
            continue
        if ("/" not in words[0] or any(c in words[0] for c in "*<") or "..." in words[0]
                or words[0].startswith(".git/")):
            continue
        if "#" in words[0]:
            # A cited rule neither names the check nor goes missing, unless its anchor is gone.
            doc, anchor = words[0].split("#", 1)
            if anchor.lower() not in _anchors(root / doc) | _checklist_ids(root / doc):
                gone.append(words[0])
            continue
        path = words[0].rstrip("/")
        if (root / path).exists():
            named = True
        elif not _build_output(root, path):
            gone.append(path)
    return named, gone


def _build_output(root: Path, path: str) -> bool:
    """A gitignored or build-output path: absent from a fresh clone, so neither named nor gone."""
    return path.split("/")[0] == "target" or path in _git_ignored(root, [path])


def register_rows(root: Path) -> list[str]:
    """Quality-register rows in handbook/domains/*.md that name no existing check.

    A row's cells after the metric (mechanism, then check, where or status)
    name the test files, scripts, commands that can fail or CI jobs that check it, say
    `missing`, or start with "process" where no mechanism can decide. Rows marked `missing`, rows naming no check, and rows naming a
    path that doesn't exist count.
    """
    found = []
    for d in sorted((root / "handbook/domains").glob("*.md")):
        lines = _lines(d)
        for i, line in _register_lines(d):
            is_row = line.startswith("|") and not set(line) <= set("|-: ")
            header = i < len(lines) and set(lines[i].strip()) <= set("|-: ") and "-" in lines[i]
            if not is_row or header:
                continue
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            check, where = " | ".join(cells[2:]), f"{_rel(root, d)}:{i}"
            status = cells[-1].strip("` ").lower()
            if status.startswith("process") or (len(cells) > 2 and cells[2].lower().startswith("process")):
                continue
            named, gone = _register_check(root, check)
            if gone:
                found.append(f"{where} no such check {', '.join(gone)}: {cells[0]}")
            elif "`missing`" in check or status.startswith("missing") or not named:
                found.append(f"{where} missing: {cells[0]}")
    return found

GUIDELINE_CITE_RE = re.compile(r"handbook/([\w/-]+\.md)#([\w-]+)")
CHECKLIST_ID_RE = re.compile(r"\b([a-z][a-z0-9-]*)/(R\d+)\b")


def _anchors(path: Path) -> set[str]:
    """GitHub-style heading anchors of a markdown file."""
    out = set()
    for line in _lines(path) if path.is_file() else []:
        if line.startswith("#"):
            title = line.lstrip("#").strip().lower()
            out.add(re.sub(r"\s", "-", re.sub(r"[^\w\s-]", "", title)))
    return out


def _checklist_ids(path: Path) -> set[str]:
    """Rule ids (`r12`) a checklist defines at the start of a heading, list item or table row."""
    lines = _lines(path) if path.is_file() else []
    return {m.group(1).lower() for line in lines
            if (m := re.match(r"^\s*(?:#+|[-*]|\|)?\s*(?:\*\*|`)?(?:[a-z][a-z0-9-]*/)?(R\d+)\b", line))}


def _cites_guideline(root: Path, text: str) -> bool:
    """Whether text cites a handbook heading, a checklist rule's anchor, or a rule id like `std-api/R12`."""
    for m in GUIDELINE_CITE_RE.finditer(text):
        doc = root / "handbook" / m.group(1)
        if m.group(2) in _anchors(doc) or (doc.name.endswith("-checklist.md") and m.group(2) in _checklist_ids(doc)):
            return True
    return any(m.group(2).lower() in _checklist_ids(root / "handbook/guidelines" / f"{m.group(1)}-checklist.md")
               for m in CHECKLIST_ID_RE.finditer(text))


def uncited_rules(root: Path) -> list[str]:
    """Lints and gates that cite no handbook rule by heading anchor.

    A rule lives once, as an item in a handbook guideline or process doc; the
    lint (`spec/lints/<name>/lint.toml` or `explain.md`) or gate script (its
    first 40 lines) that enforces it cites `handbook/<doc>.md#<heading>`, a
    checklist rule's anchor (`handbook/guidelines/<topic>-checklist.md#r12`)
    or its id (`<topic>/R12`).
    """
    found = []
    for d in sorted((root / "spec/lints").glob("*/")):
        text = "".join((d / f).read_text(encoding="utf-8") for f in ("lint.toml", "explain.md") if (d / f).is_file())
        if not _cites_guideline(root, text):
            found.append(f"lint {d.name}")
    runners = [root / "scripts/check.sh", root / "scripts/ci.py"]
    gates = sorted({g for f in runners if f.is_file()
                    for g in re.findall(r"scripts/[\w-]+\.(?:py|sh|mjs)", f.read_text(encoding="utf-8"))})
    for g in gates:
        name = Path(g).name
        if name.startswith("test_") or name in ("check.sh", "ci.py", "gate-env.sh") or not (root / g).is_file():
            continue
        if not _cites_guideline(root, "\n".join(_lines(root / g)[:40])):
            found.append(f"gate {g}")
    return found


def _register_lines(path: Path) -> list[tuple[int, str]]:
    """(line number, line) of the Quality register section, subsections included."""
    out, level = [], 0
    for i, line in enumerate(_lines(path) if path.is_file() else [], 1):
        if line.startswith("#"):
            depth = len(line) - len(line.lstrip("#"))
            if line.lstrip("#").strip().lower() == "quality register":
                level = depth
                continue
            if level and depth <= level:
                level = 0
        if level:
            out.append((i, line))
    return out


def _register_attributes(root: Path, domain: str) -> set[str]:
    """First cells of the Quality register rows in handbook/domains/<domain>.md."""
    return {line.strip().strip("|").split("|")[0].strip()
            for _, line in _register_lines(root / "handbook/domains" / f"{domain}.md") if line.startswith("|")}


def escapes_open(root: Path) -> list[str]:
    """Escapes in plan/escapes.toml whose suite isn't fixed or whose row doesn't exist."""
    log = root / "plan/escapes.toml"
    if not log.is_file():
        return []
    import tomllib
    found = []
    for i, e in enumerate(tomllib.loads(log.read_text(encoding="utf-8")).get("escape", []), 1):
        what = f"#{i} {e.get('domain', '?')}: {e.get('bug', '?')}"
        if e.get("row") not in _register_attributes(root, e.get("domain", "")):
            found.append(f"{what} (no such register row: {e.get('row')!r})")
        elif not e.get("suite_fix"):
            found.append(f"{what} (suite not fixed)")
    return found


NON_TEST_RS_PARTS = {"tests", "benches", "examples"}


def _cargo_build_script(rel: str) -> bool:
    parts = Path(rel).parts
    return len(parts) == 3 and parts[0] == "crates" and parts[2] == "build.rs"


def _src_lines(root: Path, *subdirs: str) -> Iterator[tuple[str, int, str]]:
    """Non-test, non-comment Rust lines under `subdirs`: tests/, tests.rs, *_tests.rs,
    a crate's Cargo build script and each item a `#[cfg(test)]` gates are left out; the rest
    of the file counts (a module named `build.rs` under src/ counts like any other)."""
    for p in _rs_files(root, *subdirs, tests=False):
        rel = _rel(root, p)
        parts = set(Path(rel).parts[:-1])
        if parts & NON_TEST_RS_PARTS or _cargo_build_script(rel) or p.name.endswith("_tests.rs") or p.name.startswith("tests_") \
                or "/src/tests/" in rel:
            continue
        for i, line in _ungated_lines(_lines(p)):
            if not line.lstrip().startswith("//"):
                yield rel, i, line


RUST_LITERAL_RE = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'')


def _ungated_lines(lines: list[str]) -> Iterator[tuple[int, str]]:
    """(line number, line) for every line outside a `#[cfg(test)]` item: the attribute,
    the item's other attributes and the item through its closing brace or `;`."""
    gated = opened = False
    depth = 0
    for i, line in enumerate(lines, 1):
        s = line.strip()
        if not gated:
            if s == "#[cfg(test)]":
                gated, opened, depth = True, False, 0
            else:
                yield i, line
            continue
        if not opened and (not s or s.startswith(("#[", "//"))):
            continue
        code = RUST_LITERAL_RE.sub('""', line.split("//")[0] if "\"" not in line else line)
        depth += code.count("{") - code.count("}")
        opened = opened or "{" in code
        if (opened and depth <= 0) or (not opened and s.endswith(";")):
            gated = False


UNWRAP_RE = re.compile(r"\.unwrap\(\)|\.expect\(|\bpanic!\(|\bunreachable!\(")
# compiler-changes/R16: a crate's one stop for a broken invariant is `fn broken_invariant` in
# its `src/invariant.rs`; that fn's panic is the only one allowed there.
INVARIANT_FILE_RE = re.compile(r"^crates/[^/]+/src/invariant\.rs$")
BROKEN_CALL_RE = re.compile(r"\bbroken_invariant\(")
# What a renamed panic wrapper needs: to report its caller, or to never return.
WRAPPER_MARK_RE = re.compile(r"#\[track_caller\]|\)\s*->\s*!\s*(?:\{|where\b|$)")
WRAPPER_MAX_BODY = 8


def _crate_src(root: Path) -> dict[str, list[tuple[int, str]]]:
    files: dict[str, list[tuple[int, str]]] = defaultdict(list)
    for rel, i, line in _src_lines(root):
        files[rel].append((i, line))
    return files


def _broken_wrappers(rel: str, lines: list[tuple[int, str]]) -> Iterator[str]:
    """Calls to `broken_invariant` that forward a name instead of stating one, and small generic
    or `Option`/`Result`-taking fns calling it: wrappers that would hide their callers."""
    for k, (i, line) in enumerate(lines):
        for m in BROKEN_CALL_RE.finditer(line):
            rest = line[m.end():].strip() or (lines[k + 1][1].strip() if k + 1 < len(lines) else "")
            if not rest.startswith('"'):
                yield f"{rel}:{i}"
        fn = FN_RE.match(line)
        if not fn:
            continue
        sig, j = line, k
        while "{" not in lines[j][1] and j + 1 < len(lines) and not lines[j][1].rstrip().endswith(";"):
            j += 1
            sig += lines[j][1]
        end = j + 1
        while end < len(lines) and not (lines[end][1].strip() == "}" and _indent(lines[end][1]) == len(fn.group(1))):
            end += 1
        body = [b for _, b in lines[j + 1:end]]
        params = sig.split("->")[0]
        if (len(body) <= WRAPPER_MAX_BODY and any(BROKEN_CALL_RE.search(b) for b in body)
                and (re.search(r"\bfn\s+\w+\s*<", sig) or re.search(r"\b(?:Option|Result)<", params))):
            yield f"{rel}:{i}"


def unwrap_input(root: Path) -> list[str]:
    """`.unwrap()`, `.expect(`, `panic!(`, `unreachable!(` in shipped crate code, and the
    wrappers that would rename them (handbook/guidelines/compiler-changes-checklist.md#compiler-changes-r16):
    input errors become diagnostics; an invariant stops through its crate's one
    `invariant::broken_invariant`, whose own panic is the one not counted, and `invariant_sites`
    counts its calls. A `#[track_caller]` or `-> !` fn elsewhere, a `broken_invariant` call
    whose argument is not a literal, and a small generic or `Option`/`Result`-taking
    fn that calls `broken_invariant` each count."""
    out = []
    for rel, lines in _crate_src(root).items():
        found = [f"{rel}:{i}" for i, line in lines for _ in UNWRAP_RE.finditer(line)]
        if INVARIANT_FILE_RE.match(rel):
            out += found[1:]
            continue
        out += found
        out += [f"{rel}:{i}" for i, line in lines if WRAPPER_MARK_RE.search(line)]
        out += _broken_wrappers(rel, lines)
    return sorted(out, key=lambda x: (x.rsplit(":", 1)[0], int(x.rsplit(":", 1)[1])))


def invariant_sites(root: Path) -> list[str]:
    """Calls to a crate's `invariant::broken_invariant` (compiler-changes/R16): facts the code
    trusts but its types don't prove yet. Each one goes when a type makes it unbreakable."""
    return [f"{rel}:{i}" for rel, lines in _crate_src(root).items() if not INVARIANT_FILE_RE.match(rel)
            for i, line in lines for _ in BROKEN_CALL_RE.finditer(line)]


def unsupported_sites(root: Path) -> list[str]:
    """Lowering's `unsupported(` calls and `"E0501"` literals: programs the backend refuses."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root, "crates/cpl-ir/src")
            for _ in re.finditer(r"\bunsupported\(|\"E0501\"", line)]


LSP_STRING_TYPE_RE = re.compile(
    r'(?:starts_with|strip_prefix|strip_suffix|ends_with|split_once|rsplit_once|find)\("(?:fn |->|: |cost|O\()'
    r'|"cost: O\(')


def lsp_string_types(root: Path) -> list[str]:
    """LSP lines reading types or costs back out of rendered text instead of checker facts."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root, "crates/cpl-lsp/src")
            if LSP_STRING_TYPE_RE.search(line)]


# Tables in cpl-lint that patch findings after the rules ran (owner: lint-fmt).
LINT_POST_FILTERS = {
    "crates/cpl-lint/src/typed.rs": ("TYPED",),
    "crates/cpl-lint/src/corpus_tests.rs": ("DISTINCT", "KNOWN_WORSE"),
    "crates/cpl-lint/src/record_tests.rs": ("DISTINCT",),
}
ARRAY_DECL_RE = re.compile(
    r"^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:const|static)\s+([A-Z][A-Z0-9_]*)\s*:[^=]*=\s*&?\[", re.M)


def lint_post_filters(root: Path) -> list[str]:
    """Entries of cpl-lint's after-the-fact tables (typed.rs's TYPED, SUPERSEDED_BY, ...)."""
    found = []
    for rel, names in LINT_POST_FILTERS.items():
        if not (root / rel).is_file():
            continue
        text = (root / rel).read_text(encoding="utf-8")
        for m in ARRAY_DECL_RE.finditer(text):
            if m.group(1) in names:
                found += [f"{rel} {m.group(1)}"] * _array_items(text[m.end():])
    return found


def _list_entries(root: Path, rel: str) -> int:
    p = root / rel
    return sum(1 for l in _lines(p) if l.strip() and not l.lstrip().startswith("#")) if p.is_file() else 0


def _registered_entries(root: Path, counter: str) -> list[str]:
    return [rel for rel, (c, _, _) in LISTS.items() if c == counter for _ in range(_list_entries(root, rel))]


def silent_unknowns(root: Path) -> list[str]:
    """Expressions in std, the corpus and spec/conformance the checker typed `Unknown`
    with no diagnostic there, per file in crates/cpl-conform/silent_unknowns.txt
    (`<count> <file>`), which tests/silent_unknowns.rs keeps equal to the checker's facts.
    Its `cause:` rows count the same spans again by cause, so they are left out."""
    p = root / "crates/cpl-conform/silent_unknowns.txt"
    rows = [line.split(" ", 1) for line in _lines(p)] if p.is_file() else []
    return [file for n, file in rows if not file.startswith("cause:") for _ in range(int(n))]


def corpus_known(root: Path) -> list[str]:
    """Entries of any corpus/*-known.txt: ports a corpus check skips."""
    lists = sorted(p.relative_to(root).as_posix() for p in (root / "corpus").rglob("*-known.txt"))
    return [rel for rel in lists for _ in range(_list_entries(root, rel))]


def known_lists(root: Path) -> list[str]:
    """Entries of *-known.txt and other accepted-finding baselines."""
    return _registered_entries(root, "known_lists")


def prelude_gaps(root: Path) -> list[str]:
    """Entries of spec/conformance/prelude-gaps.txt, deleted: every lib-table fn is a prelude name."""
    return _registered_entries(root, "prelude_gaps")


def rule_gaps(root: Path) -> list[str]:
    """Entries of spec/conformance/rule-gaps.txt, deleted: every rule has the cases it needs."""
    return _registered_entries(root, "rule_gaps")


def generate_gaps(root: Path) -> list[str]:
    """Entries of crates/cpl-stress/tests/generate-gaps.txt, deleted: stress generates every readable input."""
    return _registered_entries(root, "generate_gaps")


def sig_gaps(root: Path) -> list[str]:
    """Entries of spec/conformance/sig-gaps.txt, deleted: every named arg the chapters use is in std."""
    return _registered_entries(root, "sig_gaps")


def trait_gaps(root: Path) -> list[str]:
    """Entries of spec/conformance/trait-gaps.txt, deleted: every trait method has a case."""
    return _registered_entries(root, "trait_gaps")


def idiom_lint_gaps(root: Path) -> list[str]:
    """Entries of spec/idioms/lints-known.txt, deleted: the lint catalog names only real APIs."""
    return _registered_entries(root, "idiom_lint_gaps")


def std_lint_findings(root: Path) -> list[str]:
    """Entries of scripts/std-lint-baseline.txt, deleted: std-lint-ratchet.py fails on any hit."""
    return _registered_entries(root, "std_lint_findings")


def corpus_lint_warnings(root: Path) -> list[str]:
    """Entries of crates/cpl-lint/corpus_warnings.txt, deleted: the corpus test fails on any warning."""
    return _registered_entries(root, "corpus_lint_warnings")


def permits(root: Path) -> list[str]:
    """Entries of spec/conformance/permits.txt."""
    return _registered_entries(root, "permits")


def _workflow_lines(root: Path) -> Iterator[tuple[str, int, str]]:
    for rel in _git(root, "ls-files", ".github/workflows/*.yml", ".github/actions/*", "scripts/*.sh",
                    "scripts/**/*.sh", "justfile"):
        for i, line in enumerate(_lines(root / rel) if (root / rel).is_file() else [], 1):
            if not line.lstrip().startswith("#"):
                yield rel, i, line


def _grep_all(root: Path, pattern: str, *paths: str, flags: tuple[str, ...] = ()) -> list[str]:
    """`file:line` for every tracked line under `paths` (default all) matching `pattern`,
    one per match, the ratchet and its test aside."""
    return [":".join(hit.split(":", 2)[:2]) for hit in
            _git(root, "grep", "-n", "-o", *flags, "-E", "-e", pattern, "--", *(paths or (".",)),
                 ":!scripts/debt-ratchet.py", ":!scripts/test_debt_ratchet.py")]


TEST_PATHS = ("crates/*/tests/**", "crates/**/tests.rs", "crates/**/*_tests.rs", "crates/**/tests/**",
              "scripts/test_*", "bench/test_*", "web/tests/**", "web/**/*.test.*")


def allow_attrs(root: Path) -> list[str]:
    """`#[allow(...)]` and `#![allow(...)]` in crates: a lint silenced instead of answered."""
    return _grep_all(root, r"#!?\[allow\(", "crates/**/*.rs")


def std_lint_allows(root: Path) -> list[str]:
    """`with lint(...: allow)` in std: a lint silenced instead of answered."""
    return _grep_all(root, r"with lint\([^)]*: *allow", "std")


def unsafe_blocks(root: Path) -> list[str]:
    """`unsafe {`, `unsafe fn`, `unsafe impl` in crates."""
    return _grep_all(root, r"\bunsafe (\{|fn\b|impl\b)", "crates/**/*.rs")


ENV_MODULE = "crates/cpl-env/"


def cpl_env_reads(root: Path) -> list[str]:
    """`var`/`var_os`/`set_var`/`remove_var` of a `"CPL_…"` literal in crates outside `cpl-env`, the one
    reader that declares and documents every `CPL_*` variable: a second config channel. A test sets a
    child's variable by `cpl_env::NAME.name()`."""
    return [h for h in _grep_all(root, r"\b(var|var_os|set_var|remove_var)\(\s*\"CPL_", "crates/**/*.rs")
            if _outside(h, (ENV_MODULE,))]


def test_sleeps(root: Path) -> list[str]:
    """Sleeps in tests: a wait for a race instead of an event."""
    return _grep_all(root, r"sleep\(|time\.sleep|setTimeout\(|waitForTimeout", *TEST_PATHS)


def ci_retries(root: Path) -> list[str]:
    """Retries in workflows, scripts and the justfile outside the one helper,
    scripts/retry.sh and retry.py: a `--retry`/`--retries` flag or a hand-rolled retry loop.
    Tests get none, even with `--flaky-result fail`: a retry costs the run and a flake fails once."""
    hits = _git(root, "grep", "-n", "-E", "-e", RETRY_RE, "--", ".github", "scripts", "justfile",
                ":!scripts/retry.sh", ":!scripts/retry.py", ":!scripts/test_*.py",
                ":!scripts/debt-ratchet.py", ":!scripts/test_debt_ratchet.py")
    return [":".join(h.split(":", 2)[:2]) for h in hits]


# What the retry helper may wrap: a fetch over the network or a push that races
# another lane, by name. The retried command must start with one of these.
RETRY_FETCHES = {
    "apt": r"(timeout \d+ )?sudo apt-get install ",
    "curl": r"(timeout \d+ )?(sh -c ['\"])?curl(_page)? ",
    "npm": r"(timeout \d+ )?npm install ",
    "brew": r"brew install ",
    "choco": r"choco install ",
    "gh api": r"gh api ",
    "gh release": r"gh release view ",
    "gh run": r"gh run view ",
    "micromamba": r"\"?\S*/micromamba\"? create ",
    "git push race": r"(sh -c 'git pull -q --rebase origin main && git push |push( |$)|push_reservation,)",
    "git push holding the land lock": r"push_held( |$)",
    "git push deleting the land lock": r"delete_lock( |$)",
    "chat completion": r"post_chat,",
    "gradle fetch": r"\./gradlew --no-daemon --init-script \S+ prefetch\b",
    "cargo install": r"cargo install --locked --root ",
}
_RETRIED = re.compile(r"(?:^|[\s$(=])retry[ (](.*)")


def retry_calls(root: Path) -> list[str]:
    """Calls of the one retry helper (`retry ...` in shell, `retry(` in Python) in
    workflows and scripts whose command is not a named fetch in `RETRY_FETCHES`: a
    retry may only cover the network or a push race, never a flaky step of our own."""
    hits = _git(root, "grep", "-n", "-E", "-e", r"(^|[[:space:]$(=])retry( |\()", "--", ".github", "scripts", "justfile",
                ":!scripts/retry.sh", ":!scripts/retry.py", ":!scripts/test_*.py",
                ":!scripts/debt-ratchet.py", ":!scripts/test_debt_ratchet.py")
    out = []
    for h in hits:
        rel, i, line = h.split(":", 2)
        if line.lstrip().startswith("#") or " import retry" in line:
            continue
        retried = _RETRIED.search(line)
        if not (retried and any(re.match(p, retried.group(1)) for p in RETRY_FETCHES.values())):
            out.append(f"{rel}:{i}")
    return out


# Workflows that operate CI or ship, and check no code, so a contestant has nothing to rerun.
CI_ONLY_WORKFLOWS = {
    "commit-status.yml": "marks main's status from other runs",
    "culprit.yml": "bisects failed runs",
    "unstick.yml": "re-queues stuck runs",
    "watchdog.yml": "cancels hung runs",
    "release.yml": "builds and publishes a release",
    "site.yml": "deploys the built site",
    "writable-from-docs.yml": "a model writes programs from the docs",
}
# Jobs that write to main or stage binaries for other jobs, inside checking workflows.
CI_ONLY_JOBS = {
    "ci.yml:regen": "commits regenerated files to main",
    "ci.yml:tests-build": "archives the test binaries the `tests` shards run",
}
CI_CHECK = re.compile(r"(^|[\s;&|(])\./gradlew\b|(^|[\s;&|(])(\./)?(scripts|editors|research|bench|crates)/\S+\.(sh|py|lua|el|mjs|js)\b|cargo (test|nextest|run)\b"
                      r"|pnpm .*\b(test|build|smoke|typecheck)\b|npm (test|run|ci)\b|nvim --headless")


def ci_jobs_without_recipe(root: Path) -> list[str]:
    """Checking CI jobs that run a check other than through `just <recipe>`: a red job
    should be one command to rerun locally, the same one CI ran."""
    out = []
    for rel in _git(root, "ls-files", ".github/workflows/*.yml"):
        if Path(rel).name in CI_ONLY_WORKFLOWS:
            continue
        job, in_jobs, block, flagged = None, False, None, set()
        for i, line in enumerate((root / rel).read_text().splitlines(), 1):
            if re.match(r"^jobs:\s*$", line):
                in_jobs = True
                continue
            if in_jobs and (m := re.match(r"^  ([A-Za-z0-9_-]+):\s*$", line)):
                job, block = m.group(1), None
                continue
            if job is None or job in flagged or f"{Path(rel).name}:{job}" in CI_ONLY_JOBS:
                continue
            indent = len(line) - len(line.lstrip())
            if block is not None and (not line.strip() or indent > block):
                cmd = line.strip()
            elif m := re.match(r"^(\s*)(?:- )?run:\s*(\|?)(.*)$", line):
                block = indent if m.group(2) else None
                cmd = m.group(3).strip()
            else:
                block = None
                continue
            if cmd and not cmd.startswith(("#", "just ", ". scripts/retry.sh")) and CI_CHECK.search(cmd):
                flagged.add(job)
                out.append(f"{rel}:{i} ({job})")
    return out


# The two sleeps a blocking wait can't replace without a crate: the file-change
# poll and the judge's rate limit. Every other caller reuses one of them.
SLEEP_OWNERS = ("crates/cpl-wait/src/files.rs", "crates/cpl-wait/src/pace.rs")


def poll_sleeps(root: Path) -> list[str]:
    """`thread::sleep` in shipped Rust outside cpl-wait's file watch and rate limiter:
    a poll or backoff where `cpl_wait::on_exit`, a channel or a lock belongs."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root)
            if "thread::sleep" in line and rel not in SLEEP_OWNERS]


RETRY_RE = r"--retr(y|ies)\b|\bfor \w+ in [0-9]+( [0-9]+)+; do|\b(re)?try\(\) *\{|\bfor (try|tries|attempt) in |\bfor _ in range\((tries|retries|attempts)\)"


def hardcoded_codes(root: Path) -> list[str]:
    """String literals naming a checker code (`E0123`) in shipped code outside cpl-check and cpl-diag."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root)
            if _outside(rel, ("crates/cpl-check/", "crates/cpl-diag/"))
            for _ in re.finditer(r'"[^"\n]*\bE\d{4}\b', line)]


def xfail_markers(root: Path) -> list[str]:
    """`xfail`/known-fail/expected-fail markers, `// gap:` lines on failing conformance
    cases (decision 515), and lines of known-fail lists (web a11y)."""
    words = _grep_all(root, r"xfail|known[-_]fail|expected[-_]fail", ".", ":!decisions.md",
                      ":!scripts/debt-ratchet.json", flags=("-i",))
    return words + \
        _grep_all(root, r"^// gap:", "spec/conformance") + _registered_entries(root, "xfail_markers")


def todo_fixme(root: Path) -> list[str]:
    """`TODO`/`FIXME` in std, web and scripts."""
    return _grep_all(root, r"\b(TODO|FIXME)\b", "std", "web", "scripts")


def web_skip_only(root: Path) -> list[str]:
    """`.skip(`/`.only(` in web tests: a test that doesn't run, or that stops the rest running.
    Only JS/TS sources: in a `.cp` program `.only(` is the std graph filter."""
    return _grep_all(root, r"\.(skip|only)\(", *WEB_SCRIPT_PATHS)


WEB_SCRIPT_PATHS = tuple(f"web/*.{ext}" for ext in ("ts", "tsx", "js", "jsx", "mjs", "cjs"))


def ts_ignore(root: Path) -> list[str]:
    """`@ts-ignore`/`@ts-nocheck` in web: a type error silenced."""
    return _grep_all(root, r"@ts-(ignore|nocheck)", "web")


HABIT_FILES = ("crates/cpl-check/src/check/advice.rs",)


HABIT_REGISTRY = "crates/cpl-diag/src/habits.rs"
HABIT_MISTAKES = "crates/cpl-cli/tests/habits"
_HABIT_TEXT = re.compile(r'(with_help|\.help\s*=\s*Some|\.fix|with_fix)\(|\b(?:title|help):\s*|\blet (?:help|title)\s*=\s*')


def _first_arg(text: str, i: int) -> str:
    """The expression starting at `text[i]`, up to its `,`, `;` or closing bracket."""
    depth, j = 0, i
    while j < len(text):
        c = text[j]
        if c == '"':
            j += 1
            while j < len(text) and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                break
            depth -= 1
        elif c in ",;" and depth == 0:
            break
        j += 1
    return text[i:j]


def habit_matchers(root: Path) -> list[str]:
    """Help or fix-title text in foreign/advice/habit code that doesn't come from a
    `cpl_diag::habits` row (a help or fix call, `title:`/`help:`, or `let help`/`let title`
    whose value holds a string literal), plus rows with no mistake program in
    crates/cpl-cli/tests/habits. The generated test in crates/cpl-cli/tests/diagnostics.rs
    fails any row whose mistake doesn't show it or whose fix doesn't check (Consumers; this
    replaces the count of `fn` items in those files)."""
    files = [rel for rel in _git(root, "ls-files", "crates/**/*.rs")
             if (rel in HABIT_FILES or re.search(r"foreign|habit|advice", Path(rel).name)
                 or "/foreign/" in rel)
             and rel != HABIT_REGISTRY and not rel.endswith("tests.rs")]
    found = []
    for rel in files:
        text = (root / rel).read_text()
        for m in _HABIT_TEXT.finditer(text):
            line = text[text.rfind("\n", 0, m.start()) + 1:m.start()]
            arg = _first_arg(text, m.end())
            if not line.lstrip().startswith("//") and '"' in arg and "habits::" not in arg:
                found.append(f"{rel}:{text.count(chr(10), 0, m.start()) + 1}")
    registry = (root / HABIT_REGISTRY).read_text() if (root / HABIT_REGISTRY).is_file() else ""
    for key in re.findall(r'\b(?:call|value|shape)\(\s*"(\w+)"', registry):
        if not (root / HABIT_MISTAKES / f"{key}.cp").is_file():
            found.append(f"{HABIT_REGISTRY}:{key}")
    return found


FIX_GATE = ("crates/cpl-ir/src/fix.rs", "crates/cpl-lint/src/fix.rs")


def unchecked_fix_paths(root: Path) -> list[str]:
    """Fix appliers called outside the one re-checking gate (FIX_GATE: `cpl fix`'s
    checked rounds), the raw `splice_fixes` included, and any `fn apply_`
    definition in cpl-diag/src/fix.rs, where only the unchecked text step lives."""
    calls = [f"{rel}:{i}" for rel, i, line in _src_lines(root)
             if rel not in FIX_GATE and rel != "crates/cpl-diag/src/fix.rs"
             and re.search(r"\b(apply_fixes|splice_fixes|apply_error_fixes|apply_lint_fixes)\(", line)
             and not re.search(r"\bfn\s", line)]
    defs = [f"{rel}:{i}" for rel, i, line in _src_lines(root, "crates/cpl-diag/src")
            if rel == "crates/cpl-diag/src/fix.rs" and re.search(r"\bfn apply_", line)]
    return calls + defs


def stress_engines(root: Path) -> list[str]:
    """Stress front ends and verdict pickers in cpl-stress: definitions of the
    old twins (`fn run_case|stress_source|judge_case`), files besides judge.rs
    constructing `Verdict::`, and files besides stress.rs calling `stress_with`.

    It is 0 (decision 561, superseding 545): `case.rs` `Roles::judge` runs a
    case's roles and `judge.rs` picks its verdict. `Stress::build`/`Stress::run` is
    the one front end the CLI and the browser call; how a role's program runs
    (in the VM, or as a native process) is a `Runner` below it."""
    lines = list(_src_lines(root, "crates/cpl-stress/src"))
    defs = [f"{rel}:{i}" for rel, i, line in lines if re.search(r"\bfn (run_case|stress_source|judge_case)\b", line)]
    files = sorted({rel for rel, _, line in lines
                    if "Verdict::" in line and Path(rel).name != "judge.rs"})
    loops = sorted({f"{rel}:{i}" for rel, i, line in lines
                    if re.search(r"(?<!fn )\bstress_with\(", line) and Path(rel).name != "stress.rs"})
    return defs + loops + [f for f in files if not any(d.startswith(f + ":") for d in defs)]


def or_true(root: Path) -> list[str]:
    """`|| true` anywhere in the repo: a failure swallowed."""
    return _grep_all(root, r"\|\|[[:space:]]*true\b")


TRACE_CEILING_RE = re.compile(r'^\s*\("(\w+)", (\d+(?:\.\d+)?)\),$')


def trace_budget_gap(root: Path) -> list[str]:
    """Tenths of a slowdown over the 1.2x trace budget that trace_log's
    `OVER_BUDGET` ceilings allow, one entry per tenth."""
    path = root / "crates/cpl-interp/tests/trace_log.rs"
    lines = _lines(path) if path.is_file() else []
    start = next((i for i, l in enumerate(lines) if l.startswith("const OVER_BUDGET")), len(lines))
    out = []
    for line in lines[start + 1:]:
        m = TRACE_CEILING_RE.match(line)
        if not m:
            break
        out += [f"{m[1]} {k + 1}" for k in range(max(0, round((float(m[2]) - 1.2) * 10)))]
    return out


TIMEOUT_ARM = re.compile(r"^\s*124\)(?!.*\bexit\s+(?:[1-9]|\"?\$))")
TIMEOUT_EXCEPT = re.compile(r"^(.+?\.py)-(\d+)-\s*(?:return(?:\s+(?:None|False|True))?|pass|continue)\s*$")


def timeout_passes(root: Path) -> list[str]:
    """A timeout that ends in success: a shell `124)` arm that doesn't exit non-zero, or a
    Python `except TimeoutExpired:` whose next line gives no verdict (None, False, True, pass,
    continue) unless a `# timeout fails: <where>` note names who reports it."""
    skip = (":!scripts/debt-ratchet.py", ":!scripts/test_*.py", ":!**/test_*.py", ":!**/tests/**")
    arms = [":".join(h.split(":", 2)[:2]) for h in
            _git(root, "grep", "-n", "-E", "-e", r"^[[:space:]]*124\)", "--", ".", *skip)
            if TIMEOUT_ARM.match(h.split(":", 2)[2])]
    py = [f"{m[1]}:{m[2]}" for h in
          _git(root, "grep", "-n", "-A1", "-E", "-e", r"except[[:space:]]*\(?[[:alnum:]_.]*TimeoutExpired", "--", "*.py", *skip)
          if (m := TIMEOUT_EXCEPT.match(h))]
    return sorted(arms + py)


# Lowering's own answers in the handoff: the IR, and the messages lowering itself makes.
HANDOFF_OWN_FIELDS = {"funcs", "errors"}


def _struct_fields(root: Path, srcs: list[str]) -> dict[str, list[tuple[str, str, str]]]:
    """Each struct the crates under `srcs` define: its fields, `(name, type, where)`, a tuple
    struct's as `0`, `1`.."""
    out: dict[str, list[tuple[str, str, str]]] = {}
    for p in _rs_files(root, *srcs, tests=False):
        lines, rel = _lines(p), _rel(root, p)
        for i, l in enumerate(lines):
            m = re.match(r"\s*pub(?:\([^)]*\))?\s+struct\s+(\w+)(?:<[^>]*>)?\s*(\(|\{)", l)
            if not m:
                continue
            name, fields = m.group(1), []
            if m.group(2) == "(":
                inner, parts, depth, cur = l[m.end():l.rfind(")")], [], 0, ""
                for ch in inner:
                    depth += ch in "<([" and 1 or ch in ">)]" and -1 or 0
                    if ch == "," and depth == 0:
                        parts.append(cur)
                        cur = ""
                    else:
                        cur += ch
                parts.append(cur)
                fields = [(str(k), t.strip(), f"{rel}:{i + 1}") for k, t in enumerate(parts) if t.strip()]
            else:
                for k in range(i + 1, len(lines)):
                    if lines[k].startswith("}"):
                        break
                    f = re.match(r"\s*(?:pub(?:\([^)]*\))?\s+)?(\w+):\s*(.+?),?$", lines[k])
                    if f and not lines[k].lstrip().startswith("//"):
                        fields.append((f.group(1), f.group(2), f"{rel}:{k + 1}"))
            out[name] = fields
    return out


def next_handoff_side_fields(root: Path) -> list[str]:
    """Every field that reaches link beside the IR (shortcut audit S18): each field of lowering's
    `Generic` other than its fns and its own messages, where a field's type is a struct of
    `cpl-next-ir` or `cpl-next-lower` each field of that, as far as it goes. A new side struct
    is counted field by field, and a collection or tuple as one table. Each becomes a per-item
    answer link reads by id (`domain(ItemId)`, `global(ItemId)`, `diagnostics(ItemId)`,
    handbook/design/next/incremental.md), so this reaches 0."""
    if not (root / "crates/cpl-next-lower/src/lib.rs").is_file():
        return []
    structs = _struct_fields(root, ["crates/cpl-next-ir/src", "crates/cpl-next-lower/src"])
    if "Generic" not in structs:
        return ["crates/cpl-next-lower/src/lib.rs: no `pub struct Generic` (the handoff moved; update this counter)"]
    out, seen = [], set()

    def walk(owner: str, field: str, ty: str, at: str) -> None:
        # A field whose type is itself a struct of the handoff holds its fields; one holding
        # values in a collection or tuple is one table.
        whole = re.sub(r"^&(?:'\w+\s+)?(?:mut\s+)?", "", ty).split("::")[-1]
        inner = [whole] if whole in structs and whole not in seen else []
        if not inner:
            out.append(f"{at}: {owner}.{field}")
            return
        for n in inner:
            seen.add(n)
            for f, t, w in structs[n]:
                walk(n, f, t, w)

    for f, t, w in structs["Generic"]:
        if f not in HANDOFF_OWN_FIELDS:
            walk("Generic", f, t, w)
    return out


def next_check_set_facts(root: Path) -> list[str]:
    """Sites in the checker that build a function-value set fact, `SetTy::Global` or
    `SetTy::AddedAt` (shortcut audit S25): which closures a set holds is a link-time fact
    (decision 35, "sets leave the type"), so this reaches 0 when sets leave the checker's types."""
    out = []
    for p in _rs_files(root, "crates/cpl-next-check/src", tests=False):
        rel = _rel(root, p)
        out += [f"{rel}:{i}" for i, line in enumerate(_lines(p), 1)
                if re.search(r"SetTy::(?:Global|AddedAt)\(", line) and not line.lstrip().startswith("//")]
    return out


def next_snapshot_keyed_queries(root: Path) -> list[str]:
    """Facade answers memoized per whole `Snapshot` (shortcut audit S3): a `OnceLock` field of the
    facade's per-snapshot state is recomputed in full by any edit, and so is a database query
    whose compute calls the whole pipeline (`compile`, `front`, `back`). One query database
    keyed by item path and input hash, whose whole-program steps read per-item answers,
    replaces them, so this reaches 0 (END-STATE #20)."""
    out: list[str] = []
    p = root / "crates/cpl-next/src/facade.rs"
    if p.exists():
        rel = _rel(root, p)
        out += [f"{rel}:{i}" for i, line in enumerate(_lines(p), 1) if re.match(r"\s*\w+:\s*OnceLock<", line)]
    # A query of the database that answers by running the whole pipeline is the same cache
    # under another name: it reruns everything when any input it read changes.
    for q in _rs_files(root, "crates/cpl-next/src", tests=False):
        rel, inside = _rel(root, q), False
        for i, line in enumerate(_lines(q), 1):
            if re.match(r"impl\b.*\bQuery for\b", line):
                inside = True
            elif line.startswith("}"):
                inside = False
            elif inside and NEXT_WHOLE_PIPELINE_RE.search(line):
                out.append(f"{rel}:{i}")
    return out


NEXT_WHOLE_PIPELINE_RE = re.compile(r"(?<![\w.])(?:crate::)?(?:compile|front|back)\(")


TYPE_ITEM_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait|type)\s+(\w+)")
# Diagnostics: the checker's, and its relay of them, are checker data wherever they are defined.
CHECKER_RELAYS = ("Deferred", "Msg")


def _link_input_checker_types(root: Path) -> list[str]:
    """Each checker type link's inputs reach, by the type graph: from cpl-next-link's `Inputs`,
    every type a struct's fields, an enum's variants, a trait's method signatures or an alias
    name, each name resolved in the nearest crate defining it: the naming crate, then its
    dependencies, then theirs. A type defined
    in cpl-next-check, or a diagnostic (`Msg`, or the relay of them `Deferred`), counts, with its path."""
    deps = _crate_deps(root)
    def layers(c: str) -> list[set[str]]:
        """The crates `c` reaches, nearest first: itself, its dependencies, theirs, ..."""
        seen, out, at = {c}, [{c}], {c}
        while at := {d for x in at for d in deps.get(x, []) if d not in seen}:
            seen |= at
            out.append(at)
        return out
    defs: dict[str, list[tuple[str, str, set[str]]]] = {}
    for p in _rs_files(root, *_next_srcs(root), tests=False):
        rel = _rel(root, p)
        lines = list(_lines(p))
        for i, line in enumerate(lines):
            if m := TYPE_ITEM_RE.match(line):
                body = " ".join(x.split("//")[0] for x in lines[i:_block(lines, i) + 1])
                names = set(re.findall(r"\b[A-Z]\w*", body)) - {m.group(1)}
                defs.setdefault(m.group(1), []).append((rel.split("/")[1], rel, names))
    start = [("cpl-next-link", "Inputs")]
    if not any(c == "cpl-next-link" for c, _, _ in defs.get("Inputs", [])):
        return []
    prev: dict[tuple[str, str], tuple[str, str] | None] = {start[0]: None}
    where: dict[tuple[str, str], str] = {}
    todo, out = list(start), []
    while todo:
        node = todo.pop(0)
        crate, name = node
        mine = [(r, ns) for c, r, ns in defs.get(name, []) if c == crate]
        for rel, names in mine:
            where.setdefault(node, rel)
            for n in sorted(names):
                found = next(([c for c, _, _ in defs.get(n, []) if c in at] for at in layers(crate)
                              if any(c in at for c, _, _ in defs.get(n, []))), [])
                for c in found:
                    if (c, n) not in prev:
                        prev[(c, n)] = node
                        todo.append((c, n))
    for node in prev:
        crate, name = node
        if crate == "cpl-next-check" or name in CHECKER_RELAYS:
            path, at = [], node
            while at:
                path.append(at[1])
                at = prev[at]
            out.append(f"{where.get(node, crate)}: link's Inputs reaches {name} via {' -> '.join(reversed(path))}")
    return sorted(out)


def link_reads_check(root: Path) -> list[str]:
    """Where checker data reaches the linker (cpl-next-link) (shortcut audit S18), by the type
    graph, not by names: each checker type link's `Inputs` reach through the types they name
    (`_link_input_checker_types`), so a checker diagnostic or table relayed through lower or a
    view counts however it is spelled. By crate: no crate past lowering may reach the checker,
    through however many crates."""
    out = _link_input_checker_types(root)
    # The structural half: no crate past lowering may reach the checker at all, whatever names it
    # uses and through however many crates (a direct-only check misses `link -> x -> check`).
    deps = _crate_deps(root)
    for c in AFTER_LOWER_CRATES:
        path = _dep_path(deps, c, "cpl-next-check")
        if path:
            out.append(f"crates/{c}/Cargo.toml: reaches the checker via {' -> '.join(path)}")
    return out


def _crate_deps(root: Path) -> dict[str, list[str]]:
    """Each workspace crate's normal `cpl-*` dependencies (not dev or build), by package name.
    `cargo metadata` resolves renames (`x = { package = "cpl-next-check" }`) and
    `[dependencies.x]` tables; the Cargo.toml reading below is only for a tree Cargo can't load."""
    if (root / "Cargo.toml").is_file():
        try:
            out = subprocess.run(["cargo", "metadata", "--format-version", "1", "--no-deps", "--offline"],
                                 cwd=root, capture_output=True, text=True, timeout=60, check=True).stdout
            return {Path(p["manifest_path"]).parent.name: [d["name"] for d in p["dependencies"]
                                                           if d["kind"] is None and d["name"].startswith("cpl")]
                    for p in json.loads(out)["packages"]}
        except (OSError, subprocess.SubprocessError, ValueError, KeyError):
            pass
    deps: dict[str, list[str]] = {}
    for toml in sorted((root / "crates").glob("*/Cargo.toml")):
        section, names = "", []
        for line in _lines(toml):
            if line.startswith("["):
                section = line.strip()
                if (m := re.fullmatch(r"\[(?:target\..*\.)?dependencies\.(cpl[\w-]*)\]", section)):
                    names.append(m.group(1))
            elif re.search(r"(?<![\w-])dependencies]$", section) \
                    and (m := re.match(r"\s*([\w-]+)\s*[=.]", line)):
                pkg = re.search(r'package\s*=\s*"(cpl[\w-]*)"', line)
                if pkg or m.group(1).startswith("cpl"):
                    names.append(pkg.group(1) if pkg else m.group(1))
        deps[toml.parent.name] = names
    return deps


def _dep_path(deps: dict[str, list[str]], start: str, goal: str) -> list[str]:
    """The shortest dependency path from `start` to `goal`, or [] when there is none."""
    prev, todo = {start: ""}, [start]
    while todo:
        c = todo.pop(0)
        if c == goal:
            path = [c]
            while prev[path[-1]]:
                path.append(prev[path[-1]])
            return path[::-1]
        for d in deps.get(c, []):
            if d not in prev:
                prev[d] = c
                todo.append(d)
    return []


AFTER_LOWER_CRATES = ("cpl-next-own", "cpl-next-link", "cpl-next-cpp", "cpl-next-interp", "cpl-next-engine")
# A scope key built from another name (`{d}$lo`, `{b}.{field}`): meaning carried in a string.
# `{lo}..={hi}` shows a range type and is no key. Positional holes (`{}`, `{0}`) count too.
NEXT_NAME_KEY_RE = re.compile(r'format!\("\{(?:[a-z_]\w*|\d*)\}(?:\$|\.(?!\.))')


def next_check_name_keys(root: Path) -> list[str]:
    """Checker lines that build a binding or lookup key by formatting another name (`{d}$lo`,
    `{b}.{field}`): a domain's slots or a scoped member found by spelling, not by an id bound in
    scope, so a rename, shadow or nesting bug becomes a wrong domain. 0 is every such slot an id."""
    return [f"{_rel(root, p)}:{i}" for p in _rs_files(root, "crates/cpl-next-check/src", tests=False)
            if not p.name.endswith("_tests.rs")
            for i, l in enumerate(_lines(p), 1) if not l.lstrip().startswith("//") and NEXT_NAME_KEY_RE.search(l)]


NEXT_TEXT_KEYED_RE = re.compile(
    r"\b(?:HashMap|HashSet|BTreeMap|BTreeSet)<\s*(?:String|&(?:'\w+\s+)?str|Box<str>|Arc<str>|Rc<str>)\s*[,>]"
    r"|\bVec<\(\s*(?:String|&(?:'\w+\s+)?str)\s*,")


# The one table that turns a name's text into an id: every later table keys by its `Sym`.
NEXT_RESOLVER = "crates/cpl-next-decls/src/sym.rs"


def next_string_keyed_maps(root: Path) -> list[str]:
    """Maps, sets and pair lists keyed by a name's text in the new pipeline, outside the resolver
    (S4): text becomes an id once, in declarations (contracts 0 and 2), and every later table
    (methods, members, impls, domains, memo tables, the prelude, facade items) is keyed by
    `FnId`/`StructId`/`ModId`/`ItemId`, so a shadow, rename or nesting bug cannot pick a wrong item."""
    out = []
    for p in _rs_files(root, "crates", tests=False):
        rel = _rel(root, p)
        if not rel.startswith("crates/cpl-next") or "/src/" not in rel or rel == NEXT_RESOLVER:
            continue
        if p.name.endswith("_tests.rs"):
            continue
        out += [f"{rel}:{i}" for i, l in enumerate(_lines(p), 1)
                if not l.lstrip().startswith("//") and NEXT_TEXT_KEYED_RE.search(l)]
    return out


def trace_engine_refs(root: Path) -> list[str]:
    """`CPL_TRACE_ENGINE` mentions: the switch back to the tree-walker for traces."""
    return _grep_all(root, "CPL_TRACE_ENGINE")


STRUCT_RE = re.compile(r"pub struct (\w+)")
FIELD_RE = re.compile(r"(?:@\w+ )*(?:var )?(pub )?(?:var )?(\w+): ")


EDGE_LABEL_RE = re.compile(r"^\s*/// (empty|ties): (.*)$")
EDGE_CLAIM_RE = re.compile(r"^`([^`]+)`((?:\s+\((?:stdin `[^`]*`|empty stdin|traps R\d+|error E\d+)\))*)\s*$")


def std_model_gaps(root: Path) -> list[str]:
    """Std types and pub std fns no spec/conformance/std-models/ case checks against a brute force
    (scripts/std-models.py --list)."""
    import importlib.util
    if not (root / "scripts/std-models.py").exists():
        return []
    spec = importlib.util.spec_from_file_location("std_models", root / "scripts/std-models.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod.gaps(mod.std_texts(root), mod.case_texts(root))


def std_default_model_gaps(root: Path) -> list[str]:
    """Pub std traits' default methods (`Seq.sum`) no spec/conformance/std-models/ case checks
    against a brute force: std_model_gaps's pub fns, moved into a trait (a12ba755e)."""
    import importlib.util
    if not (root / "scripts/std-models.py").exists():
        return []
    spec = importlib.util.spec_from_file_location("std_models", root / "scripts/std-models.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod.default_gaps(mod.std_texts(root), mod.case_texts(root))


def edge_labels_unrun(root: Path) -> list[str]:
    """std `empty:`/`ties:` labels `cpl-conform docs` can't run: prose, or a bare value
    rather than a claim (`docs.rs` `runnable_label`), so no case checks that edge (Content)."""
    found = []
    for rel in _git(root, "ls-files", "std/*.cp"):
        for i, line in enumerate((root / rel).read_text(encoding="utf-8").split("\n"), 1):
            if not (m := EDGE_LABEL_RE.match(line)):
                continue
            c = EDGE_CLAIM_RE.match(m.group(2).strip())
            last = c.group(1).split(";")[-1].strip() if c else ""
            if not (c and (re.search(r" (==|!=) ", last) or last.startswith("not ") or c.group(2))):
                found.append(f"{rel}:{i}")
    return found


def std_trait_matrix(root: Path) -> list[str]:
    """std containers missing `Show`, a walk or `is_empty` (std-api/R12, scripts/std-trait-matrix.py; Content)."""
    import importlib.util
    spec = importlib.util.spec_from_file_location("std_trait_matrix", Path(__file__).with_name("std-trait-matrix.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return [f"{rel}:{line} {name} {trait}" for rel, line, name, missing in mod.matrix(str(root)) for trait in missing]


def std_types_shown_empty(root: Path) -> list[str]:
    """`pub struct`s in std with no `pub` field and no `impl Show`: a trace shows their
    values as `{}`, since a std type's private fields never reach a user (Content).
    The struct filling the `option` role (`@role(option)`) is not one: `?T` is deliberately not `Show` (R10.3.7).
    A struct a `@role(trace)` fn takes is not one either: that fn is what a trace shows (Rtrace.7)."""
    shows, domains, blanket, empty = set(), set(), False, []
    for rel in _git(root, "ls-files", "std/*.cp"):
        lines = (root / rel).read_text(encoding="utf-8").split("\n")
        shows |= {m.group(1) for l in lines if (m := re.match(r"impl(?:<.*>)? Show for (\w+)", l))}
        shows |= {m.group(1) for a, l in zip(lines, lines[1:]) if a == "@role(trace)"
                  and (m := re.match(r"(?:pub )?fn \w+(?:<[^>]*>)?\(\w+: (\w+)", l))}
        domains |= {m.group(1) for l in lines if (m := re.match(r"impl(?:<.*>)? Domain for (\w+)", l))}
        blanket |= any(re.match(r"impl<D: Domain> Show for D:", l) for l in lines)
        for i, line in enumerate(lines):
            if not (m := STRUCT_RE.match(line)):
                continue
            marks, j = [], i - 1
            while j >= 0 and lines[j].startswith("@"):
                marks.append(lines[j].strip())
                j -= 1
            if "@role(option)" in marks:
                continue
            header = re.search(r"\((.*)\)", line)
            fields = [FIELD_RE.match(a.strip()) for a in header.group(1).split(",")] if header else []
            for body in lines[i + 1:]:
                if body.strip() and not body.startswith("    "):
                    break
                if body.startswith("    ") and not body.startswith("     "):
                    fields.append(FIELD_RE.match(body.strip()))
            fields = [f for f in fields if f]
            if fields and not any(f.group(1) for f in fields):
                empty.append((m.group(1), f"{rel}:{i + 1}"))
    shows |= domains if blanket else set()
    return [at for name, at in empty if name not in shows]


CONTINUE_RE = re.compile(r"^(\s*)(?:-\s+)?continue-on-error:\s*(.+?)\s*$")


INSTALL_RE = re.compile(r"\s*(- )?uses: taiki-e/install-action|.*\bcargo b?install\b|.*download-\w+\.bash")
INSTALL_ACTION = ".github/actions/install-tools/action.yml"


def uncached_installs(root: Path) -> list[str]:
    """Tool installs in a workflow or action outside .github/actions/install-tools, whose cache
    hit makes no network call: an install on every run fails every job when its host is down
    (run 45388976012: cargo-binstall exit 76 in setup-rust)."""
    return [f"{rel}:{i}" for rel, i, line in _workflow_lines(root)
            if rel != INSTALL_ACTION and INSTALL_RE.match(line)]


def continue_on_error(root: Path) -> list[str]:
    """Jobs and steps marked `continue-on-error` (not `false`) in any workflow or action:
    a red that turns nothing red."""
    return [f"{rel}:{i}" for rel, i, line in _workflow_lines(root)
            if (m := CONTINUE_RE.match(line)) and m.group(2) != "false"]


def dream_unverified(root: Path) -> list[str]:
    """corpus/problems/*/dream.cp whose meta.json has no `dream_stress` record with ok=true
    for the current blob (written by `scripts/corpus.py record`)."""
    found = []
    for line in _git(root, "ls-files", "-s", "corpus/problems/*/dream.cp"):
        meta_sha, rel = line.split()[1], line.split("\t", 1)[1]
        meta = root / Path(rel).parent / "meta.json"
        try:
            rec = json.loads(meta.read_text(encoding="utf-8")).get("dream_stress") or {}
        except (OSError, ValueError):
            rec = {}
        if rec.get("ok") is not True or rec.get("sha") != meta_sha:
            found.append(rel)
    return found


def measure_broken(root: Path) -> list[str]:
    """plan/domains/*/measure.py with no scripts/test_<domain>_measure.py to catch "0 of 0"."""
    return [_rel(root, p) for p in sorted((root / "plan/domains").glob("*/measure.py"))
            if not (root / f"scripts/test_{p.parent.name}_measure.py").is_file()]


def mono_unguarded(root: Path) -> list[str]:
    """1 while no shipped cpl-check or cpl-ir code raises E0319 (`GROWING_INSTANTIATION`):
    monomorphization without a termination guard."""
    used = any("GROWING_INSTANTIATION" in line for _, _, line in _src_lines(root, "crates/cpl-check/src", "crates/cpl-ir/src"))
    return [] if used else ["crates/cpl-check, crates/cpl-ir: no GROWING_INSTANTIATION use"]


# Generics nest one level, `<S: Seq<T>, T>`.
STD_ITEM_RE = re.compile(r"^(\s*)(?:@\w+\s+)*pub\s+(fn|struct|enum|trait|type)\s+(\w+)\s*(<(?:[^<>(]|<[^<>(]*>)*>)?\s*(?:\(([^)]*))?")
# Receivers a built-in arm on lists, strings, maps, sets, arrays or a type
# parameter can catch: a std fn over one of these never runs its body.
BUILTIN_SELF_RE = re.compile(r"^self:\s*(?:\[|List<|Str\b|Map<|Set<|HashMap<|HashSet<|Range<)")
COMPILER_SRC = ("crates/cpl-check/src", "crates/cpl-ir/src", "crates/cpl-interp/src", "crates/cpl-codegen/src")


def _builtin_floor(root: Path) -> set[str]:
    """The decision-266 intrinsic names the compiler keeps."""
    floor: set[str] = set()
    table = root / "plan/domains/tech-debt/architecture/builtin-floor.md"
    for line in _lines(table) if table.is_file() else []:
        cells = line.split("|")
        if len(cells) > 3 and cells[2].strip() == "intrinsic":
            floor.update(re.findall(r"`(\w+)`", cells[3]))
    return floor


def ir_builtin_derived(root: Path) -> list[str]:
    """String-matched names not on a decision-266 intrinsic row: std bodies the compiler still owns."""
    floor = _builtin_floor(root)
    return [n for n in ir_builtin_names(root) if n not in floor]


def _compiler_strings(root: Path) -> set[str]:
    """Every identifier-like string literal in the checker, IR, interpreter and emitter, tests aside."""
    found: set[str] = set()
    for base in COMPILER_SRC:
        files = sorted((root / base).rglob("*.rs"))
        test_mods: set[Path] = set()
        for f in files:
            here = f.parent if f.name in ("lib.rs", "main.rs", "mod.rs") else f.parent / f.stem
            for name in re.findall(r"#\[cfg\(test\)\]\s*(?:#\[[^\]]*\]\s*)*mod (\w+);", f.read_text(encoding="utf-8")):
                test_mods.update((here / f"{name}.rs", here / name / "mod.rs"))
        for f in files:
            if "tests" in f.parts or f.name in ("tests.rs", "test.rs") or f in test_mods:
                continue
            text = f.read_text(encoding="utf-8")
            text = text.split("#[cfg(test)]")[0]
            found.update(re.findall(r'"([A-Za-z_]\w*)"', text))
    return found


def std_shadowed(root: Path) -> list[str]:
    """Public std items whose calls the compiler answers itself instead of running the body,
    off the decision-266 floor: (1) a fn the lowering string-matches by name whose receiver a
    built-in arm can catch (a free fn, or a `self` that is a type parameter, list, string, map,
    set, range or array); (2) a type the checker, IR, interpreter or emitter names in a string
    literal, whose methods it may answer too; (3) a fn the checker's known-call arms
    (`crates/cpl-check/src/check/known.rs`) type or check by name instead of its signature."""
    floor = _builtin_floor(root)
    arms = set(ir_builtin_names(root))
    named = _compiler_strings(root)
    known = root / "crates/cpl-check/src/check/known.rs"
    checked = set(re.findall(r'^\s*\(\s*"(\w+)"', known.read_text(encoding="utf-8"), re.M)) \
        if known.is_file() else set()
    out: set[str] = set()
    for p in sorted((_old_std(root)).glob("*.cp")):
        for line in _lines(p):
            m = STD_ITEM_RE.match(line)
            if not m:
                continue
            indent, kind, name, generics, params = m.groups()
            if kind != "fn":
                if not indent and name in named and name not in floor:
                    out.add(name)
                continue
            if not indent and name in checked and name not in floor:
                out.add(name)
            if name not in arms or name in floor:
                continue
            if indent:
                continue
            first = (params or "").split(",")[0].strip()
            params_of = re.findall(r"\b([A-Z]\w*)\b", generics or "")
            self_ty = first.split(":", 1)[1].strip() if first.startswith("self:") else ""
            if not first.startswith("self") or BUILTIN_SELF_RE.match(first) \
                    or re.split(r"[<\s]", self_ty)[0] in params_of:
                out.add(name)
    return sorted(out)


WALK_RE = re.compile(r"\b(?:glob|rglob|iglob|listdir|iterdir|scandir|walk|read_dir|readdir\w*|ls)\b|\*")


SCALING_BOARD = "research/perf/scaling.json"


def _scaling_board(root: Path) -> dict | None:
    try:
        return json.loads((root / SCALING_BOARD).read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None


def scaling_mismatches(root: Path) -> list[str]:
    """bench/scaling cases whose std fn grows other than its `cost:` says, as
    `bench/scaling.py --write` recorded them; a missing board counts."""
    board = _scaling_board(root)
    if board is None:
        return [f"{SCALING_BOARD}: no board; run bench/scaling.py --write"]
    return [f"{c}: {r.get('fn')}" for c, r in sorted(board.get("cases", {}).items()) if not r.get("ok")]


def std_fns_unscaled(root: Path) -> list[str]:
    """std fns with a `cost:` line and no bench/scaling case timing it."""
    board = _scaling_board(root)
    if board is None:
        return [f"{SCALING_BOARD}: no board; run bench/scaling.py --write"]
    return list(board.get("unscaled", []))


ONE_PROBLEM_RE = re.compile(r"corpus/problems/[\w.-]+")
GIT_CHANGED_RE = re.compile(r"(?<![A-Za-z0-9])_?git\b[^|;]*\b(?:diff|status)\b")


def _names_corpus_set(line: str) -> bool:
    """A line naming the problem set, not one named problem or a pathspec over
    the files git says changed."""
    if GIT_CHANGED_RE.search(line):
        return False
    return bool(re.search(r"(?<![\[&])[\"']problems[\"']", line)) or "corpus/problems" in ONE_PROBLEM_RE.sub("", line)


# A comment can name the corpus but can't walk it.
COMMENT_RE = re.compile(r"\s*(?://|#(?!!)|\*)")

# The walker, its test's fixture corpus, and this detector and its test.
CORPUS_OWN = {"scripts/corpus.py", "crates/cpl-corpus/src/lib.rs", "scripts/test_corpus.py", "scripts/debt-ratchet.py", "scripts/test_debt_ratchet.py"}


def corpus_walkers(root: Path) -> list[str]:
    """Files besides scripts/corpus.py that list corpus/problems themselves: a second corpus walker."""
    found = []
    for rel in _git(root, "ls-files", "*.py", "*.sh", "*.rs", "*.js", "*.mjs", "*.ts", "justfile"):
        if rel in CORPUS_OWN or rel.startswith(("research/", "corpus/problems/")) \
                or "node_modules/" in rel or not (root / rel).is_file():
            continue
        lines = _lines(root / rel)
        for i, line in enumerate(lines):
            if _names_corpus_set(line) and not COMMENT_RE.match(line):
                if any(WALK_RE.search(l) and not GIT_CHANGED_RE.search(l) for l in lines[max(0, i - 3):i + 4]):
                    found.append(rel)
                    break
    return found

def _outside(rel: str, owners: tuple[str, ...]) -> bool:
    return not rel.startswith(owners)


JSON_FORMAT_RE = re.compile(r'format!\("(?:\{\{\\"|[^"]*\\":)')
# A JSON key inside any string literal: `{{\"` (a format brace before a key) or a
# compact `\":`, whether the literal is a format!/write! argument on its own line
# or a push_str piece.
JSON_LITERAL_RE = re.compile(r'\{\{\\"|\\":(?! )')


def _hand_json(line: str) -> bool:
    return bool(JSON_FORMAT_RE.search(line)) or any(
        JSON_LITERAL_RE.search(lit) for lit in RUST_LITERAL_RE.findall(line))


def json_outside_cpl_json(root: Path) -> list[str]:
    """`serde_json` in a Cargo.toml or a hand-built JSON `format!` outside crates/cpl-json:
    a second machine-output schema."""
    found = [rel for rel in _git(root, "ls-files", "Cargo.toml", "crates/*/Cargo.toml")
             if _outside(rel, ("crates/cpl-json/",)) and "serde_json" in (root / rel).read_text(encoding="utf-8")]
    return found + [f"{rel}:{i}" for rel, i, line in _src_lines(root)
                    if _outside(rel, ("crates/cpl-json/",)) and _hand_json(line)]


# The one config module, named by its owner (Consumers' tools child) once it exists.
# The one reader of cp.toml and its schema.
CONFIG_MODULES: tuple[str, ...] = ("crates/cpl-config/",)
CONFIG_RE = re.compile(r'"cp\.toml"|\btoml::')


BAR = 1.1
BAR_RECORD = "research/perf/v03-bar.txt"
BAR_CMD = "python3 bench/v03_perf_bench.py --suite corpus --live --record"


def _bar(root: Path) -> list[tuple[str, float, float, bool]]:
    """BAR_RECORD (`bench/v03_perf_bench.py --suite corpus --live --record`): each timed port's
    time and peak-RSS ratio against orig.cpp at --max, and whether port.cp is still the one timed."""
    p = root / BAR_RECORD
    if not p.is_file() and any((root / "corpus/problems").glob("*/orig.cpp")):
        raise MeasureError(f"{BAR_RECORD} is missing: run {BAR_CMD}")
    rows = []
    for row in (r.split() for r in (_lines(p) if p.is_file() else []) if r.strip() and not r.startswith("#")):
        port = root / "corpus/problems" / row[0] / "port.cp"
        if port.is_file():
            current = hashlib.sha256(port.read_bytes()).hexdigest()[:12] == row[1]
            rows.append((row[0], float(row[2]), 0.0 if row[3] == "-" else float(row[3]), current))
    return rows


def ports_over_bar(root: Path) -> list[str]:
    """Ports over 1.1x orig.cpp on time or peak RSS at --max (perf). A port edited since it was
    timed keeps its row until BAR_CMD times it again, so an edit alone can't clear it."""
    return [f"{n} time {t}x rss {m}x" + ("" if cur else f" (port.cp edited since: run {BAR_CMD})")
            for n, t, m, cur in _bar(root) if t > BAR or m > BAR]


def perf_twins_stale(root: Path) -> list[str]:
    """perf.cp twins kept beside a port.cp that, as it is now, was timed within the bar."""
    return [f"corpus/problems/{n}/perf.cp: port.cp time {t}x rss {m}x" for n, t, m, cur in _bar(root)
            if cur and t <= BAR and m <= BAR and (root / "corpus/problems" / n / "perf.cp").is_file()]


def config_schemas(root: Path) -> list[str]:
    """Shipped files reading `cp.toml` or using `toml::` outside the one config module."""
    return sorted({rel for rel, _, line in _src_lines(root)
                   if CONFIG_RE.search(line) and (not CONFIG_MODULES or _outside(rel, CONFIG_MODULES))})


def std_doc_parsers(root: Path) -> list[str]:
    """`"///` literals outside the syntax crates and cpl-doc: a second reader of std doc
    comments. The one reader is `cpl_next_syntax::doc`."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root)
            if '"///' in line
            and _outside(rel, ("crates/cpl-syntax/", "crates/cpl-next-syntax/", "crates/cpl-doc/"))]


# A text edit engine: the fix appliers by name, any fn that takes a
# `&str` and `(Span, String)` edits, or an in-place `replace_range`.
EDIT_ENGINE_RE = re.compile(
    r"\bfn apply_fixes\b|\bfn apply_edits\b|\bfn \w+\([^)]*&str[^)]*\(Span, String\)|\.replace_range\(")


def edit_engines(root: Path) -> list[str]:
    """Text edits applied outside cpl-diag's `apply_fixes` and cpl-fmt: a second edit engine."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root)
            if EDIT_ENGINE_RE.search(line) and _outside(rel, ("crates/cpl-diag/", "crates/cpl-fmt/"))]


def codemods_outside_fix(root: Path) -> list[str]:
    """Scripts that rewrite the `.cp` files they glob, instead of going through `cpl fix`."""
    found = []
    for rel in _git(root, "ls-files", "scripts/*.py", "scripts/*.sh"):
        if Path(rel).name.startswith("test_"):
            continue
        text = (root / rel).read_text(encoding="utf-8", errors="replace")
        for v in sorted(set(re.findall(r"for (\w+) in [^\n]*\*\.cp['\"]", text))):
            if re.search(rf"\b{v}\.write_text\(|open\({v},\s*['\"]w|sed -i[^\n]*\${{?{v}\b", text):
                found.append(f"{rel} {v}")
    return found


def module_loaders(root: Path) -> list[str]:
    """`Database::new(` outside cpl-check: a second module loader beside `SessionOptions`."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root)
            if "Database::new(" in line and _outside(rel, ("crates/cpl-check/",))]


WEB_STRING_ARRAY_RE = re.compile(r"\[([^\[\]]*)\]", re.S)
WEB_ARRAY_ITEM_RE = re.compile(r"""^\s*['"`]([^'"`\n]+)['"`]\s*$""", re.M)


def web_hand_copied(root: Path) -> list[str]:
    """Arrays of four or more strings in web/ sources, at least half of them tool data:
    pub std names, language keywords (editors/vscode/src/languageData.ts) or diagnostic
    codes. The site reads those from `cpl doc --json`, `cpl explain --json` and the
    editor tables instead of keeping a copy."""
    names: set[str] = set()
    for p in sorted((root / "std").glob("*.cp")):
        names.update(re.findall(r"^\s*pub\s+(?:fn|struct|enum|trait|type)\s+(\w+)", p.read_text(), re.M))
    data = root / "editors/vscode/src/languageData.ts"
    if data.is_file():
        names.update(re.findall(r"'(\w+)'", data.read_text(encoding="utf-8").partition("CONTEXTUAL_KEYWORDS")[0]))
    out = []
    for rel in _git(root, "ls-files", "web"):
        if not WEB_SOURCE_RE.match(rel) or not (root / rel).is_file():
            continue
        text = (root / rel).read_text(encoding="utf-8", errors="replace")
        for m in WEB_STRING_ARRAY_RE.finditer(text):
            items = WEB_ARRAY_ITEM_RE.findall(m.group(1).replace(",", "\n"))
            hits = [i for i in items if i in names or re.fullmatch(r"[EWL]\d{4}", i)]
            if len(items) >= 4 and 2 * len(hits) >= len(items):
                out.append(f"{rel}:{text.count(chr(10), 0, m.start()) + 1}")
    return out


DIAG_SITE_RE = re.compile(r"\bDiagnostic::(?:error|warning)\(")


# A crate's own funnel that takes message text and builds the diagnostic itself:
# each call passes text the registry should hold, so each call counts. A
# `said.error(span)` renders a registry row's `Said`, which only cpl-diag builds.
DIAG_TEXT_FUNNELS = {
    "crates/cpl-check/src": re.compile(r"\b(?!said\.)[a-z_][a-z0-9_]*\.error\("),
    "crates/cpl-ir/src": re.compile(r"\b(?!said\.)[a-z_][a-z0-9_]*\.error\("),
    "crates/cpl-lint/src": re.compile(r"\b[a-z_][a-z0-9_]*\.warn\("),
}


def diag_text_outside_registry(root: Path) -> list[str]:
    """Diagnostics built with inline message text outside cpl-diag: each one's wording
    belongs in a cpl_diag::msgs row, and the emit site names the row plus its
    arguments. Counts each `Diagnostic::error`/`warning` call and each call of a
    crate's text-taking funnel (`Checker::error`, lower's `error`, lint `warn`)."""
    out = []
    for rel, i, line in _src_lines(root):
        if not _outside(rel, ("crates/cpl-diag/",)):
            continue
        funnel = next((r for d, r in DIAG_TEXT_FUNNELS.items() if rel.startswith(d + "/")), None)
        if DIAG_SITE_RE.search(line) or (funnel and funnel.search(line)):
            out.append(f"{rel}:{i}")
    return out


TRAP_SITE_RE = re.compile(r"\btrap(?:::<[^>]*>)?\(|(?<!impl )(?<!struct )\bTrap \{|\.fail\(|\b(?:overflow|div_zero)\(")
# A string literal with words in it once its `{..}` holes are gone: message text.
TEXT_LITERAL_RE = re.compile(r'"((?:\\.|[^"\\])*)"')


def _call_args(lines: list[str], at: int, col: int) -> str:
    """From `lines[at][col:]`, the text up to the bracket that closes the first one opened."""
    depth, out = 0, []
    for line in lines[at:at + 40]:
        for ch in line[col:]:
            out.append(ch)
            if ch in "([{":
                depth += 1
            elif ch in ")]}":
                depth -= 1
                if depth == 0:
                    return "".join(out)
        col = 0
        out.append("\n")
    return "".join(out)


def trap_text_outside_registry(root: Path) -> list[str]:
    """Runtime trap messages written inline in cpl-interp: a trap's text is a
    user-facing diagnostic (run, trace and explain show it), so it belongs in a
    cpl_diag::msgs row like any other message. A site counts when its arguments
    hold a string literal with words, rather than a `msgs::interp` row."""
    by_file: dict[str, list[tuple[int, str]]] = {}
    for rel, i, line in _src_lines(root, "crates/cpl-interp"):
        by_file.setdefault(rel, []).append((i, line))
    sites = []
    for rel, numbered in by_file.items():
        lines = [line for _, line in numbered]
        for k, (i, line) in enumerate(numbered):
            if re.search(r"\bfn (?:trap|fail|overflow|div_zero)\b", line):
                continue
            for m in TRAP_SITE_RE.finditer(line):
                args = _call_args(lines, k, m.end() - 1)
                if any(re.search(r"[A-Za-z]{2}", re.sub(r"\{[^{}]*\}", "", t))
                       for t in TEXT_LITERAL_RE.findall(args)):
                    sites.append(f"{rel}:{i}")
                    break
    return sites


# The languages whose habits the compiler recovers, as a doc comment names one: `Python's
# \`a.index(x)\``, `the C++/Rust spelling`, `a lambda from Kotlin`.
FOREIGN_LANG = r"(?:C\+\+|C|Python|Rust|Java|Kotlin|JavaScript|TypeScript|Swift|Lua|Pascal|Go)"
FOREIGN_HABIT_DOC_RE = re.compile(r"(?<![\w`])" + FOREIGN_LANG + r"(?:'s\b|/(?=[A-Z]))|\bfrom " + FOREIGN_LANG + r"(?![\w`+])")
FN_ITEM_RE = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?fn\s")
RECOVERY_CRATES = ("crates/cpl-syntax/src", "crates/cpl-next-syntax/src", "crates/cpl-check/src", "crates/cpl-ir/src")


def foreign_outside_modules(root: Path) -> list[str]:
    """Fns in the parser, checker and lowering whose doc comment names another language's
    habit they recover (`Python's ...`, `C++/Rust ...`, `from Kotlin`), outside a `foreign/`
    module (or a `foreign.rs`): each language's recovery lives in its own module, so it is
    found, and shrinks, in one place. Tests are left out."""
    found = []
    for p in _rs_files(root, *RECOVERY_CRATES, tests=False):
        rel = _rel(root, p)
        if "foreign" in Path(rel).parts[:-1] or p.stem == "foreign" or "/src/tests/" in rel \
                or p.name.endswith("_tests.rs") or p.name.startswith("tests_"):
            continue
        doc: list[str] = []
        for i, line in _ungated_lines(_lines(p)):
            s = line.strip()
            if s.startswith("///"):
                doc.append(s[3:])
                continue
            if s.startswith("#["):
                continue
            if FN_ITEM_RE.match(s) and FOREIGN_HABIT_DOC_RE.search(" ".join(doc)):
                found.append(f"{rel}:{i}")
            doc = []
    return found


# The VM's one event model: the `observe::Observer` its `tracer` slot holds.
EVENT_SINK_TYPE_RE = re.compile(r"^\s*(?:pub(?:\(crate\))?\s+)?(\w+):\s*Option<[^>]*(?:Profiler|Stats|Recorder|Counter|Observer)\b")


OBSERVER_HOOK_RE = re.compile(r"^\s*fn\s+(\w+)\s*\(\s*&mut\s+self\b")


def event_models(root: Path) -> list[str]:
    """A second way for the interpreter to report what a run does. Uses of an interpreter
    field that collects execution events beside the one `tracer: Option<Box<O: Observer>>`
    slot (a profiler, VM stats, a recorder), and each `&mut self` hook of `trait Observer`
    beside `on(Event)` (and `profile`, which takes the result): trace, profiler, debugger
    and replay hear one typed event stream, `event::Event`."""
    found: list[str] = []
    observe = root / "crates/cpl-interp/src/observe.rs"
    if observe.is_file():
        inside = False
        for i, line in enumerate(_lines(observe), 1):
            if re.match(r"^(?:pub(?:\(crate\))?\s+)?trait Observer\b", line):
                inside = True
            elif inside and line.startswith("}"):
                inside = False
            elif inside and (m := OBSERVER_HOOK_RE.match(line)) and m.group(1) not in {"on", "profile"}:
                found.append(f"crates/cpl-interp/src/observe.rs:{i}")
    lib = root / "crates/cpl-interp/src/lib.rs"
    if not lib.is_file():
        return found
    sinks = {m.group(1) for line in _lines(lib) if (m := EVENT_SINK_TYPE_RE.match(line))} - {"tracer"}
    if not sinks:
        return found
    use = re.compile(r"\.(?:%s)\b" % "|".join(sorted(sinks)))
    return found + [f"{rel}:{i}" for rel, i, line in _src_lines(root, "crates/cpl-interp")
                    if use.search(line)]


WIDTH_ARM_RE = re.compile(r'"(?:[IU](?:8|16|32)|u?int(?:8|16|32|64)_t)"\s*(?:=>|\|)')
WIDTH_TABLE = "crates/cpl-ir/src/lib.rs"  # `IntWidth`


def width_tables(root: Path) -> list[str]:
    """Match arms keyed on a narrow integer width's name (`"U8" =>`, `"int32_t" |`) outside
    `cpl_ir::IntWidth`: a second width table for the checker, lints, VM or codegen."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root)
            if WIDTH_ARM_RE.search(line) and rel != WIDTH_TABLE]


JUDGE_NAMES = ("Codeforces", "AtCoder", "Cses", "Kattis", "Dmoj", "Usaco", "Yosupo", "Luogu")
JUDGE_RE = re.compile(r'"(?:%s)"|^\s*(?:%s)\s*,?\s*$|::(?:%s)\b' % (
    "|".join(n.lower() for n in JUDGE_NAMES), "|".join(JUDGE_NAMES), "|".join(JUDGE_NAMES)))


def judge_tables(root: Path) -> list[str]:
    """Judges named outside crates/cpl-judge's table, by id literal or enum variant:
    `cpl fetch`, the emitter and the CLI all read the one judge/target table."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root)
            if JUDGE_RE.search(line) and _outside(rel, ("crates/cpl-judge/",))]


# The editors' LSP-client wiring and language registration; everything else is logic
# the language server should own.
EDITOR_GLUE = frozenset({
    "editors/vscode/src/extension.ts", "editors/vscode/src/client.ts", "editors/vscode/src/languageData.ts",
    "editors/jetbrains/src/main/kotlin/dev/cplang/ide/CpLanguage.kt",
    "editors/jetbrains/src/main/kotlin/dev/cplang/ide/CpLsp.kt",
    "editors/jetbrains/src/main/kotlin/dev/cplang/ide/CpLsp4ij.kt",
    "editors/zed/src/lib.rs", "editors/neovim/ftdetect/cp.vim", "editors/neovim/ftplugin/cp.vim",
    "editors/neovim/plugin/cplang.lua", "editors/vim/plugin/cplang.vim", "editors/vim/ftplugin/cp_lsp.vim", "editors/emacs/cp-mode.el",
})
EDITOR_SOURCE_RE = re.compile(r"^editors/(?!tree-sitter-cp/)(?!.*(?:/tests?/|/scripts/|smoke|\.test\.)).*\.(?:ts|js|kt|rs|lua|el|vim)$")
EDITOR_COMMENT = ("//", "/*", "*", "#", ";", "--", '"')
# In an `EDITOR_GLUE` file a line is client wiring unless it decides or computes:
# a branch, a loop, a condition, or a search or transform of data (strings blanked).
# A `case` that is an object key (`{ case: name }`) or a member (`args.case`) is data.
EDITOR_DECIDES_RE = re.compile(r"\b(?:if|else|for|while|switch|catch|when)\b|(?<!\.)\bcase\b(?!\s*:)|\?\?|\?:|&&|\|\||\s\?\s"
                               r"|\.(?:map|filter|find|flatMap|sort|reduce|split|replace|match|test|some|every|includes)\("
                               r"|(?<!\.)\b(?:and|or|not|elseif)\b|\bvim\.(?:tbl_\w+|split|startswith)\(|:(?:match|gsub|find|sub)\(")
EDITOR_IMPL_RE = re.compile(r"^\s*impl\b.*\bfor\b")  # Rust's `impl Trait for Type` is no loop
EDITOR_STRING_RE = re.compile(r"'(?:\\.|[^'\\])*'|\"(?:\\.|[^\"\\])*\"|`(?:\\.|[^`\\])*`")


# `if (event.affectsConfiguration('<launch section>')) { client.restart(); }` is process
# lifecycle: the server restarts when the settings that start it change.
EDITOR_RESTART_IF_RE = re.compile(r"^\s*if \(\w+\.affectsConfiguration\('([\w.]+)'\)\) \{?\s*(.*)$")
EDITOR_RESTART_RE = re.compile(r"^\s*(?:void )?\w+\.restart\(\);?\s*\}?\s*$")


def _launch_sections(root: Path, editor: str, files: list[str]) -> set[str]:
    """The settings sections (`id`s) of an editor's package.json whose every setting is
    under the section and read only in the file that makes its LSP client: launch settings."""
    try:
        sections = json.loads((root / editor / "package.json").read_text())["contributes"]["configuration"]
    except (OSError, ValueError, KeyError, TypeError):
        return set()
    texts = {rel: (root / rel).read_text(errors="replace") for rel in files
             if rel.startswith(editor + "/src/") and (root / rel).is_file()}
    client = {rel for rel, text in texts.items() if "new LanguageClient(" in text}
    out = set()
    for section in sections if isinstance(sections, list) else [sections]:
        sid, keys = section.get("id", ""), list(section.get("properties", {}))
        reads = [{rel for rel, text in texts.items() if f"'{key.split('.', 1)[-1]}'" in text} for key in keys]
        if sid and keys and all(key.startswith(sid + ".") for key in keys) and all(r and r <= client for r in reads):
            out.add(sid)
    return out


def _restarts_on_launch_change(line: str, after: list[str], sections: set[str]) -> bool:
    m = EDITOR_RESTART_IF_RE.match(line)
    return bool(m) and m.group(1) in sections and bool(EDITOR_RESTART_RE.match(m.group(2) or "".join(after[:1])))


def editor_logic_lines(root: Path) -> list[str]:
    """Code lines in git-tracked editor sources (never node_modules/, out/ or dist/ a build
    leaves) other than LSP-client wiring: every code line,
    and in an `EDITOR_GLUE` file each line that decides or computes (`EDITOR_DECIDES_RE`)
    other than a restart when the launch settings change (`_launch_sections`).
    Editors stay thin, with completion, running, results and tracing in the server."""
    out, files, launch = [], _git(root, "ls-files", "editors"), {}
    for rel in files:
        if EDITOR_SOURCE_RE.match(rel) and (root / rel).is_file():
            glue, editor = rel in EDITOR_GLUE, "/".join(rel.split("/")[:2])
            if glue and editor not in launch:
                launch[editor] = _launch_sections(root, editor, files)
            lines = _lines(root / rel)
            out += [f"{rel}:{i}" for i, line in enumerate(lines, 1)
                    if line.strip() and not line.strip().startswith(EDITOR_COMMENT)
                    and (not glue or (EDITOR_DECIDES_RE.search(EDITOR_IMPL_RE.sub("", EDITOR_STRING_RE.sub("''", line)))
                                      and not _restarts_on_launch_change(line, lines[i:i + 1], launch[editor])))]
    return out



PASS_ROW_RE = re.compile(r'\bpass\(\s*"([\w-]+)",\s*"([\w/.]+)",\s*(\w+)')
IR_WALK_RE = re.compile(r"\bdeep_\w+\(")


def _private_walks(root: Path) -> list[tuple[str, int]]:
    """Each line of codegen outside uses/ that walks IR bodies itself (a `deep_*` call)."""
    src = root / "crates/cpl-codegen/src"
    return [(rel, i) for rel, i, line in _src_lines(root, "crates/cpl-codegen/src")
            if IR_WALK_RE.search(line) and not (root / rel).is_relative_to(src / "uses")
            and (root / rel) != src / "walk.rs"]


def opt_without_uses(root: Path) -> list[str]:
    """Codegen that decides what code does without the Uses table: each `Unasked` row of the
    PASSES table (crates/cpl-codegen/src/passes.rs), each module that walks IR bodies with no
    row there, and each line outside uses/ that walks IR bodies itself (a `deep_*` call), even
    in a pass whose row asks the table: that walk belongs in uses/ as a named query every pass
    can ask. Every optimization asks the effects/Uses table what code does."""
    src = root / "crates/cpl-codegen/src"
    table = src / "passes.rs"
    rows = PASS_ROW_RE.findall(table.read_text(encoding="utf-8")) if table.is_file() else []
    out = [f"{_rel(root, src / module)}: {name} asks no Uses query"
           for name, module, evidence in rows if evidence == "Unasked"]
    listed = {_rel(root, src / module) for _, module, _ in rows}
    walks = _private_walks(root)
    out += [f"{rel}: walks IR with no PASSES row" for rel in sorted({rel for rel, _ in walks} - listed)]
    out += [f"{rel}:{i}: walks IR itself, not through a named uses/ query" for rel, i in walks]
    return sorted(out)


# Every tracked list file outside test data: the counter that counts its
# entries (None for a data list that exempts nothing), its owner, and how it
# goes away. A new list without a row here fails `unregistered_lists`.
LISTS: dict[str, tuple[str | None, str, str]] = {
    "corpus/gold/GOLD.txt": (None, "idioms", "data: the gold set"),
    "corpus/newcomer/acceptance.txt": (None, "idioms", "data: newcomer acceptance expectations"),
    "corpus/sets/ecnerwala.txt": (None, "idioms", "data: a corpus set"),
    "corpus/sets/sketches.txt": (None, "idioms", "data: a corpus set"),
    "corpus/sets/v03.txt": (None, "idioms", "data: a corpus set"),
    "crates/cpl-cli/tests/std-check-errors.txt": ("known_lists", "stdlib", "kept empty"),
    "crates/cpl-cli/tests/tool-gaps.txt": ("skip_lists", "tools", "deleted: tool_matrix.rs fails on any gap"),
    "crates/cpl-cli/tests/vm_fallbacks.txt": ("vm_fallbacks", "tech-debt", "VM runs every program"),
    "crates/cpl-conform/silent_unknowns.txt": ("silent_unknowns", "core", "deleted: the checker types every expression or reports why not"),
    "crates/cpl-lint/corpus_warnings.txt": ("corpus_lint_warnings", "lint-fmt", "deleted: corpus_tests.rs fails on any std or spec.cp warning"),
    "crates/cpl-next/tests/smoke.txt": (None, "core", "data: the commit-tier smoke sample each backend runs"),
    "crates/cpl-next/tests/passing.txt": ("next_not_passing", "core", "every case listed: the landing that switches `cpl` to cpl-next deletes the list"),
    "tests/old-parser-accepts.txt": ("next_old_parser_accepts", "core", "deleted in stage 5's switching landing, with the old parser that accepts these forms"),
    "crates/cpl-next/tests/lowered.txt": ("next_lowered", "core", "deleted: programs.rs fails on any error only lowering finds"),
    "crates/cpl-next/tests/error-typed.txt": ("next_error_typed", "core", "deleted: the verifier refuses any instance holding an error type"),
    "crates/cpl-next/tests/std-unchecked.tsv": ("next_std_unchecked", "core", "deleted: std_modules.rs fails on any std module that does not compile"),
    "crates/cpl-next/tests/wrong.tsv": ("next_wrong", "core", "deleted: programs.rs fails on any wrong case"),
    "crates/cpl-next/tests/masked.txt": ("next_masked", "core", "deleted: no refusal hides a verdict beside it"),
    "crates/cpl-stress/tests/generate-gaps.txt": ("generate_gaps", "stress", "deleted: readable_shapes.rs fails on any shape stress can't generate"),
    "scripts/public-hosts.txt": (None, "infra", "data: hosts the public-history scan allows"),
    "scripts/std-lint-baseline.txt": ("std_lint_findings", "stdlib", "deleted: std-lint-ratchet.py fails on any hit"),
    "spec/conformance/permits.txt": ("permits", "spec", "a case per permit"),
    "spec/conformance/prelude-gaps.txt": ("prelude_gaps", "stdlib", "deleted: spec-gate.py prelude fails on any prelude gap"),
    "spec/conformance/rule-gaps.txt": ("rule_gaps", "spec", "deleted: spec-gate.py coverage fails on any rule gap"),
    "spec/conformance/sig-gaps.txt": ("sig_gaps", "spec", "deleted: spec-gate.py sigs fails on any mismatch"),
    "spec/conformance/trait-gaps.txt": ("trait_gaps", "spec", "deleted: spec-gate.py traits fails on any missing case"),
    "spec/idioms/lints-known.txt": ("idiom_lint_gaps", "idioms", "deleted: idioms-catalog.py fails on any finding"),
    "web/scripts/doc-examples-failing.txt": ("skip_lists", "site", "deleted: gen-ref.mjs fails on any std example that doesn't run"),
    "web/scripts/practice-mutants-equivalent.txt": ("known_lists", "site", "deleted: practice-mutants.mjs fails on any surviving mutant"),
    "web/tests/a11y-known.txt": ("xfail_markers", "site", "deleted: a11y.mjs fails on any violation"),
    "web/tests/completion-known.txt": ("known_lists", "site", "deleted: completion-parity.mjs fails on any miss"),
    "web/tests/search-known.txt": ("known_lists", "site", "deleted: search-tasks.mjs fails on any miss"),
}
# Directories of test data and fixtures, not lists of exemptions.
LIST_DATA_DIRS = ("research/", "corpus/problems/", "corpus/submissions/", "bench/stress/", "node_modules/")
LIST_NAME_RE = re.compile(
    r"(known|gaps|failing|allow|xfail|skip|exempt|baseline|unbuilt|unmatched|oversize|permits|equivalent)"
    r"[^/]*\.(txt|json|toml|ya?ml)$")


def unregistered_lists(root: Path) -> list[str]:
    """Tracked list files with no `LISTS` row: a list must land with its counter, owner and plan."""
    found = []
    for rel in _git(root, "ls-files"):
        name = rel.rsplit("/", 1)[-1]
        top_txt = rel.endswith(".txt") and not rel.startswith(LIST_DATA_DIRS) \
            and not re.match(r"spec/conformance/[^/]+/|crates/[^/]+/tests/[^/]+/", rel)
        if (top_txt or LIST_NAME_RE.search(name) and not rel.startswith(LIST_DATA_DIRS)) and rel not in LISTS \
                and rel != "scripts/debt-ratchet.json":
            found.append(rel)
    found += [f"LISTS {rel}: no counter {c}" for rel, (c, _, _) in LISTS.items()
              if c is not None and c not in COUNTERS]
    return found


# Counters that are paid off: any instance fails, and no baseline, `--update` or
# main's own measurement can raise them above 0. Every gated counter whose
# committed baseline is 0 is listed here (scripts/test_debt_ratchet.py checks).
CP_BINDING = re.compile(r"\b(?:let|var|const|fn|type|struct|enum|for)\s+(?:mut\s+)?\(?([A-Za-z_]\w*)")


CP_CALL = re.compile(r"[A-Za-z_]\w*\s*\(|(?<=[\w)\]])\.[A-Za-z_]")


def idiom_edit_stateable(before: str, after: str) -> bool:
    """Whether a lint record's before -> after edit is what a `@lint(from:, to:)`
    pattern states (decision 512): each changed line run stays in one block (a
    statement run), no two runs change the same name bound in the file (then
    they are one edit at two places, not two findings), no run adds a `@!`
    directive (directives are core, not std items a pattern's `to` can call),
    and some run's new lines call a fn, method or getter: a `to` calls its std
    fn or a `builtin:` (R06.4.9), so a fix in core syntax alone (compound
    assignment, chained comparison, `while let`, a comprehension) has no std
    item for its @lint to sit on and stays Rust (R06.4.10)."""
    old = [line.rstrip() for line in before.split("\n")]
    new = [line.rstrip() for line in after.split("\n")]
    hunks = [o for o in difflib.SequenceMatcher(None, old, new, autojunk=False).get_opcodes() if o[0] != "equal"]
    bound = set(CP_BINDING.findall(before + "\n" + after))
    seen: set[str] = set()
    hosted = False
    for _, i1, i2, j1, j2 in hunks:
        if any(line.lstrip().startswith("@!") for line in new[j1:j2]):
            return False
        hosted |= any(CP_CALL.search(line) for line in new[j1:j2])
        run = [line for line in old[i1:i2] if line.strip()]
        if any(_indent(line) < _indent(run[0]) for line in run):
            return False
        names = bound & (set(re.findall(r"[A-Za-z_]\w*", " ".join(old[i1:i2])))
                         ^ set(re.findall(r"[A-Za-z_]\w*", " ".join(new[j1:j2]))))
        if names & seen:
            return False
        seen |= names
    return hosted


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip())


def rust_idiom_lints(root: Path) -> list[str]:
    """Idiom-group spec/lints records with no std `@lint(<name>, ...)`, so cpl-lint's Rust
    implements them, whose before.cp -> after.cp edit a from/to pattern could state."""
    std = "\n".join((root / rel).read_text(encoding="utf-8") for rel in _git(root, "ls-files", "std/*.cp", "std/**.cp"))
    declared = set(re.findall(r"@lint\(\s*(\w+)", std))
    out = []
    for toml in sorted((root / "spec/lints").glob("*/lint.toml")):
        d = toml.parent
        if (d.name in declared or not re.search(r'^group = "idiom"', toml.read_text(encoding="utf-8"), re.M)
                or not ((d / "before.cp").is_file() and (d / "after.cp").is_file())):
            continue
        if idiom_edit_stateable((d / "before.cp").read_text(encoding="utf-8"), (d / "after.cp").read_text(encoding="utf-8")):
            out.append(_rel(root, d))
    return out


HARD_ZERO = frozenset({"frozen_std_drift", "decls_const_op_arms", "next_standin_notes", "next_old_parser_accepts_unruled", "next_listed_twice", "next_repr_second_defs", "next_op_arms_outside_ir", "next_ownership_outside_own", "next_layer_foreign_deps",
    "next_oracle_callers", "next_ignored_settings", "event_labels","next_layout_outside_table", 
    "fast_unchecked", "next_lent_outside_own", "next_backend_owns_reads", "next_check_name_keys",
    "unreferenced_scripts", "explain_copy_mismatches", "link_reads_check", "tool_compiler_deps", "end_state_broken", "next_cases_removed",
    "cpp_deep_copies", "conformance_wrong_output", "cpp_sunk_copies",
    "habit_matchers", "corpus_fmt_dirty", "judge_tables", "v02_port_files", "reviews_uncapped", "lint_std_names", "cpl_env_reads", "dead_code_allows",
    "lint_name_matchers", "lint_type_syntax", "domain_member_names",
    "corpus_no_statement", "dup_fn_bodies", "near_dup_fns", "layer_violations", "event_models",
    "stdlib_todo", "editor_logic_lines",
    "uncached_installs",
    "corpus_walkers", "ci_jobs_without_recipe", "poll_sleeps", "retry_calls", "site_spec_snippets",
    "bench_lists", "hardcoded_codes", "scaling_mismatches", "std_fns_unscaled", "config_schemas", "corpus_test_files", "corpus_known", "corpus_no_max", "corpus_max_short", "corpus_max_unread", "corpus_no_limits", "stale_arch_paths",
    "own_databases", "string_keyed_maps", "module_loaders", "web_skip_only", "test_sleeps", "todo_fixme", "ci_retries", "cpp_std_functions", "or_true", "codemods_outside_fix",
    "pending_tests", "unchecked_fix_paths", "edition_refs", "uncited_rules", "decision_dup_numbers", "why_written_twice", "authority_drift",
    "continue_on_error", "unregistered_lists", "codegen_effect_guesses", "spec_std_refs", "opt_without_uses",
    "stress_hand_json", "lint_dup_helpers", "lint_post_filters",
    "measure_broken", "mono_unguarded", "ts_ignore", "ports_unreported",
    "site_rederived", "browser_lsp_gaps",
    "unrun_script_tests", "trap_text_outside_registry", "diag_text_outside_registry", "foreign_outside_modules",
    "permits", "written_twice", "reviews_orphan", "reviews_pending",
    "plan_delta_lines", "idioms_evidenced_unlinted", "v02_port_files", "judge_tables",
    "trait_gaps",
    "unwrap_input",
    "consumer_rederived",
    "std_trait_matrix",
    "edge_labels_unrun",
    "std_model_gaps",
    "idiom_lint_gaps",
    "invariant_sites",
    "rule_gaps",
    "direct_python",
    "sig_gaps",
    "std_lint_findings",
    "corpus_lint_warnings",
    "judge_tables",
    "std_lint_allows",
    "generate_gaps",
    "xfail_markers",
    "std_doc_parsers",
    "std_types_shown_empty",
    "prelude_gaps",
    "json_outside_cpl_json", "rust_ignore", "dream_unverified", "trace_engine_refs", "lsp_string_types", "known_lists", "skip_lists",
    "interp_max_over_20s",
    "edit_engines",
    "web_hand_copied",
    "corpus_no_tests", "cpp_optional_copies", "cpp_std_functions",
    "helper_env_vars",
    "timeout_passes",
    "stress_engines",
    "unwrap_input",
    "tests_unrun",
    "conformance_wrong_output",
})
# Paid down to the one place the thing lives: above the floor fails, with no baseline.
HARD_FLOOR = {"config_schemas": 1}  # cpl-config, the one cp.toml reader

# name -> (label, measure, ratcheted). Informational counts are shown, never gated.
COUNTERS = {
    "stdlib_todo": ("stdlib todo stubs", stdlib_todo, True),
    "std_gaps": ("open std/GAPS.md lines", std_gaps, True),
    "rust_ignore": ("Rust #[ignore] attributes", rust_ignore, True),
    "pending_tests": ("crates/cpl-cli/tests/pending/ files", pending_tests, True),
    "dead_code_allows": ("allow(dead_code) attributes", dead_code_allows, True),
    "external_crates": ("third-party packages in Cargo.lock", external_crates, True),
    "written_twice": ("hand-kept files listing most of a sourced fact", written_twice, True),
    "bench_lists": ("entries in bench/*.txt exemption lists (scoreboard K5)", bench_lists, True),
    "skip_lists": ("entries in skip, xfail, known-failure and gap lists", skip_lists, True),
    "corpus_test_files": ("files committed under corpus/problems/*/tests/", corpus_test_files, True),
    "corpus_no_max": ("non-interactive problem packages without a gen.py --max", corpus_no_max, True),
    "corpus_no_limits": ("problem ports without a `// limits:` header line (Content)", corpus_no_limits, True),
    "corpus_no_tests": ("problem packages without a tests.toml (Content)", corpus_no_tests, True),
    "corpus_no_statement": ("problem packages without an inline statement sample (Content)",
                            corpus_no_statement, True),
    "corpus_fmt_dirty": ("corpus port, dream and spec files cpl fmt would change (Content)",
                         corpus_fmt_dirty, True),
    "corpus_max_short": ("v03 --max inputs reading a size under the port's Bounds (generator's lane)",
                         corpus_max_short, True),
    "corpus_max_unread": ("v03 --max inputs whose sizes max-short can't read (Idioms)", corpus_max_unread, True),
    "dream_over_32kb": ("dream.cp files whose C++ is over 32 KB (Dreams)", dream_over_32kb, True),
    "interp_max_over_20s": ("corpus packages the interpreter runs a --max case past 20 s on (Core)", interp_max_over_20s, True),
    "tools_tests_over_30s": ("tools crates' tests over 30 s (Tools)", tools_tests_over_30s, True),
    "edition_refs": ("dead 0.2 edition references", edition_refs, True),
    "ir_builtin_names": ("names string-matched in lower/call.rs and lower/call/", ir_builtin_names, True),
    "ir_builtin_derived": ("string-matched names off the decision-266 floor table", ir_builtin_derived, True),
    "prelude_privilege": ("public std names in cpl-check prelude.rs tables", prelude_privilege, True),
    "backend_std_names": ("std type names in IR/interp/codegen", backend_std_names, True),
    "std_names_outside_rt": ("public std names as string literals in compiler source, outside cpl-rt",
                             std_names_outside_rt, True),
    "domain_member_names": ("std `@member` domain member names as string literals in compiler source",
                            domain_member_names, True),
    "lint_std_names": ("std fn names as string literals in lint rules, not through std_fn", lint_std_names, True),
    "lint_name_matchers": ("lint rule lines matching a call by its written std fn name, not its callee", lint_name_matchers, True),
    "lint_type_syntax": ("lint rule lines reading a written type instead of checker facts", lint_type_syntax, True),
    "idioms_evidenced_unlinted": ("non-idiomatic spots with evidence that no lint teaches", idioms_evidenced_unlinted, True),
    "dup_fn_bodies": ("copied fn bodies across files", dup_fn_bodies, True),
    "near_dup_fns": ("near-identical fns across crates", near_dup_fns, True),
    "consumer_rederived": ("tool sites re-deriving checker facts", consumer_rederived, True),
    "site_rederived": ("web/ sources re-deriving tool data", site_rederived, True),
    "browser_lsp_gaps": ("language-server capabilities the browser editor never asks for", browser_lsp_gaps, True),
    "site_spec_snippets": ("site programs the compiler doesn't accept yet (.spec.cp, status=spec)", site_spec_snippets, True),
    "layer_violations": ("forbidden shipped crate edges", layer_violations, True),
    "tool_compiler_deps": ("tool crate edges and code lines that reach a compiler crate past the facade (stage 2; cpl-syntax is stage 8)", tool_compiler_deps, True),
    "facade_today_uses": ("tool code lines reading the compiler's own forms through cpl_facade::today or a root re-export (contract 9)", facade_today_uses, True),
    "tree_walker": ("second interpreter engine", tree_walker, True),
    "v02_port_files": ("corpus ports not yet re-ported in v0.3 style", v02_port_files, True),
    "ports_unreported": ("corpus ports without a friction report or review (idioms)", ports_unreported, True),
    "reviews_pending": ("taste reviews still at Score: pending (idioms)", reviews_pending, True),
    "reviews_orphan": ("taste reviews whose package has no port or dream (idioms)", reviews_orphan, True),
    "reviews_uncapped": ("reviews scoring a port with no dream above 3 (idioms)", reviews_uncapped, True),
    "direct_python": ("direct python3 calls in gate/justfile", direct_python, True),
    "unrun_script_tests": ("script tests the gate never runs", unrun_script_tests, True),
    "unreferenced_scripts": ("files under scripts/ nothing else names (Infra)", unreferenced_scripts, True),
    "tests_unrun": ("workspace Rust tests no CI tier runs", tests_unrun, True),
    "decision_dup_numbers": ("decision numbers headed twice in decisions.md", decision_dup_numbers, True),
    "lint_dup_helpers": ("helper fns copied across lint rule files", lint_dup_helpers, True),
    "plan_delta_lines": ("lines of notes under plan/domains/*/delta", plan_delta_lines, True),
    "why_written_twice": ("reasoning written in two homes, or rationale under spec/", why_written_twice, True),
    "authority_drift": ("authority sentence missing from a home, or contradicted", authority_drift, True),
    "conformance_failing": ("conformance cases not passing in research/scoreboard.json (Content)", conformance_failing, True),
    "conformance_wrong_output": ("conformance cases with wrong output in research/scoreboard.json (Content)",
                                 conformance_wrong_output, True),
    "vm_fallbacks": ("source call sites that route the VM into the tree-walker", vm_fallbacks, True),
    "next_wrong": ("cases the new pipeline (cpl-next) gets wrong (Core)", next_wrong, True),
    "end_state_unmeasured": ("suites of the end state (plan/end-state.toml) with no number yet (Core)", end_state_unmeasured, True),
    "end_state_broken": ("end-state suites malformed, removed or retargeted with no `changed` note, or a stale "
                         "plan/END-STATE.md (Core)", end_state_broken, True),
    "next_cases_removed": ("conformance cases or new-pipeline tests deleted with no line in "
                           "spec/conformance/REMOVED.md (Core)", next_cases_removed, True),
    "next_std_names": ("library declarations the new pipeline (cpl-next) knows by name (Core)", next_std_names, True),
    "next_refusal_kinds": ("kinds of feature the new pipeline (cpl-next) refuses: `Unbuilt` variants (Core)", next_refusal_kinds, True),
    "frozen_std_drift": ("files of the old pipeline's std that are not std/ at 6a31a3747^ (Core)", frozen_std_drift, True),
    "old_pipeline_lines": ("code lines in the frozen old checker, lowering, interpreter and emitter (Core)", old_pipeline_lines, True),
    "old_std_copy_lines": ("code lines of the pinned std copy the frozen old pipeline reads (Core)", old_std_copy_lines, True),
    "next_not_passing": ("cases the new pipeline (cpl-next) does not pass yet: wrong or refused (Core)", next_not_passing, True),
    "next_refused": ("cases the new pipeline (cpl-next) refuses: on no list (Core)", next_refused, True),
    "next_masked": ("cases the new pipeline (cpl-next) refuses beside a coded verdict on the program (Core)", next_masked, True),
    "next_lowered": ("programs cases whose error only lowering finds (Core)", next_lowered, True),
    "next_old_parser_accepts": ("rejected-form cases cpl-next accepts through the old parser, each ruled (Core; reported)", next_old_parser_accepts, False),
    "next_old_parser_accepts_unruled": ("old-parser accepts citing no decision or not expecting E0503/E0504 (Core)", next_old_parser_accepts_unruled, True),
    "next_error_typed": ("programs cases linking an instance that holds an error type (Core)", next_error_typed, True),
    "next_std_unchecked": ("std modules the new pipeline does not compile, every fn checked (Core)", next_std_unchecked, True),
    "next_core_privilege": ("checks giving core's module powers no library has in the new pipeline (Core)", next_core_privilege, True),
    "next_whole_program_inputs": ("new-pipeline params, fields, closures and holder-struct methods that hold a whole-program value (Core)", next_whole_program_inputs, True),
    "next_handoff_side_fields": ("fields reaching link beside the IR, through any struct or tuple of the handoff (Core)", next_handoff_side_fields, True),
    "next_check_set_facts": ("checker sites building a function-value set fact link should own (Core)", next_check_set_facts, True),
    "next_snapshot_keyed_queries": ("facade answers memoized per whole snapshot, not per item (Core)", next_snapshot_keyed_queries, True),
    "next_reference_evaluator_lines": ("code lines of the new pipeline's reference interpreter (Core)", next_reference_evaluator_lines, True),
    "next_harness_refusals": ("sites in the new harness that answer refused themselves instead of the pipeline (Core; target 0)", next_harness_refusals, True),
    "next_oracle_callers": ("non-test callers of the new pipeline's reference interpreter, the tests' oracle (Core)", next_oracle_callers, True),
    "next_layout_outside_table": ("lines in the new engine or C++ emitter that read struct fields or union members around the IR's one layout table (Core)", next_layout_outside_table, True),
    "next_standin_notes": ("new-pipeline lines saying the code beside them stands in for a designed component (Core)", next_standin_notes, True),
    "next_source_text_guards": ("new-pipeline guards that read Rust source as text through include_str! (Core)", next_source_text_guards, True),
    "decls_const_op_arms": ("per-operator TokenKind arms in cpl-next-decls, which evaluates constants through ir's ops (Core)", decls_const_op_arms, True),
    "next_synthetic_name_tests": ("cpl-next checker tests of a name's text for the synthetic `$` (Core)", next_synthetic_name_tests, True),
    "next_op_arms_outside_ir": ("new-pipeline `Intr::` match arms outside cpl-next-ir that are not a back end's machine primitive, or a primitive a back end lacks (Core)", next_op_arms_outside_ir, True),
    "next_repr_second_defs": ("new-pipeline definitions of a representation predicate (unlent, plain, stores) outside the IR's one classifier (Core)", next_repr_second_defs, True),
    "next_core_standin_lines": ("lines of the new pipeline's private core stand-in std.cp (Core)", next_core_standin_lines, True),
    "next_cpp_runtime_lines": ("lines of hand-written C++ runtime in the new emitter (Core)", next_cpp_runtime_lines, True),
    "next_split": ("cases whose interpreter, engine and C++ runs disagree: one source of semantics broken (Core)", next_split, True),
    "next_wrong_accepts": ("cases the new pipeline accepts that must be rejected: soundness holes (Core)", next_wrong_accepts, True),
    "next_wrong_outputs": ("cases that run but print a wrong answer or miss a note: wrong answers judged as accepted (Core)", next_wrong_outputs, True),
    "next_listed_twice": ("cases on two committed verdict lists: the lists contradict themselves (Core)", next_listed_twice, True),
    "next_late_rejects": ("cases the spec accepts that link or own rejects with a user code: unbuilt features said as errors (Core)", next_late_rejects, True),
    "next_core_checks": ("new-pipeline code singling out core or a std module by id or name (Core)", next_core_checks, True),
    "next_ignored_settings": ("new-pipeline settings accepted and then ignored: an empty arm over source names (Core)", next_ignored_settings, True),
    "next_hidden_const_evaluators": ("constant evaluators outside cpl-next-consts, over the whole program (Core)", next_hidden_const_evaluators, True),
    "event_labels": ("new-pipeline events named by a label string instead of a typed point (Core)", event_labels, True),
    "trace_old_interp_uses": ("tool and facade lines reading the old interpreter's trace (Core)", trace_old_interp_uses, True),
    "next_name_compares": ("new-pipeline comparisons of source text against a fixed identifier: names known by text (Core)", next_name_compares, True),
    "next_library_gates": ("new-pipeline checks of whether a module is a library: std-only rules (Core)", next_library_gates, True),
    "next_ownership_outside_own": ("new-pipeline fns outside cpl-next-own deciding a lend or drop (Core)", next_ownership_outside_own, True),
    "next_weak_oracle_passes": ("new-pipeline passing cases whose oracle is exit 1 or ok without output (Core)", next_weak_oracle_passes, True),
    "syntax_std_names": ("string literals in cpl-syntax and cpl-next-syntax naming a std type (Core)", syntax_std_names, True),
    "next_backend_owns_reads": ("new-pipeline backend reads of whether a layout owns, placing a drop or retain (Core)", next_backend_owns_reads, True),
    "fast_unchecked": ("std @fast fns whose op no test compares with their CP body (Core)", fast_unchecked, True),
    "next_lent_outside_own": ("new-pipeline places after the checker, outside cpl-next-own, making a type lent (Core)", next_lent_outside_own, True),
    "next_asks_refused": ("conformance cases refused for a header the new harness does not judge (Core)", next_asks_refused, True),
    "next_layer_foreign_deps": ("compiler layer crates depending on a tool or build crate (Core)", next_layer_foreign_deps, True),
    "next_facade_split": ("facade queries the two facades do not share, and second Session/Snapshot types (Core)", next_facade_split, True),
    "next_run_api_unbuilt": ("contract 0/7/8 run and event members with no cpl-next counterpart (Core)", next_run_api_unbuilt, True),
    "old_parser_lines": ("code lines of the old parser `cpl-syntax/src/parser*`, deleted when the lossless tree's parser lands (Core; target 0)",
                         old_parser_lines, True),
    "next_syntax_old_refs": ("new-pipeline uses of the old parser's crate `cpl_syntax` and the names imported from it (Core; target 0)",
                             next_syntax_old_refs, True),
    "next_old_syntax_analyses": ("new-pipeline uses of semantic decisions kept in `cpl_syntax` (decorators, fall_through, visit, ...) (Core; target 0)",
                                 next_old_syntax_analyses, True),
    "tool_old_syntax_uses": ("tool-crate uses of the old syntax crate `cpl_syntax`, ported onto the lossless tree at stage 8 (Core; target 0)",
                             tool_old_syntax_uses, True),
    "own_databases": ("Database built outside cpl-check", own_databases, True),
    "ir_reinfer": ("lowering re-inferring a type the checker's facts give", ir_reinfer, True),
    "next_check_name_keys": ("checker keys built by formatting another name, not an id (Core)", next_check_name_keys, True),
    "next_string_keyed_maps": ("new-pipeline tables keyed by a name's text after resolution, not an id (S4, Core)", next_string_keyed_maps, True),
    "link_reads_check": ("checker types and diagnostics the new pipeline's linker's inputs reach by the type graph (Core)", link_reads_check, True),
    "string_keyed_maps": ("name-keyed HashMaps in cpl-check, not NameMap", string_keyed_maps, True),
    "stress_hand_json": ("hand-built JSON in cpl-stress and cpl-wasm", stress_hand_json, True),
    "helper_env_vars": ("CPL_* helper side channels beyond CPL_SEED", helper_env_vars, True),
    "cpp_deep_copies": ("`// copies:` header sum over corpus programs (read_only_copies)", cpp_deep_copies, True),
    "spec_required_copies": ("`// copies:` header sum over spec/conformance cases (read_only_copies)",
                             spec_required_copies, True),
    "cpp_optional_copies": ("`// optional_copies:` header sum over the corpus (read_only_copies)",
                            cpp_optional_copies, True),
    "cpp_std_functions": ("`// std_functions:` header sum over the corpus (read_only_copies)", cpp_std_functions, True),
    "cpp_sunk_copies": ("`// sunk_copies:` header sum over the corpus (read_only_copies)", cpp_sunk_copies, True),
    "explain_copy_mismatches": ("ok conformance cases whose `cpl explain` copy list differs from the C++ (spec-gate explain-copies --exact)",
                                explain_copy_mismatches, True),
    "codegen_effect_guesses": ("codegen effect or purity guesses outside the Uses table", codegen_effect_guesses, True),
    "opt_without_uses": ("codegen passes deciding what code does without the Uses table (PASSES)", opt_without_uses, True),
    "rs_near_cap": ("rs files within 100 lines of the cap", rs_near_cap, False),
    "stale_arch_paths": ("missing paths cited by ARCHITECTURE docs", stale_arch_paths, True),
    "crate_edges": ("shipped crate edges", crate_edges, False),
    "dev_crate_edges": ("test-only crate edges", dev_crate_edges, False),
    "spec_std_refs": ("spec/std references", spec_std_refs, False),
    # The total is shown; `register_rows_by_domain` gates each domain's count.
    "register_rows": ("quality-register rows naming no existing check", register_rows, False),
    "escapes_open": ("escaped bugs whose suite isn't fixed yet", escapes_open, False),
    "uncited_rules": ("lints and gates citing no handbook rule", uncited_rules, True),
    "unwrap_input": ("unwrap/expect/panic!/unreachable! in shipped crate code", unwrap_input, True),
    "invariant_sites": ("calls to a crate's invariant::broken_invariant", invariant_sites, True),
    "unsupported_sites": ("lowering unsupported( calls and E0501 literals", unsupported_sites, True),
    "lsp_string_types": ("LSP lines reading types out of rendered text", lsp_string_types, True),
    "lint_post_filters": ("entries of cpl-lint after-the-fact tables", lint_post_filters, True),
    "known_lists": ("entries of known-finding lists", known_lists, True),
    "prelude_gaps": ("entries of spec/conformance/prelude-gaps.txt", prelude_gaps, True),
    "permits": ("entries of spec/conformance/permits.txt", permits, True),
    "trait_gaps": ("entries of spec/conformance/trait-gaps.txt", trait_gaps, True),
    "rule_gaps": ("entries of spec/conformance/rule-gaps.txt", rule_gaps, True),
    "sig_gaps": ("entries of spec/conformance/sig-gaps.txt", sig_gaps, True),
    "generate_gaps": ("entries of crates/cpl-stress/tests/generate-gaps.txt", generate_gaps, True),
    "idiom_lint_gaps": ("entries of spec/idioms/lints-known.txt", idiom_lint_gaps, True),
    "std_lint_findings": ("entries of scripts/std-lint-baseline.txt", std_lint_findings, True),
    "corpus_lint_warnings": ("entries of crates/cpl-lint/corpus_warnings.txt", corpus_lint_warnings, True),
    "or_true": ("`|| true` anywhere (scoreboard CI2)", or_true, True),
    "timeout_passes": ("a timeout that ends in success: shell `124)` or a verdict-less TimeoutExpired (Factory)",
                       timeout_passes, True),
    "continue_on_error": ("continue-on-error jobs and steps (scoreboard CI1)", continue_on_error, True),
    "uncached_installs": ("CI tool installs outside the cached install-tools action (Factory)", uncached_installs, True),
    "trace_engine_refs": ("CPL_TRACE_ENGINE mentions (scoreboard F3, Core)", trace_engine_refs, True),
    "trace_budget_gap": ("tenths over the 1.2x trace budget trace_log's ceilings allow (VM-trace)",
                         trace_budget_gap, True),
    "std_types_shown_empty": ("std types a trace shows as `{}`: no pub field, no Show (Content)",
                              std_types_shown_empty, True),
    "std_trait_matrix": ("std containers missing Show, a walk or is_empty (std-api/R12, Content)",
                         std_trait_matrix, True),
    "std_model_gaps": ("std types and pub std fns with no seeded model case against a brute force (Content)",
                       std_model_gaps, True),
    "std_default_model_gaps": ("pub std traits' default methods with no seeded model case against a brute force (Content)",
                               std_default_model_gaps, True),
    "edge_labels_unrun": ("std empty:/ties: labels that are not a claim cpl-conform docs runs (Content)",
                          edge_labels_unrun, True),
    "corpus_known": ("entries of corpus/*-known.txt (scoreboard K6, Content)", corpus_known, True),
    "allow_attrs": ("#[allow(...)] in crates (Core, Consumers)", allow_attrs, True),
    "std_lint_allows": ("with lint(...: allow) in std (Content)", std_lint_allows, True),
    "unsafe_blocks": ("unsafe blocks, fns and impls in crates (Core)", unsafe_blocks, True),
    "cpl_env_reads": ("CPL_* env reads outside the config module (Consumers)", cpl_env_reads, True),
    "test_sleeps": ("sleeps in tests (Factory)", test_sleeps, True),
    "ci_retries": ("--retry/--retries in CI and scripts (Factory)", ci_retries, True),
    "retry_calls": ("retry calls in CI and scripts wrapping no named network fetch or push race (Factory)", retry_calls, True),
    "silent_unknowns": ("expressions typed Unknown with no diagnostic there, in std, corpus and conformance (Core)", silent_unknowns, True),
    "poll_sleeps": ("thread::sleep in shipped Rust (Factory)", poll_sleeps, True),
    "hardcoded_codes": ("E0xxx literals outside cpl-check and cpl-diag (Consumers)", hardcoded_codes, True),
    "xfail_markers": ("xfail/known-fail markers and lists (Content)", xfail_markers, True),
    "todo_fixme": ("TODO/FIXME in std, web and scripts", todo_fixme, True),
    "web_skip_only": ("web .skip(/.only( (Factory)", web_skip_only, True),
    "ts_ignore": ("web @ts-ignore (Factory)", ts_ignore, True),
    "habit_matchers": ("habit help/fix text outside cpl_diag::habits rows, and rows without a mistake (Consumers)", habit_matchers, True),
    "rust_idiom_lints": ("idiom lints in Rust a std @lint from/to could state (Consumers)", rust_idiom_lints, True),
    "unchecked_fix_paths": ("fix appliers outside the re-checking gate (Consumers)", unchecked_fix_paths, True),
    "stress_engines": ("stress-case runners picking a verdict (Consumers)", stress_engines, True),
    "dream_unverified": ("dream.cp files with no passing stress record", dream_unverified, True),
    "measure_broken": ("plan measure.py scripts with no test", measure_broken, True),
    "mono_unguarded": ("monomorphization with no termination guard", mono_unguarded, True),
    "std_shadowed": ("std names lowering bypasses off the builtin floor", std_shadowed, True),
    "corpus_walkers": ("files listing corpus/problems besides scripts/corpus.py", corpus_walkers, True),
    "scaling_mismatches": ("bench/scaling cases slower or faster than the std fn's cost:", scaling_mismatches, True),
    "std_fns_unscaled": ("std fns with a cost: and no bench/scaling case", std_fns_unscaled, True),
    "json_outside_cpl_json": ("serde_json or hand-built JSON outside cpl-json", json_outside_cpl_json, True),
    "config_schemas": ("cp.toml/toml:: readers outside the config module", config_schemas, True),
    "ports_over_bar": ("corpus ports over 1.1x orig.cpp on time or peak RSS at --max (perf)", ports_over_bar, True),
    "perf_twins_stale": ("perf.cp twins whose port.cp is within 1.1x of orig.cpp (perf)", perf_twins_stale, True),
    "std_doc_parsers": ("`///` doc-comment readers outside cpl-syntax/cpl-doc", std_doc_parsers, True),
    "edit_engines": ("text edits applied outside cpl-diag/cpl-fmt", edit_engines, True),
    "codemods_outside_fix": ("scripts rewriting .cp files instead of cpl fix", codemods_outside_fix, True),
    "module_loaders": ("Database::new outside cpl-check", module_loaders, True),
    "ci_jobs_without_recipe": ("checking CI jobs not run through a just recipe (Infra)", ci_jobs_without_recipe, True),
    "web_hand_copied": ("web/ string arrays copying std names, keywords or codes (Factory)", web_hand_copied, True),
    "diag_text_outside_registry": ("diagnostics with inline text outside cpl-diag's registry (Consumers, Core)",
                                   diag_text_outside_registry, True),
    "trap_text_outside_registry": ("runtime trap messages with inline text outside cpl-diag's registry (Consumers)",
                                   trap_text_outside_registry, True),
    "foreign_outside_modules": ("fns recovering another language's habit outside a foreign/ module (Consumers)",
                                foreign_outside_modules, True),
    "event_models": ("event sinks beside the Observer slot, hooks beside Observer::on (Core, Consumers)", event_models, True),
    "width_tables": ("integer-width match arms outside cpl_ir::IntWidth (Core)", width_tables, True),
    "judge_tables": ("judges named outside cpl-judge's table (Core, Consumers)", judge_tables, True),
    "editor_logic_lines": ("editor code lines beyond LSP-client wiring (Consumers)", editor_logic_lines, True),
    "unregistered_lists": ("list files with no LISTS row (counter, owner, plan)", unregistered_lists, True),
}



def register_rows_by_domain(root: Path) -> dict[str, int]:
    """Each domain's count of register rows naming no existing check; each only goes down."""
    counts = {d.stem: 0 for d in sorted((root / "handbook/domains").glob("*.md")) if d.stem != "README"}
    for row in register_rows(root):
        counts[Path(row.split(":", 1)[0]).stem] += 1
    return counts


def rs_over_cap(root: Path, max_rs_lines: int) -> dict[str, int]:
    over = {}
    for p in _rs_files(root):
        n = _code_lines(p)
        if n > max_rs_lines:
            over[_rel(root, p)] = n
    return dict(sorted(over.items(), key=lambda kv: (-kv[1], kv[0])))


def _count(name: str, root: Path) -> int:
    return len(COUNTERS[name][1](root))


CI_ONLY = ("explain_copy_mismatches",)

SLOW_COUNTERS = ("explain_copy_mismatches", "std_model_gaps", "std_default_model_gaps", "near_dup_fns", "corpus_no_statement", "unwrap_input", "written_twice",
                 "corpus_walkers")


def _counts(names: list[str], root: Path) -> dict[str, int]:
    """Each counter's count, measured in forked processes: serially they take
    longer than the commit tier's whole budget."""
    jobs = min(len(names), os.cpu_count() or 1)
    if jobs < 2 or not hasattr(os, "fork"):
        return {n: _count(n, root) for n in names}
    import multiprocessing
    import traceback
    ctx = multiprocessing.get_context("fork")
    results, todo = ctx.SimpleQueue(), ctx.SimpleQueue()
    # Workers take the next counter as they finish, the slowest first: fixed
    # chunks left two slow counters in one worker.
    for n in sorted(names, key=lambda n: n not in SLOW_COUNTERS):
        todo.put(n)
    for _ in range(jobs):
        todo.put(None)

    def work() -> None:
        while (n := todo.get()) is not None:
            try:
                results.put((n, _count(n, root), None))
            except BaseException:
                results.put((n, None, traceback.format_exc()))

    procs = [ctx.Process(target=work) for _ in range(jobs)]
    for p in procs:
        p.start()
    counts, failures = {}, []
    for _ in names:
        n, count, failure = results.get()
        counts[n] = count
        if failure:
            failures.append(f"{n}:\n{failure}")
    for p in procs:
        p.join()
    if failures:
        raise RuntimeError("debt-ratchet: a counter failed\n" + "\n".join(failures))
    return {n: counts[n] for n in names}


def measure(root: Path = ROOT, max_rs_lines: int = DEFAULT_MAX_RS_LINES, only: set[str] | None = None) -> dict:
    """Current file sizes over the cap, every ratcheted count (those in `only`, if given), and the crate edges."""
    data: dict = {"max_rs_lines": max_rs_lines, "rs_file_size_allowlist": rs_over_cap(root, max_rs_lines)}
    data.update(_counts([name for name, (_, _, gated) in COUNTERS.items()
                         if gated and (only is None or name in only)], root))
    data["register_rows_by_domain"] = register_rows_by_domain(root)
    data["crate_edges"] = crate_edges(root)
    data["a4_edges"] = a4_edges(root)
    data["rs_near_cap"] = sorted(_near_cap(root, max_rs_lines))
    for key, fn in ID_SETS.items():
        data[key] = sorted(set(fn(root)))
    return data


def improved(actual: dict, baseline: dict) -> list[str]:
    """Names to pass to `--update NAME...` to tighten exactly what shrank, and nothing a stale tree measured."""
    names = []
    if actual.get("rs_file_size_allowlist") != baseline.get("rs_file_size_allowlist", actual.get("rs_file_size_allowlist")):
        names.append("rs_over_cap")
    names += [n for n, (_, _, g) in COUNTERS.items() if g and n in baseline and n in actual and int(actual[n]) < int(baseline[n])]
    if any(n < baseline.get("register_rows_by_domain", {}).get(d, n)
           for d, n in actual.get("register_rows_by_domain", {}).items()):
        names.append("register_rows_by_domain")
    known, now = set(baseline.get("crate_edges", [])), set(actual.get("crate_edges", []))
    added = now - known
    if known - now or (added and added <= set(actual.get("a4_edges", []))):
        names.append("crate_edges")
    if set(baseline.get("rs_near_cap", [])) - set(actual.get("rs_near_cap", [])):
        names.append("rs_near_cap")
    names += [k for k in ID_SETS if set(baseline.get(k, [])) - set(actual.get(k, []))]
    return names


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
        if name in HARD_FLOOR and int(actual.get(name, 0)) > HARD_FLOOR[name]:
            errors.append(f"{label}: {actual[name]} found; this debt is paid down to {HARD_FLOOR[name]} "
                          f"and must stay there (`--list {name}` shows them)")
            continue
        if name in HARD_ZERO and int(actual.get(name, 0)) > 0:
            errors.append(f"{label}: {actual[name]} found; this debt is paid off and must stay at 0 "
                          f"(`--list {name}` shows them)")
            continue
        if not gated or name not in actual:
            continue
        if name not in baseline:
            if baseline:
                errors.append(f"{label}: gated counter with no baseline (`--update {name}` in the commit that adds it)")
            continue
        cur, base = int(actual[name]), int(baseline[name])
        if name in HARD_ZERO and cur > 0:
            errors.append(f"{label}: {cur}, must be 0 (a hard gate, no baseline; `--list {name}` shows them)")
        elif cur > base:
            errors.append(
                f"{label}: increased from {base} to {cur} (ratchet may only go down; "
                f"`--list {name}` shows them)"
            )
        elif cur < base:
            notes.append(f"{label}: improved from {base} to {cur}")

    # A domain not in the baseline yet (a new charter) is not checked.
    for d, cur in sorted(actual.get("register_rows_by_domain", {}).items()):
        base = baseline.get("register_rows_by_domain", {}).get(d)
        if base is not None and cur > base:
            errors.append(f"handbook/domains/{d}.md: quality-register rows naming no existing check increased "
                          f"from {base} to {cur} (only goes down; `--list register_rows` shows them)")
        elif base is not None and cur < base:
            notes.append(f"handbook/domains/{d}.md: register rows naming no check improved from {base} to {cur}")

    if "crate_edges" in baseline and "crate_edges" in actual:
        known, now = set(baseline["crate_edges"]), set(actual["crate_edges"])
        documented = set(actual.get("a4_edges", []))
        for edge in sorted(now - known):
            if edge in documented:
                notes.append(f"crate edge {edge} added; handbook/design/architecture.md (Crates) documents it")
                continue
            errors.append(
                f"new crate edge {edge}: add it to handbook/design/architecture.md (Crates) and run --update"
            )
        for edge in sorted(known - now):
            notes.append(f"crate edge {edge} removed")

    if "rs_near_cap" in baseline and "rs_near_cap" in actual:
        known, now = set(baseline["rs_near_cap"]), set(actual["rs_near_cap"])
        # A capped file shrinking into the band is progress, not a new near-cap file.
        for rel in sorted(now - known - set(allowlist)):
            errors.append(
                f"{rel}: entered the last {NEAR_CAP_LINES} lines under the {max_lines}-line cap "
                f"(split it on a seam now; a deliberate entry is `--update rs_near_cap` and a decisions.md line)"
            )
        for rel in sorted(known - now):
            notes.append(f"{rel}: left the near-cap band")

    for key in ID_SETS:
        if key in baseline and key in actual:
            known, now = set(baseline[key]), set(actual[key])
            for case in sorted(now - known):
                errors.append(f"{case}: new in {key} (a fix elsewhere keeps the count, not this case; fix it, "
                              f"or `--update {key}` with a Regressed: line naming the case and its commit)")
            for case in sorted(known - now):
                notes.append(f"{case}: left {key}")

    return errors, warnings, notes


# Files besides this script that define what a counter counts. The bound a
# counter reads (read_only_copies.rs) is not one: raising it is the bump.
MEASURED_BY = {
    "corpus_fmt_dirty": ("crates/cpl-fmt/src",),
    "cpp_deep_copies": ("crates/cpl-codegen/src/cpp_counts.rs",),
    "spec_required_copies": ("crates/cpl-codegen/src/cpp_counts.rs",),
    "cpp_optional_copies": ("crates/cpl-codegen/src/cpp_counts.rs",),
    "cpp_std_functions": ("crates/cpl-codegen/src/cpp_counts.rs",),
    "corpus_max_short": ("scripts/corpus.py",),
    "corpus_max_unread": ("scripts/corpus.py",),
}


def _measuring_code(text: str, fn: str) -> str:
    """The source of `fn` in `text` and of every top-level def or constant it reaches."""
    try:
        tree = ast.parse(text)
    except SyntaxError:
        return text
    defs: dict[str, ast.AST] = {}
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            defs[node.name] = node
        elif isinstance(node, (ast.Assign, ast.AnnAssign)):
            for t in node.targets if isinstance(node, ast.Assign) else [node.target]:
                if isinstance(t, ast.Name):
                    defs[t.id] = node
    seen, todo = set(), [fn]
    while todo:
        name = todo.pop()
        if name in seen or name not in defs:
            continue
        seen.add(name)
        todo += [n.id for n in ast.walk(defs[name]) if isinstance(n, ast.Name)]
    return "\n".join(ast.get_source_segment(text, defs[n]) or "" for n in sorted(seen))


def remeasured(root: Path, base: str, name: str) -> bool:
    """Whether the code that measures `name` differs between `base` and the tree."""
    fn = {"rs_file_size_allowlist": "rs_over_cap", "rs_near_cap": "rs_near_cap",
          **{k: f.__name__ for k, f in ID_SETS.items()}}.get(name)
    fn = fn or (COUNTERS[name][1].__name__ if name in COUNTERS else None)
    if fn is None:
        return False
    script = root / "scripts/debt-ratchet.py"
    now = script.read_text(encoding="utf-8") if script.is_file() else ""
    then = "\n".join(_git(root, "show", f"{base}:scripts/debt-ratchet.py"))
    if _measuring_code(now, fn) != _measuring_code(then, fn):
        return True
    paths = MEASURED_BY.get(name, ())
    return bool(paths) and bool(_git(root, "diff", "--name-only", base, "--", *paths))


def upstream_counters(actual: dict, baseline: dict, bumps: list[str]) -> set[str]:
    """The counters main must be measured for: those over their baseline (main's
    debt or this change's) and those raised (a restore of a number lowered by hand
    passes only if main measures it; 0e6b20d67's 1 -> 61 was refused unmeasured)."""
    over = {n for n, (_, _, g) in COUNTERS.items() if g and n in baseline and n in actual
            and n not in HARD_ZERO and n not in HARD_FLOOR and int(actual[n]) > int(baseline[n])}
    paid = {n for n in HARD_ZERO | set(HARD_FLOOR) if int(actual.get(n, 0)) > HARD_FLOOR.get(n, 0)}
    return over | paid | ({b.partition(":")[0] for b in bumps} & set(COUNTERS))


def waived(errors: list[str], mine: list[str], actual: dict, upstream: dict) -> list[str]:
    """The errors main already has that still fail this push: one leak must not spread to every
    lane (17:00 trend audit). A hard zero always fails; any other counter over its baseline on
    main fails unless this change lowers it. A case counter measured by a full run is CI's on
    main and never blocks a push (Ryan's 16:00 ruling)."""
    def num(v) -> int:
        return len(v) if isinstance(v, (list, tuple, set, dict)) else int(v)
    out = []
    for e in errors:
        if e in mine:
            continue
        name = next((n for n in COUNTERS if e.startswith(f"{COUNTERS[n][0]}: ")), None)
        if name is None or name in HARD_ZERO or name in HARD_FLOOR:
            out.append(e)
        elif name not in RUN_MEASURED + CASE_COUNTED and name not in CI_ONLY and name in upstream \
                and num(actual[name]) >= num(upstream[name]):
            out.append(e)
    return out


def mains_paid_debt(actual: dict, upstream: dict) -> set[str]:
    """Hard-zero (and hard-floor) counters over their limit here by no more than main already
    measures: main's red, which its owner fixes and CI on main still fails, not this change's.
    A change that adds one more fails, so a red main never makes every land skip the gate."""
    return {n for n in HARD_ZERO | set(HARD_FLOOR)
            if HARD_FLOOR.get(n, 0) < int(actual.get(n, 0)) <= int(upstream.get(n, -1))}


def unhidden_wrong(root: Path, ref: str) -> list[str]:
    """Cases `ref` refused that this tree lists wrong: a refusal no longer hides their wrong
    verdict, so each admits one to `next_wrong`. A case `ref` passed, or one it lacks, never does."""
    def ids(name: str) -> set[str]:
        return {l.split("\t")[0] for l in _git(root, "show", f"{ref}:{NEXT_TESTS}/{name}") if l.strip()}
    cases = set()
    for l in _git(root, "ls-tree", "-r", "--name-only", ref, "--", "spec/conformance", f"{NEXT_TESTS}/programs"):
        if l.endswith(".cp"):
            cases.add(l.removeprefix("spec/conformance/").replace(f"{NEXT_TESTS}/programs/", "next/"))
    answered = ids("wrong.tsv") | ids("passing.txt")
    return [c for c in next_wrong(root) if c in cases and c not in answered]


# The commit whose std the old pipeline is pinned to (`crates/cpl-check/frozen/std`).
FROZEN_PIN = "6a31a3747^"

# The one authority for what a gate takes on someone's word (audit #18 (2)): a lane for a pushed
# commit (`lane_for`), a counter raise (`raise`), a frozen-crate edit by its patch id (`frozen`)
# and a wasm budget raise (`budget_raise`). Read as origin/main holds it, which only a PR its
# CODEOWNERS approve may change (`signoff_edits`); no commit message signs anything.
SIGNOFFS = "plan/signoffs.toml"
SIGNOFF_PATHS = (SIGNOFFS, ".github/CODEOWNERS")


def signoffs(root: Path) -> dict:
    """`SIGNOFFS` as origin/main holds it, so no branch admits its own rows; HEAD's only in a
    clone with no origin/main."""
    ref = "origin/main" if _git(root, "rev-parse", "-q", "--verify", "origin/main", env=_git_env()) else "HEAD"
    shown = subprocess.run(["git", "-C", str(root), "show", f"{ref}:{SIGNOFFS}"], capture_output=True,
                           text=True, env=_git_env())
    return tomllib.loads(shown.stdout) if shown.returncode == 0 else {}


SIGNOFF_FIELDS = {"lane_for": {"sha": str, "lane": str, "why": str},
                  "raise": {"counter": str, "old": int, "new": int, "why": str},
                  "frozen": {"patch_id": str, "sha": str, "why": str},
                  "budget_raise": {"crate": str, "old": int, "new": int, "why": str}}


def signoffs_malformed(root: Path) -> list[str]:
    """What is wrong with this tree's `SIGNOFFS`: TOML that doesn't parse, a table no gate reads,
    or a row missing a field or holding the wrong type. A PR is checked here, before main holds
    it, because every land reads main's copy and stops at a broken one."""
    path = root / SIGNOFFS
    if not path.is_file():
        return []
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as e:
        return [f"{SIGNOFFS}: {e}"]
    out = [f"{SIGNOFFS}: [[{k}]] is no table a gate reads" for k in data if k not in SIGNOFF_FIELDS]
    for kind, fields in SIGNOFF_FIELDS.items():
        rows = data.get(kind, [])
        for n, row in enumerate(rows if isinstance(rows, list) else [rows], 1):
            bad = [f for f, t in fields.items() if not isinstance(row, dict) or not isinstance(row.get(f), t)
                   or (t is str and not row[f].strip())]
            if bad:
                out.append(f"{SIGNOFFS}: [[{kind}]] row {n} lacks or mistypes {', '.join(bad)}")
    return out


def owner_approved(sha: str) -> bool:
    """Whether GitHub merged `sha` in a PR a code owner of `SIGNOFFS` approved: CI on main asks,
    with its token; anywhere else nothing can tell, so nothing is."""
    repo = os.environ.get("GITHUB_REPOSITORY")
    if not (os.environ.get("CI") and repo):
        return False
    owners = {o.lstrip("@").lower() for l in (ROOT / ".github/CODEOWNERS").read_text().splitlines()
              if l.split() and l.split()[0] == f"/{SIGNOFFS}" for o in l.split()[1:]} \
        if (ROOT / ".github/CODEOWNERS").is_file() else set()
    gh = lambda path: json.loads(subprocess.run(["gh", "api", path], capture_output=True, text=True).stdout or "[]")
    try:
        return any(pr.get("merged_at") and any(r.get("state") == "APPROVED" and
                                               (r.get("user") or {}).get("login", "").lower() in owners
                                               for r in gh(f"repos/{repo}/pulls/{pr['number']}/reviews"))
                   for pr in gh(f"repos/{repo}/commits/{sha}/pulls"))
    except (json.JSONDecodeError, KeyError, TypeError):
        return False


def signoff_edits(root: Path, base: str) -> list[str]:
    """Commits in `base..HEAD` that main doesn't hold and that edit `SIGNOFF_PATHS`, on any
    branch: only a PR a code owner approves changes them (`owner_approved`), so a lane never
    signs its own debt off."""
    # CI on main has origin/main at the pushed tip, so there it would hide the push itself.
    main = ["--not", "origin/main"] if os.environ.get("GITHUB_REF") != "refs/heads/main" and \
        _git(root, "rev-parse", "-q", "--verify", "origin/main", env=_git_env()) else []
    return [f"{sha[:9]}: edits {SIGNOFFS} or its CODEOWNERS; only a PR its code owner approves may, "
            f"so send the row to the coordinator"
            for sha in _git(root, "rev-list", f"{base}..HEAD", *main, "--", *SIGNOFF_PATHS, env=_git_env())
            if not owner_approved(sha)]


SKIPPED_RATCHET_RE = re.compile(r"^Skipped:.*\b(ratchet|debt-ratchet|full suite|lists?)\b", re.M | re.I)


def skipped_ratchet(root: Path, base: str) -> list[str]:
    """Commits in `base..HEAD` that touch the new pipeline (`crates/cpl-next*`) and name the
    ratchet, its lists or the full suite in a `Skipped:` trailer: those are what the ratchet
    measures, so it is never skipped for them (the 11 AM blind audit found 23 in 435)."""
    paths = [str(p.relative_to(root)) for p in sorted((root / "crates").glob("cpl-next*"))]
    if not paths:
        return []
    return [f"{sha[:9]}: touches the new pipeline and skips the ratchet in a `Skipped:` trailer; "
            f"run it (CI's lists jobs do)"
            for sha in _git(root, "rev-list", "--no-merges", f"{base}..HEAD", "--", *paths)
            if SKIPPED_RATCHET_RE.search("\n".join(_git(root, "log", "-1", "--format=%B", sha)))]


# A pushed commit with no `Lane:` gets its lane from a `[[lane_for]]` row in `SIGNOFFS`, never a
# rewrite or a trailer (audit #18 (2)).
LANE_FOR_RE = re.compile(r"^Lane-for:\s*(.*)$", re.MULTILINE)


def lanes_for(root: Path, base: str) -> tuple[dict[str, str], list[str]]:
    """The commits `SIGNOFFS` gives a lane to, as {sha prefix: lane}, and each `Lane-for:`
    trailer in `base..HEAD` refused: a commit message is no authority."""
    given = {str(r["sha"]): str(r["lane"]) for r in signoffs(root).get("lane_for", [])}
    bad = [f"{sha[:9]}: `Lane-for: {m.group(1).strip()}` in a commit message gives no lane; the "
           f"coordinator adds a [[lane_for]] row to {SIGNOFFS}"
           for sha in _git(root, "rev-list", "--no-merges", f"{base}..HEAD")
           for m in LANE_FOR_RE.finditer("\n".join(_git(root, "log", "-1", "--format=%B", sha)))]
    return given, bad


def unlaned(root: Path, base: str) -> list[str]:
    """Commits in `base..HEAD` that touch the new pipeline (`crates/cpl-next*`) or `std/` with no
    `Lane:` trailer. commit-msg adds or demands one, but a hook is skipped where it is not
    installed (4feef68b5, 88ef1dda8 and 1bb42dbcd landed with none), so a regression traced to
    one reached no owner without a search."""
    paths = [str(p.relative_to(root)) for p in sorted((root / "crates").glob("cpl-next*"))]
    paths += ["std"] if (root / "std").is_dir() else []
    if not paths:
        return lanes_for(root, base)[1]
    given, bad = lanes_for(root, base)
    return bad + [f"{sha[:9]}: touches the new pipeline or std with no `Lane:` trailer; add the lane "
                  f"that lands it (`git commit --amend`, or set CPL_LANE; a pushed one takes a "
                  f"[[lane_for]] row in {SIGNOFFS})"
                  for sha in _git(root, "rev-list", "--no-merges", f"{base}..HEAD", "--", *paths)
                  if not any(sha.startswith(g) for g in given)
                  and not any(l.strip() for l in _git(root, "log", "-1", "--format=%(trailers:key=Lane,valueonly)", sha))]


# Counters a programs run measures: a raise of one is the run's evidence, never a re-reading.
RUN_MEASURED = ("next_wrong", "next_not_passing", "next_refused", "next_masked", "next_lowered",
                "next_error_typed", "next_split", "next_wrong_accepts", "next_late_rejects",
                "next_wrong_outputs")
# Cases counted off a committed board rather than a run: also case regressions, so never a push's.
CASE_COUNTED = ("conformance_failing",)
RAISE_RE = re.compile(r"^Raise:\s*(\S+)\s*(.*)$", re.MULTILINE)
FULL_RUN_RE = re.compile(r"\btarget/cpl-next/(?!only/)[\w.-]+")


def raise_evidence(root: Path, base: str, name: str) -> str | None:
    """None when a `Raise: <name>` line in `base..HEAD` names a full programs run's list
    (`target/cpl-next/<list>`) and the commit that run measured; else what is missing. A
    `CPL_NEXT_ONLY` run (`target/cpl-next/only/`) counts only the cases it ran: 152149266 set
    next_error_typed to 10 from next/ alone, and the full run measured 116 (031f4aad9)."""
    why = [m.group(2) for sha in _git(root, "rev-list", "--no-merges", f"{base}..HEAD")
           for m in RAISE_RE.finditer("\n".join(_git(root, "log", "-1", "--format=%B", sha)))
           if m.group(1).rstrip(":") == name]
    def full_run(w: str) -> bool:
        if "CPL_NEXT_ONLY" in w or "target/cpl-next/only" in w or not FULL_RUN_RE.search(w):
            return False
        return any(_git(root, "rev-parse", "-q", "--verify", f"{s}^{{commit}}")
                   for s in re.findall(r"\b[0-9a-f]{7,40}\b", w))
    if any(full_run(w) for w in why):
        return None
    return (f"{name} is measured by a programs run: raise it only with `Raise: {name} <old> -> <new>: "
            f"<why>, full run target/cpl-next/<list> at <sha>`, never from a CPL_NEXT_ONLY run")


def raises(root: Path, base: str) -> list[tuple[str, str, str]]:
    """Each `Raise: <name> <rest>` in `base..HEAD`, as (commit, name, rest), and the commit
    being made's from CPL_RAISE (`;`-separated): pre-commit runs before its message exists, and
    commit-msg writes them as trailers."""
    out = [(sha[:9], m.group(1).rstrip(":"), m.group(2).strip())
           for sha in _git(root, "rev-list", "--no-merges", f"{base}..HEAD")
           for m in RAISE_RE.finditer("\n".join(_git(root, "log", "-1", "--format=%B", sha)))]
    given = [r.strip().split(None, 1) for r in os.environ.get("CPL_RAISE", "").split(";") if r.strip()]
    return out + [("this commit", r[0].rstrip(":"), r[1] if len(r) > 1 else "") for r in given]


def signed_raise(root: Path, base: str, name: str, old: int, new: int) -> str | None:
    """None when a `[[raise]]` row of `SIGNOFFS` raises `name` from `old` to `new` with its
    reason (`why`). Only the coordinator's rows count, so no lane admits its own debt: a
    `Raise:` that signs itself in a commit message is refused, and a hard-zero counter is
    never raised. Else what is missing."""
    if name in HARD_ZERO or name in HARD_FLOOR:
        return f"{name} is hard-zero; it is never raised (remove the debt)"
    selfsigned = [w for w, n, rest in raises(root, base) if n == name and "signed off" in rest]
    if selfsigned:
        return (f"`Raise: {name}` signs itself in {selfsigned[0]}'s message; a raise is valid only as "
                f"a [[raise]] row in {SIGNOFFS}")
    rows = [r for r in signoffs(root).get("raise", []) if r.get("counter") == name]
    if not rows:
        return (f"remove the new debt instead, or ask the coordinator for a [[raise]] row in {SIGNOFFS}: "
                f'counter = "{name}", old = {old}, new = {new}, why = "<reason>"')
    row = next((r for r in rows if (int(r.get("old", -1)), int(r.get("new", -1))) == (old, new)), None)
    if row is None:
        said = ", ".join(f"{r.get('old')} -> {r.get('new')}" for r in rows)
        return f"{SIGNOFFS} raises {name} only {said}, not {old} -> {new}"
    if not str(row.get("why", "")).strip():
        return f"{SIGNOFFS}'s raise of {name} gives no reason (`why`)"
    return None


def frozen_edits(root: Path, base: str) -> list[str]:
    """Commits in `base..HEAD` that add lines to the frozen old pipeline (migration rule 9: no
    fixes, no features). `old_pipeline_lines` only sees the net, so a fix that deletes more than
    it adds passes it; this sees each commit. No trailer excuses one: the only commits that may
    add lines put each old-crate file they touch back as it was at `FROZEN_PIN`, or vendor
    `FROZEN_STD`, the pinned std the old pipeline reads. A fix goes to a conformance case."""
    paths = [f"crates/{c}" for c in FROZEN_CRATES]
    def restores(sha: str) -> bool:
        touched = _git(root, "diff-tree", "--no-commit-id", "--name-only", "-r", sha, "--", *paths)
        def blob(ref: str, f: str) -> str:
            return "\n".join(_git(root, "rev-parse", "-q", "--verify", f"{ref}:{f}"))
        # The commit that vendors FROZEN_STD is the pin itself: it puts the crates back at
        # FROZEN_PIN, each fitted to the crates outside the freeze, and only one commit can.
        pins = bool(blob(sha, FROZEN_STD)) and not blob(f"{sha}^", FROZEN_STD)
        # A re-pin touches only FROZEN_STD, leaving it the tree its own FROZEN_STD_TREE names.
        tree = re.search(r'^FROZEN_STD_TREE = "([0-9a-f]+)"$', "\n".join(
            _git(root, "show", f"{sha}:scripts/debt-ratchet.py")), re.M)
        repins = all(f.startswith(f"{FROZEN_STD}/") for f in touched) and bool(tree) \
            and blob(sha, FROZEN_STD) == tree.group(1) != blob(f"{sha}^", FROZEN_STD)
        return bool(touched) and (pins or repins or all(blob(sha, f) == blob(FROZEN_PIN, f) for f in touched))
    def admitted(sha: str) -> bool:
        diff = subprocess.run(["git", "-C", str(root), "show", sha, "--", *paths], capture_output=True).stdout
        pid = subprocess.run(["git", "patch-id", "--stable"], input=diff, capture_output=True).stdout.split()
        return bool(pid) and pid[0].decode() in {str(r["patch_id"]) for r in signoffs(root).get("frozen", [])}
    out = []
    for sha in _git(root, "rev-list", "--no-merges", f"{base}..HEAD", "--", *paths):
        # A shallow clone's oldest commit has no parent here, so `show` would count the whole
        # tree as added (CI's ratchet at 55999d45b: "adds 204623 lines"). Skipping it unseen let
        # 14960b972 and 0288f1c15 through, so it fails: the clone must reach `base`.
        if not _git(root, "rev-parse", "-q", "--verify", f"{sha}^"):
            out.append(f"{sha[:9]}: its parent is not in this clone, so its edits to the frozen old "
                       f"pipeline can't be seen; fetch back to {base[:9]} (CI: the ratchet's fetch-depth)")
            continue
        rows = [l.split("\t") for l in _git(root, "show", "--numstat", "--format=", sha, "--", *paths)]
        added = sum(int(r[0]) for r in rows if r[0].isdigit())
        if added and not restores(sha) and not admitted(sha):
            out.append(f"{sha[:9]}: adds {added} lines to the frozen old pipeline; send the fix to a "
                       f"conformance case the new pipeline must pass")
    return out


def measured_without_restore(root: Path, base: str, name: str) -> int | None:
    """`name` measured on HEAD with the old crates as `base` had them, or None when the change
    leaves them alone. The one change that may add to them puts them back at `FROZEN_PIN`
    (`frozen_edits`), and brings the pin's debt back with it: a raise no larger than what this
    measures away is that restore, which `old_pipeline_lines` counts and stage 5 deletes, not
    new debt. A regression already on main stays above the raised baseline, still reported."""
    paths = [f"crates/{c}" for c in OLD_PIPELINE_CRATES]
    if name not in COUNTERS or not _git(root, "diff", "--name-only", base, "HEAD", "--", *paths):
        return None
    with tempfile.TemporaryDirectory() as d:
        tree = Path(d) / "t"
        run = lambda *a, at=tree: subprocess.run(["git", "-C", str(at), *a], check=True, capture_output=True,
                                                env=_git_env())
        run("worktree", "add", "-q", "--detach", str(tree), "HEAD", at=root)
        try:
            run("rm", "-rqf", "--ignore-unmatch", "--", *paths)
            kept = [p for p in paths if _git(root, "ls-tree", "-d", "--name-only", base, p)]
            if kept:
                run("checkout", "-q", base, "--", *kept)
            return _count(name, tree)
        finally:
            run("worktree", "remove", "--force", str(tree), at=root)


def unexplained_bumps(root: Path, baseline: dict, ref: str = "origin/main",
                      upstream: dict | None = None) -> list[str]:
    """Baseline entries raised above `ref`'s.

    A raise passes only when the same change rewrites the code that measures
    that entry (a new definition of the count, re-measured), when putting the old crates back
    at `FROZEN_PIN` adds all of it (`measured_without_restore`), or when `ref`
    itself measures at least the raised number (`upstream`): that restores a
    number lowered by hand, it admits no debt, or when the coordinator signs it
    (`signed_raise`). `next_not_passing` may also rise by
    the number of conformance cases the change adds, and `next_wrong` only by the cases a
    `Regressed:` trailer names, a refusal lifted (`unhidden_wrong`) too. Never on a decisions.md
    line or a Skipped: trailer. New debt is removed, not admitted.
    """
    shown = _git(root, "show", f"{ref}:scripts/debt-ratchet.json")
    if not shown:
        return []
    old = json.loads("\n".join(shown))
    base = (_git(root, "merge-base", ref, "HEAD") or [ref])[0]
    moved = [f"{c}: wrong on {ref}, refused here: fix its verdict, or name it in decisions.md"
             for c in wrong_to_refused(root, ref)]
    moved += frozen_edits(root, base)
    # A PR branch carries a signoffs edit to its code owner; main's and the lane's tips, CI or a
    # push (pre-push `--authority`), refuse one that no approved PR merged.
    if os.environ.get("GITHUB_REF") in ("refs/heads/main", "refs/heads/lane-j-check"):
        moved += signoff_edits(root, base)
    moved += signoffs_malformed(root)
    moved += skipped_ratchet(root, base)
    moved += unlaned(root, base)
    moved += set_swaps(root, ref)
    # Main lowering a number while this change gates isn't this change raising it:
    # only a number above the change's own base is (the rebase then takes main's).
    mine = json.loads("\n".join(_git(root, "show", f"{base}:scripts/debt-ratchet.json") or ["{}"]))
    raised = [(n, n) for n, v in baseline.items()
              if n in old and isinstance(v, int) and isinstance(old[n], int) and v > old[n]
              and not (isinstance(mine.get(n), int) and v <= mine[n])
              and not (upstream and isinstance(upstream.get(n), int) and v <= upstream[n])
              and not ((w := measured_without_restore(root, base, n)) is not None
                       and v - old[n] <= _count(n, root) - w)]
    old_allow, mine_allow = old.get("rs_file_size_allowlist", {}), mine.get("rs_file_size_allowlist", {})
    raised += [(rel, "rs_file_size_allowlist") for rel, cap in baseline.get("rs_file_size_allowlist", {}).items()
               if cap > old_allow.get(rel, 0) and cap > mine_allow.get(rel, 0)]
    if "rs_near_cap" in old and (set(baseline.get("rs_near_cap", [])) - set(old["rs_near_cap"]) - set(old_allow)
                                 - set(mine.get("rs_near_cap", [])) - set(mine_allow)):
        raised.append(("rs_near_cap", "rs_near_cap"))
    # Only a regeneration's case named `Regressed:` may enter a bad set: one `ref` refused
    # and the tree now answers wrong (an exposure, `unhidden_wrong`) is named there too.
    admitted = regressed_named(root, ref)
    raised += [(k, k) for k in ID_SETS
               if k in old and set(baseline.get(k, [])) - set(old[k]) - set(mine.get(k, [])) - admitted]
    # A new case the new pipeline cannot run yet is a case added, not a
    # case broken (`passing.txt` and `next_wrong` hold those), so each admits one.
    added = len([l for l in _git(root, "diff", "--name-only", "--diff-filter=A", ref, "--", "spec/conformance",
                                    NEXT_TESTS + "/programs")
                 if l.endswith(".cp")])
    raised = [(n, key) for n, key in raised
              if not (n == "next_not_passing" and baseline[n] - old[n] <= added)
              and not (n == "next_wrong" and baseline[n] - old[n] <= len(admitted & set(next_wrong(root))))
              and not (n == "next_wrong_accepts"
                       and baseline[n] - old[n] <= len(admitted & set(next_wrong_accepts(root))))
              and not (n == "next_split" and baseline[n] - old[n] <= len(admitted & set(next_split(root))))]
    zero = [n for n, _ in raised if n in COUNTERS and old.get(n, 1) == 0]
    if zero:
        return moved + [f"{n}: reached 0 on {ref}; a counter at 0 is never raised" for n in zero]
    if not raised:
        return moved
    stale = "remove the new debt instead (only a change to the code that measures it may re-measure it)"
    return moved + [f"{n}: baseline raised above {ref} ({old.get(n, old_allow.get(n))} -> "
            f"{baseline.get(n, baseline.get('rs_file_size_allowlist', {}).get(n))}); {need}"
            for n, key in raised
            for need in [raise_evidence(root, base, n) if n in RUN_MEASURED
                         else None if remeasured(root, base, key)
                         else signed_raise(root, base, n, old[n], baseline[n]) if n in COUNTERS
                         else stale] if need]


def hand_lowered(root: Path, baseline: dict, actual: dict, ref: str = "origin/main") -> list[str]:
    """Counters this change lowers below `ref`'s to a number under what the tree
    measures: a number lowered by hand, not by removing debt (f1b1dd166 set
    vm_fallbacks 62 -> 1 with 62 still in the source, and every lane writing
    its real count was then refused as a raise). `--update NAME` lowers it."""
    old = json.loads("\n".join(_git(root, "show", f"{ref}:scripts/debt-ratchet.json") or ["{}"]))
    return [f"{n}: lowered from {old[n]} to {v} on {ref}, but this tree measures {actual[n]}; "
            f"remove the debt and run `--update {n}` instead of editing the number"
            for n, v in baseline.items()
            if n in COUNTERS and isinstance(v, int) and isinstance(old.get(n), int) and v < old[n]
            and isinstance(actual.get(n), int) and actual[n] > v]


def inherited_baseline(baseline: dict, upstream: dict) -> dict:
    """`baseline` raised to what `upstream` (main) already measures, so a check
    fails only on debt this tree adds on top of main."""
    eff = json.loads(json.dumps(baseline))
    for name, (_, _, gated) in COUNTERS.items():
        if name in HARD_ZERO:
            eff[name] = 0
        elif name in HARD_FLOOR:
            eff[name] = HARD_FLOOR[name]
        elif gated and name in eff and name in upstream:
            eff[name] = max(int(eff[name]), int(upstream[name]))
    allow = eff.setdefault("rs_file_size_allowlist", {})
    for rel, lines in upstream.get("rs_file_size_allowlist", {}).items():
        allow[rel] = max(allow.get(rel, 0), lines)
    for key in ("crate_edges", "rs_near_cap", *ID_SETS):
        if key in eff and key in upstream:
            eff[key] = sorted(set(eff[key]) | set(upstream[key]))
    return eff


class UpstreamUnknown(Exception):
    """`ref` exists but could not be checked out to measure."""


def measure_upstream(root: Path, max_rs_lines: int, ref: str = "origin/main",
                     only: set[str] | None = None) -> dict | None:
    """Measures `ref` (its counters in `only`, if given) in a scratch worktree;
    None when `ref` is missing, or HEAD is `ref` in CI (a run on main), so main's
    own debt still fails there. A local commit on a fresh tip also has HEAD at
    `ref`, and its base is `ref`."""
    head, up = (_git(root, "rev-parse", "-q", "--verify", r, env=_git_env()) for r in ("HEAD", ref))
    if not head or not up or (head == up and os.environ.get("CI")):
        return None
    # Every land on a tip whose own debt is over its baseline measures that
    # tip: keyed by its tree (and the measuring code) in the common git dir,
    # every worktree's land after the first reads it, and measures only the
    # counters it needs that no land has yet.
    common = _git(root, "rev-parse", "--path-format=absolute", "--git-common-dir", env=_git_env())
    tree = _git(root, "rev-parse", f"{up[0]}^{{tree}}", env=_git_env())
    code = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()[:12]
    cache = (Path(common[0]) / "cpl-ratchet-upstream" / f"{tree[0]}-{max_rs_lines}-{code}.json"
             if common and tree else None)
    data: dict = {}
    if cache and cache.is_file():
        try:
            data = json.loads(cache.read_text(encoding="utf-8"))
        except ValueError:
            pass
    have = set(data.get("measured", []))
    gated = {n for n, (_, _, g) in COUNTERS.items() if g}
    need = (gated if only is None else set(only) & gated) - have
    if data and not need:
        return data["counts"]
    fresh = _measure_at(root, up[0], max_rs_lines, need)
    data = {"measured": sorted(have | need), "counts": {**data.get("counts", {}), **fresh}}
    if cache:
        cache.parent.mkdir(exist_ok=True)
        tmp = cache.with_suffix(f".{os.getpid()}.tmp")
        tmp.write_text(json.dumps(data), encoding="utf-8")
        tmp.replace(cache)
        for old in sorted(cache.parent.glob("*.json"), key=lambda f: f.stat().st_mtime, reverse=True)[16:]:
            old.unlink(missing_ok=True)
    return data["counts"]


def _measure_at(root: Path, rev: str, max_rs_lines: int, only: set[str] | None = None) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        wt = Path(tmp) / "main"
        add = subprocess.run(["git", "-C", str(root), "worktree", "add", "-q", "--detach", str(wt), rev],
                             capture_output=True, text=True, env=_git_env())
        if add.returncode != 0:
            raise UpstreamUnknown(add.stderr.strip() or f"git worktree add exited {add.returncode}")
        saved = os.environ.get("CPL")
        try:
            # The worktree has no target/: it is measured with this checkout's cpl.
            os.environ["CPL"] = str(_built_cpl(root).resolve())
            # A counter main's baseline doesn't have yet is new here: main has nothing
            # to inherit for it, and may not have what it reads.
            known = wt / "scripts/debt-ratchet.json"
            names = set(json.loads(known.read_text(encoding="utf-8"))) if known.is_file() else None
            if only is not None:
                names = only if names is None else names & only
            return measure(wt, max_rs_lines, names)
        finally:
            if saved is None:
                os.environ.pop("CPL", None)
            else:
                os.environ["CPL"] = saved
            subprocess.run(["git", "-C", str(root), "worktree", "remove", "--force", str(wt)],
                           capture_output=True, env=_git_env())


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Check or update the tech-debt ratchet.")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="Verify against the ratchet file.")
    group.add_argument("--update", nargs="*", metavar="NAME",
                       help="Write current measurements to the ratchet file; with NAMEs, only those entries.")
    group.add_argument("--tighten", action="store_true",
                       help="--check, then lower each improved entry to its measured value (land.sh runs it).")
    group.add_argument("--show", nargs="*", metavar="NAME", help="Print `name count` for counters.")
    group.add_argument("--list", metavar="NAME", help="Print the offenders behind one counter.")
    group.add_argument("--authority", metavar="BASE",
                       help="Fail on a commit since BASE that edits the frozen crates or the signoffs "
                            "unadmitted: pre-push runs it on a push to main or lane-j-check.")
    group.add_argument("--frozen", metavar="BASE",
                       help="Fail on a commit since BASE that edits the frozen crates unadmitted: pre-push "
                            "runs it on a push to a PR branch, where the signoffs may change for review.")
    group.add_argument("--zero", action="store_true",
                       help="Fail while any gated counter is above 0: main's own debt, reported red.")
    parser.add_argument("--prefix", default="", metavar="PREFIX",
                        help="With --zero, only the counters named PREFIX...: `--zero --prefix next_` is the "
                             "stage-5 switch's gate (handbook/design/next/migration.md rule 14).")
    parser.add_argument("--ratchet", type=Path, default=DEFAULT_RATCHET, help="Path to debt-ratchet.json.")
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root.")
    args = parser.parse_args(argv)

    max_lines = DEFAULT_MAX_RS_LINES
    if args.ratchet.is_file():
        max_lines = int(json.loads(args.ratchet.read_text(encoding="utf-8")).get("max_rs_lines", max_lines))

    def offenders(name: str) -> list[str]:
        if name == "rs_over_cap":
            return [f"{k} {v}" for k, v in rs_over_cap(args.root, max_lines).items()]
        return COUNTERS[name][1](args.root)

    if args.authority is not None or args.frozen is not None:
        since = args.authority or args.frozen
        base = (_git(args.root, "merge-base", since, "HEAD") or [since])[0]
        bad = frozen_edits(args.root, base) + (signoff_edits(args.root, base) if args.authority else [])
        for e in bad:
            print(f"debt-ratchet: ERROR: {e}", file=sys.stderr)
        return 1 if bad else 0
    if args.zero:
        left, failed = {}, 0
        for n in ["rs_over_cap", *(n for n, (_, _, gated) in COUNTERS.items() if gated)]:
            if not n.startswith(args.prefix):
                continue
            try:
                left[n] = len(offenders(n))
            except MeasureError as e:
                print(f"debt-ratchet: {n} ERROR {e}")
                failed += 1
        left = {n: c for n, c in left.items() if c}
        for n, c in left.items():
            print(f"debt-ratchet: {n} {c} (target 0; `--list {n}` shows them)")
        print(f"debt-ratchet: {len(left)} counter(s) above 0" if left else "debt-ratchet: every counter at 0")
        return 1 if left or failed else 0

    if args.show is not None or args.list:
        names = [args.list] if args.list else (args.show or ["rs_over_cap", *COUNTERS])
        unknown = [n for n in names if n != "rs_over_cap" and n not in COUNTERS]
        if unknown:
            print(f"debt-ratchet: unknown counter {unknown[0]}; one of rs_over_cap, {', '.join(COUNTERS)}",
                  file=sys.stderr)
            return 2
        failed = 0
        for name in names:
            try:
                items = offenders(name)
            except MeasureError as e:
                print(f"{name} ERROR {e}")
                failed += 1
                continue
            print(f"{name} {len(items)}")
            if args.list:
                print("\n".join(f"  {x}" for x in items))
        return 1 if failed else 0

    if args.update is not None:
        data = measure(args.root, max_lines)
        data.pop("a4_edges", None)
        if args.update:
            keys = {"rs_over_cap": "rs_file_size_allowlist"}
            unknown = [n for n in args.update if keys.get(n, n) not in data]
            if unknown or not args.ratchet.is_file():
                print(f"debt-ratchet: cannot update {', '.join(unknown) or 'a missing ratchet file'}", file=sys.stderr)
                return 2
            base = json.loads(args.ratchet.read_text(encoding="utf-8"))
            for n in args.update:
                base[keys.get(n, n)] = data[keys.get(n, n)]
            data = base
        # A stale local baseline must not carry main's tightening back up.
        bumps = unexplained_bumps(args.root, data, os.environ.get("CPL_RATCHET_BASE") or "origin/main")
        if bumps:
            print("debt-ratchet: refusing to write a baseline above origin/main (pull first, or remove the debt):\n  "
                  + "\n  ".join(bumps), file=sys.stderr)
            return 1
        raised = [n for n in sorted(HARD_ZERO) if int(data.get(n, 0)) > 0]
        raised += [n for n, floor in sorted(HARD_FLOOR.items()) if int(data.get(n, 0)) > floor]
        if raised:
            print(f"debt-ratchet: refusing to write hard-zero counters above 0: {', '.join(raised)} "
                  f"(remove the debt; `--list NAME` shows it)", file=sys.stderr)
            return 1
        args.ratchet.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
        counts = ", ".join(f"{k}={data[k]}" for k, (_, _, g) in COUNTERS.items() if g and k in data)
        print(f"debt-ratchet: wrote {args.ratchet} (rs>{max_lines}={len(data['rs_file_size_allowlist'])}, {counts})")
        return 0

    if not args.ratchet.is_file():
        print(f"debt-ratchet: missing {args.ratchet}; run --update", file=sys.stderr)
        return 1

    baseline = json.loads(args.ratchet.read_text(encoding="utf-8"))
    # Off CI, the counters that run cpl over every case are left to CI on each main tip
    # (culprit.yml names the commit): they cost minutes whenever cpl changed, and a case
    # measurement never blocks a land.
    local = None if os.environ.get("CI") or os.environ.get("CPL_RATCHET_ALL") else set(COUNTERS) - set(CI_ONLY)
    actual = measure(args.root, max_lines, local)
    errors, warnings, notes = evaluate(actual, baseline)
    # On main, HEAD is origin/main: CI names the tip it last saw instead.
    ref = os.environ.get("CPL_RATCHET_BASE") or "origin/main"
    bumps = unexplained_bumps(args.root, baseline, ref)
    # CI on main sets CPL_RATCHET_STRICT: there every counter over its baseline is main's own
    # debt and fails, rather than being waved through as "already on main".
    if (errors or bumps) and not os.environ.get("CPL_RATCHET_STRICT"):
        try:
            # Only the counters over their baseline can be main's, not this change's.
            upstream = measure_upstream(args.root, max_lines, only=upstream_counters(actual, baseline, bumps))
        except UpstreamUnknown as e:
            # Unable to tell main's debt from this change's: warn, and leave
            # the verdict to CI on main rather than block a commit.
            warnings += [f"could not measure origin/main ({e}); not blocking: {e2}" for e2 in errors]
            upstream, errors = None, []
        if upstream is not None:
            mine = evaluate(actual, inherited_baseline(baseline, upstream))[0]
            labels = tuple(f"{COUNTERS[n][0]}: " for n in mains_paid_debt(actual, upstream))
            mine = [e for e in mine if not e.startswith(labels)] if labels else mine
            mine += waived(errors, mine, actual, upstream)
            warnings += [f"already on main, not from this change (infra fixes it): {e}"
                         for e in errors if e not in mine]
            errors = mine
            bumps = unexplained_bumps(args.root, baseline, ref, upstream)
    errors += bumps + hand_lowered(args.root, baseline, actual, ref)

    # Name the last commits to touch a grown file, so its owner is pinged
    # rather than whoever pushes next.
    for i, e in enumerate(errors):
        rel = e.partition(": grew from ")[0] if ": grew from " in e else None
        if rel:
            by = last_touched(args.root, rel)
            errors[i] = f"{e}; last touched by: {'; '.join(by)}" if by else \
                f"{e}; run `git log -3 -- {rel}` in a full clone for who grew it"

    for w in warnings:
        print(f"debt-ratchet: WARNING: {w}", file=sys.stderr)
    # Every raise is shown, so the coordinator and the audits check each one against a ruling.
    for at, n, rest in raises(args.root, (_git(args.root, "merge-base", ref, "HEAD") or [ref])[0]):
        print(f"debt-ratchet: RAISE {at}: {n} {rest}", file=sys.stderr)
    for e in errors:
        print(f"debt-ratchet: ERROR: {e}", file=sys.stderr)
    counts = ", ".join(
        f"{k}={actual[k]}/{baseline[k]}" for k, (_, _, g) in COUNTERS.items() if g and k in baseline and k in actual
    )
    print(
        f"debt-ratchet: rs>{max_lines}={len(actual['rs_file_size_allowlist'])} "
        f"(allowlist {len(baseline.get('rs_file_size_allowlist', {}))}), {counts}"
    )
    if errors:
        print("debt-ratchet: remove the new debt; a baseline rises only with a change to the code that measures it",
              file=sys.stderr)
    if notes:
        for n in notes:
            print(f"debt-ratchet: {n}")
        names = " ".join(improved(actual, baseline))
        print(f"debt-ratchet: {len(notes)} item(s) improved; tighten with "
              f"`uv run --quiet python scripts/debt-ratchet.py --update {names}`")
    if errors and notes:
        print(f"debt-ratchet: FAILED on the {len(errors)} ERROR line(s) above, not on the improvements", file=sys.stderr)
    if args.tighten and not errors and notes:
        keys = {"rs_over_cap": "rs_file_size_allowlist"}
        for n in improved(actual, baseline):
            baseline[keys.get(n, n)] = actual[keys.get(n, n)]
        args.ratchet.write_text(json.dumps(baseline, indent=2) + "\n", encoding="utf-8")
        print("debt-ratchet: lowered the improved entries to their measured values")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())

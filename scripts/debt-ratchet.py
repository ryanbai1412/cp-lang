#!/usr/bin/env python3
"""Tech-debt ratchet: architecture counts that may only go down.

Every count below is measured from the tree and compared with
`scripts/debt-ratchet.json`. A count above its baseline fails the gate; the
only way past is `--update` in the same commit plus a `decisions.md` line
saying why. Lower counts pass and print a hint to tighten the baseline.
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
import difflib
import fnmatch
import functools
import glob
import hashlib
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
    return {tuple(e.split(" -> ", 1)) for e in crate_rules(root).get("forbidden edge", []) if " -> " in e}


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
# lint baseline (scripts/std-lint-ratchet.py) and search's expected answers (the test fails
# on an answer page the site no longer has).
# The ratchet names crates only in the paths its gates read (stale_arch_paths
# fails on a missing one) and in those gates' descriptions; crate rules are architecture.md's.
WRITTEN_TWICE_CHECKED = {"handbook/design/architecture.md", "scripts/std-lint-baseline.txt",
                         "web/tests/search-tasks.mjs", "scripts/debt-ratchet.py"}


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
    for p in sorted((root / "std").glob("*.cp")):
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
CP_DUP_MIN_CHARS = 25


def _cp_fn_bodies(root: Path):
    """(rel, line, name, body lines) for every `pub fn` in std/*.cp: the signature ends at
    its depth-0 `:` (a block of deeper lines follows) or `=` (the expression after it);
    `//` lines are dropped."""
    for rel in _git(root, "ls-files", "std/**.cp", "std/*.cp"):
        src = _lines(root / rel)
        i = 0
        while i < len(src):
            m = CP_PUB_FN.match(src[i])
            if not m:
                i += 1
                continue
            indent, depth, j, k, body = len(m.group(1)), 0, i, src[i].index("pub fn") + 6, None
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
    crates, and each std `name`/`name_real`/`name_excl` pair (Content)."""
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
    return [x for _, x in sorted(out)] + _cp_near_dups(root)


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
    """Tracked `test_*.py` files the gate neither names nor globs in its `script_tests`."""
    gate = root / "scripts/check.sh"
    text = gate.read_text() if gate.is_file() else ""
    m = re.search(r"^mapfile -t script_tests < <\(git ls-files (.*)\)$", text, re.M)
    globs = re.findall(r"'([^']+)'", m[1]) if m else []
    return sorted(rel for rel in _git(root, "ls-files", "*test_*.py")
                  if Path(rel).name.startswith("test_") and Path(rel).name not in text
                  and not any(fnmatch.fnmatchcase(rel, g) for g in globs))


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


def vm_fallbacks(root: Path) -> list[str]:
    """(program, mode, tag) tree-walker fallbacks; cpl-cli's vm_fallbacks test keeps the list true."""
    path = root / "crates/cpl-cli/tests/vm_fallbacks.txt"
    return [l for l in _lines(path) if l and not l.startswith("#")] if path.is_file() else []


OWN_DATABASE_RE = re.compile(r"\bDatabase::(?:new|with_loader|default)\b|\bDatabase\s*\{")


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


def _copy_baseline(root: Path, name: str) -> list[str]:
    """The `read_only_copies` test's shrink-only count `name` over the corpus C++, one item per unit."""
    path = root / "crates/cpl-cli/tests/read_only_copies.rs"
    text = path.read_text() if path.is_file() else ""
    m = re.search(rf'\("{name}",\s*\w+,\s*(\d+)\)', text)
    return [f"{name} {i}" for i in range(int(m.group(1)))] if m else []


def cpp_deep_copies(root: Path) -> list[str]:
    return _copy_baseline(root, "deep_copies")


def cpp_std_functions(root: Path) -> list[str]:
    return _copy_baseline(root, "std_functions")


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
    gate (oversize), so the bench gates hold only once there are none."""
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
    check = root / "scripts/check.sh"
    gates = sorted(set(re.findall(r"scripts/[\w-]+\.(?:py|sh|mjs)", check.read_text(encoding="utf-8")))) \
        if check.is_file() else []
    for g in gates:
        name = Path(g).name
        if name.startswith("test_") or name in ("check.sh", "gate-env.sh") or not (root / g).is_file():
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


def _src_lines(root: Path, *subdirs: str) -> Iterator[tuple[str, int, str]]:
    """Non-test, non-comment Rust lines under `subdirs`: tests/, tests.rs, *_tests.rs,
    build.rs and everything after a `#[cfg(test)]` are left out."""
    for p in _rs_files(root, *subdirs, tests=False):
        rel = _rel(root, p)
        parts = set(Path(rel).parts[:-1])
        if parts & NON_TEST_RS_PARTS or p.name == "build.rs" or p.name.endswith("_tests.rs") or p.name.startswith("tests_") \
                or "/src/tests/" in rel:
            continue
        for i, line in enumerate(_lines(p), 1):
            if line.strip() == "#[cfg(test)]":
                break
            if not line.lstrip().startswith("//"):
                yield rel, i, line


UNWRAP_RE = re.compile(r"\.unwrap\(\)|\.expect\(|\bpanic!\(|\bunreachable!\(")


def unwrap_input(root: Path) -> list[str]:
    """`.unwrap()`, `.expect(`, `panic!(`, `unreachable!(` in shipped crate code:
    input errors become diagnostics, invariants go behind one counted helper per crate."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root) for _ in UNWRAP_RE.finditer(line)]


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


def corpus_known(root: Path) -> list[str]:
    """Entries of corpus/audit-, differential- and generators-known.txt: ports a corpus check skips."""
    return _registered_entries(root, "corpus_known")


def known_lists(root: Path) -> list[str]:
    """Entries of *-known.txt and other accepted-finding baselines."""
    return _registered_entries(root, "known_lists")


def prelude_gaps(root: Path) -> list[str]:
    """Entries of spec/conformance/prelude-gaps.txt: prelude names with no conformance case."""
    return _registered_entries(root, "prelude_gaps")


def rule_gaps(root: Path) -> list[str]:
    """Entries of spec/conformance/rule-gaps.txt, deleted: every rule has the cases it needs."""
    return _registered_entries(root, "rule_gaps")


def sig_gaps(root: Path) -> list[str]:
    """Entries of spec/conformance/sig-gaps.txt, deleted: every named arg the chapters use is in std."""
    return _registered_entries(root, "sig_gaps")


def trait_gaps(root: Path) -> list[str]:
    """Entries of spec/conformance/trait-gaps.txt, deleted: every trait method has a case."""
    return _registered_entries(root, "trait_gaps")


def idiom_lint_gaps(root: Path) -> list[str]:
    """Entries of spec/idioms/lints-known.txt, deleted: the lint catalog names only real APIs."""
    return _registered_entries(root, "idiom_lint_gaps")


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


def unsafe_blocks(root: Path) -> list[str]:
    """`unsafe {`, `unsafe fn`, `unsafe impl` in crates."""
    return _grep_all(root, r"\bunsafe (\{|fn\b|impl\b)", "crates/**/*.rs")


def cpl_env_reads(root: Path) -> list[str]:
    """`var("CPL_…")`/`var_os("CPL_…")` in crates outside the one config module: a second config channel."""
    return [h for h in _grep_all(root, r"var(_os)?\(\"CPL_", "crates/**/*.rs")
            if not CONFIG_MODULES or _outside(h, CONFIG_MODULES)]


def test_sleeps(root: Path) -> list[str]:
    """Sleeps in tests: a wait for a race instead of an event."""
    return _grep_all(root, r"sleep\(|time\.sleep|setTimeout\(|waitForTimeout", *TEST_PATHS)


def ci_retries(root: Path) -> list[str]:
    """`--retry`/`--retries` in workflows, scripts and the justfile: a failure tried again."""
    return _grep_all(root, r"--retr(y|ies)\b", ".github", "scripts", "justfile")


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


HABIT_FILES = ("crates/cpl-syntax/src/parser/foreign.rs", "crates/cpl-syntax/src/parser/c_decl.rs",
               "crates/cpl-check/src/check/advice.rs")


def habit_matchers(root: Path) -> list[str]:
    """`fn` items in hand-written habit matchers: the named files and any crates file named
    foreign/habit/advice (Consumers; a generated habit table replaces them)."""
    files = {rel for rel in _git(root, "ls-files", "crates/**/*.rs")
             if rel in HABIT_FILES or re.search(r"foreign|habit|advice", Path(rel).name)}
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root)
            if rel in files and re.match(r"\s*(pub(\([^)]*\))?\s+)?fn\s", line)]


FIX_GATE = ("crates/cpl-ir/src/fix.rs", "crates/cpl-lint/src/typed.rs")


def unchecked_fix_paths(root: Path) -> list[str]:
    """Raw fix appliers called outside the one re-checking gate, and the ungated
    `fn apply_` definitions in cpl-diag/src/fix.rs."""
    calls = [f"{rel}:{i}" for rel, i, line in _src_lines(root)
             if rel not in FIX_GATE and rel != "crates/cpl-diag/src/fix.rs"
             and re.search(r"\b(apply_fixes|apply_error_fixes|apply_lint_fixes)\(", line)
             and not re.search(r"\bfn\s", line)]
    defs = [f"{rel}:{i}" for rel, i, line in _src_lines(root, "crates/cpl-diag/src")
            if rel == "crates/cpl-diag/src/fix.rs" and re.search(r"\bfn apply_", line)]
    return calls + defs


def stress_engines(root: Path) -> list[str]:
    """Stress-case runners picking a verdict: `fn run_case|stress_source|judge_case`
    definitions in cpl-stress, plus files besides judge.rs constructing `Verdict::`."""
    lines = list(_src_lines(root, "crates/cpl-stress/src"))
    defs = [f"{rel}:{i}" for rel, i, line in lines if re.search(r"\bfn (run_case|stress_source|judge_case)\b", line)]
    files = sorted({rel for rel, _, line in lines
                    if "Verdict::" in line and Path(rel).name != "judge.rs"})
    return defs + [f for f in files if not any(d.startswith(f + ":") for d in defs)]


def or_true(root: Path) -> list[str]:
    """`|| true` anywhere in the repo: a failure swallowed."""
    return _grep_all(root, r"\|\|[[:space:]]*true\b")


def trace_engine_refs(root: Path) -> list[str]:
    """`CPL_TRACE_ENGINE` mentions: the switch back to the tree-walker for traces."""
    return _grep_all(root, "CPL_TRACE_ENGINE")


CONTINUE_RE = re.compile(r"^(\s*)(?:-\s+)?continue-on-error:\s*(.+?)\s*$")


def continue_on_error(root: Path) -> list[str]:
    """Jobs and steps marked `continue-on-error` (not `false`) in any workflow or action:
    a red that turns nothing red."""
    return [f"{rel}:{i}" for rel, i, line in _workflow_lines(root)
            if (m := CONTINUE_RE.match(line)) and m.group(2) != "false"]


def dream_unverified(root: Path) -> list[str]:
    """corpus/problems/*/dream.cp whose meta.json has no `dream_stress` record with ok=true
    for the current blob (written by `scripts/corpus.py verify`)."""
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


STD_ITEM_RE = re.compile(r"^(\s*)(?:@\w+\s+)*pub\s+(fn|struct|enum|trait|type)\s+(\w+)\s*(<[^>(]*>)?\s*(?:\(([^)]*))?")
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


def _compiler_strings(root: Path) -> set[str]:
    """Every identifier-like string literal in the checker, IR, interpreter and emitter, tests aside."""
    found: set[str] = set()
    for base in COMPILER_SRC:
        for f in sorted((root / base).rglob("*.rs")):
            if "tests" in f.parts or f.name in ("tests.rs", "test.rs"):
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
    literal, whose methods it may answer too."""
    floor = _builtin_floor(root)
    arms = set(ir_builtin_names(root))
    named = _compiler_strings(root)
    out: set[str] = set()
    for p in sorted((root / "std").glob("*.cp")):
        for line in _lines(p):
            m = STD_ITEM_RE.match(line)
            if not m:
                continue
            indent, kind, name, generics, params = m.groups()
            if kind != "fn":
                if not indent and name in named and name not in floor:
                    out.add(name)
                continue
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


def corpus_walkers(root: Path) -> list[str]:
    """Files besides scripts/corpus.py that list corpus/problems themselves: a second corpus walker."""
    found = []
    for rel in _git(root, "ls-files", "*.py", "*.sh", "*.rs", "*.js", "*.mjs", "*.ts", "justfile"):
        if rel == "scripts/corpus.py" or rel.startswith(("research/", "corpus/problems/")) \
                or "node_modules/" in rel or not (root / rel).is_file():
            continue
        lines = _lines(root / rel)
        for i, line in enumerate(lines):
            if "corpus/problems" in line or re.search(r"[\"']problems[\"']", line):
                if any(WALK_RE.search(l) for l in lines[max(0, i - 3):i + 4]):
                    found.append(rel)
                    break
    return found

def _outside(rel: str, owners: tuple[str, ...]) -> bool:
    return not rel.startswith(owners)


JSON_FORMAT_RE = re.compile(r'format!\("(?:\{\{\\"|[^"]*\\":)')


def json_outside_cpl_json(root: Path) -> list[str]:
    """`serde_json` in a Cargo.toml or a hand-built JSON `format!` outside crates/cpl-json:
    a second machine-output schema."""
    found = [rel for rel in _git(root, "ls-files", "Cargo.toml", "crates/*/Cargo.toml")
             if _outside(rel, ("crates/cpl-json/",)) and "serde_json" in (root / rel).read_text(encoding="utf-8")]
    return found + [f"{rel}:{i}" for rel, i, line in _src_lines(root)
                    if _outside(rel, ("crates/cpl-json/",)) and JSON_FORMAT_RE.search(line)]


# The one config module, named by its owner (Consumers' tools child) once it exists.
CONFIG_MODULES: tuple[str, ...] = ()
CONFIG_RE = re.compile(r'"cp\.toml"|\btoml::')


def config_schemas(root: Path) -> list[str]:
    """Shipped files reading `cp.toml` or using `toml::` outside the one config module."""
    return sorted({rel for rel, _, line in _src_lines(root)
                   if CONFIG_RE.search(line) and (not CONFIG_MODULES or _outside(rel, CONFIG_MODULES))})


def std_doc_parsers(root: Path) -> list[str]:
    """`"///` literals outside cpl-syntax and cpl-doc: a second reader of std doc comments."""
    return [f"{rel}:{i}" for rel, i, line in _src_lines(root)
            if '"///' in line and _outside(rel, ("crates/cpl-syntax/", "crates/cpl-doc/"))]


EDIT_ENGINE_RE = re.compile(r"\bfn apply_fixes\b|\bfn apply_edits\b|\.replace_range\(")


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


# Every tracked list file outside test data: the counter that counts its
# entries (None for a data list that exempts nothing), its owner, and how it
# goes away. A new list without a row here fails `unregistered_lists`.
LISTS: dict[str, tuple[str | None, str, str]] = {
    "bench/oversize.txt": ("bench_lists", "perf", "shrink inputs or give the bench a size cap"),
    "corpus/audit-known.txt": ("corpus_known", "idioms", "fix each audited port"),
    "corpus/differential-known.txt": ("corpus_known", "idioms", "fix each differing port"),
    "corpus/generators-known.txt": ("corpus_known", "idioms", "write each generator"),
    "corpus/port-check-failing.txt": ("skip_lists", "idioms", "fix each port"),
    "corpus/gold/GOLD.txt": (None, "idioms", "data: the gold set"),
    "corpus/newcomer/acceptance.txt": (None, "idioms", "data: newcomer acceptance expectations"),
    "corpus/sets/ecnerwala.txt": (None, "idioms", "data: a corpus set"),
    "corpus/sets/sketches.txt": (None, "idioms", "data: a corpus set"),
    "corpus/sets/v03.txt": (None, "idioms", "data: a corpus set"),
    "crates/cpl-cli/tests/std-check-errors.txt": ("known_lists", "stdlib", "kept empty"),
    "crates/cpl-cli/tests/tool-gaps.txt": ("skip_lists", "tools", "close each gap"),
    "crates/cpl-cli/tests/vm_fallbacks.txt": ("vm_fallbacks", "tech-debt", "VM runs every program"),
    "crates/cpl-lint/corpus_warnings.txt": ("known_lists", "lint-fmt", "fix or accept each warning"),
    "crates/cpl-stress/tests/generate-gaps.txt": ("skip_lists", "stress", "generate each shape"),
    "scripts/public-hosts.txt": (None, "infra", "data: hosts the public-history scan allows"),
    "scripts/std-lint-baseline.txt": ("known_lists", "stdlib", "fix each std lint finding"),
    "spec/conformance/permits.txt": ("permits", "spec", "a case per permit"),
    "spec/conformance/prelude-gaps.txt": ("prelude_gaps", "stdlib", "a case per prelude name"),
    "spec/conformance/rule-gaps.txt": ("rule_gaps", "spec", "deleted: conformance-coverage.py fails on any rule gap"),
    "spec/conformance/sig-gaps.txt": ("sig_gaps", "spec", "deleted: conformance-sigs.py fails on any mismatch"),
    "spec/conformance/trait-gaps.txt": ("trait_gaps", "spec", "deleted: conformance-traits.py fails on any missing case"),
    "spec/idioms/lints-known.txt": ("idiom_lint_gaps", "idioms", "deleted: idioms-catalog.py fails on any finding"),
    "web/scripts/doc-examples-failing.txt": ("skip_lists", "site", "fix each example"),
    "web/scripts/practice-mutants-equivalent.txt": ("known_lists", "site", "data: equivalent mutants"),
    "web/tests/a11y-known.txt": ("xfail_markers", "site", "fix each a11y finding"),
    "web/tests/completion-known.txt": ("known_lists", "site", "fix each completion miss"),
    "web/tests/search-known.txt": ("known_lists", "site", "fix each search miss"),
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
CP_BINDING = re.compile(r"\b(?:let|var|fn|type|struct|enum|for)\s+(?:mut\s+)?\(?([A-Za-z_]\w*)")


def idiom_edit_stateable(before: str, after: str) -> bool:
    """Whether a lint record's before -> after edit is what a `@lint(from:, to:)`
    pattern states (decision 512): each changed line run stays in one block (a
    statement run), and no two runs change the same name bound in the file (then
    they are one edit at two places, not two findings)."""
    old = [line.rstrip() for line in before.split("\n")]
    new = [line.rstrip() for line in after.split("\n")]
    hunks = [o for o in difflib.SequenceMatcher(None, old, new, autojunk=False).get_opcodes() if o[0] != "equal"]
    bound = set(CP_BINDING.findall(before + "\n" + after))
    seen: set[str] = set()
    for _, i1, i2, j1, j2 in hunks:
        run = [line for line in old[i1:i2] if line.strip()]
        if any(_indent(line) < _indent(run[0]) for line in run):
            return False
        names = bound & (set(re.findall(r"[A-Za-z_]\w*", " ".join(old[i1:i2])))
                         ^ set(re.findall(r"[A-Za-z_]\w*", " ".join(new[j1:j2]))))
        if names & seen:
            return False
        seen |= names
    return bool(hunks)


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


HARD_ZERO = frozenset({
    "own_databases", "module_loaders", "web_skip_only", "todo_fixme",
    "pending_tests", "edition_refs", "uncited_rules", "decision_dup_numbers", "why_written_twice", "authority_drift",
    "continue_on_error", "unregistered_lists", "codegen_effect_guesses", "spec_std_refs",
    "stress_hand_json", "lint_dup_helpers", "lint_post_filters",
    "measure_broken", "mono_unguarded", "ts_ignore",
    "site_rederived", "browser_lsp_gaps",
    "unrun_script_tests",
    "permits", "written_twice",
    "plan_delta_lines",
    "trait_gaps",
    "idiom_lint_gaps",
    "rule_gaps",
    "direct_python",
    "sig_gaps",
    "xfail_markers",
})

# name -> (label, measure, ratcheted). Informational counts are shown, never gated.
COUNTERS = {
    "stdlib_todo": ("stdlib todo stubs", stdlib_todo, True),
    "rust_ignore": ("Rust #[ignore] attributes", rust_ignore, True),
    "pending_tests": ("crates/cpl-cli/tests/pending/ files", pending_tests, True),
    "dead_code_allows": ("allow(dead_code) attributes", dead_code_allows, True),
    "external_crates": ("third-party packages in Cargo.lock", external_crates, True),
    "written_twice": ("hand-kept files listing most of a sourced fact", written_twice, True),
    "bench_lists": ("entries in bench/*.txt exemption lists (scoreboard K5)", bench_lists, True),
    "skip_lists": ("entries in skip, xfail, known-failure and gap lists", skip_lists, True),
    "corpus_test_files": ("files committed under corpus/problems/*/tests/", corpus_test_files, True),
    "edition_refs": ("dead 0.2 edition references", edition_refs, True),
    "ir_builtin_names": ("names string-matched in lower/call.rs and lower/call/", ir_builtin_names, True),
    "prelude_privilege": ("public std names in cpl-check prelude.rs tables", prelude_privilege, True),
    "backend_std_names": ("std type names in IR/interp/codegen", backend_std_names, True),
    "lint_std_names": ("std fn names string-matched in lint rules", lint_std_names, True),
    "lint_type_syntax": ("lint rule lines reading a written type instead of checker facts", lint_type_syntax, True),
    "idioms_evidenced_unlinted": ("non-idiomatic spots with evidence that no lint teaches", idioms_evidenced_unlinted, True),
    "dup_fn_bodies": ("copied fn bodies across files", dup_fn_bodies, True),
    "near_dup_fns": ("near-identical fns across crates", near_dup_fns, True),
    "consumer_rederived": ("tool sites re-deriving checker facts", consumer_rederived, True),
    "site_rederived": ("web/ sources re-deriving tool data", site_rederived, True),
    "browser_lsp_gaps": ("language-server capabilities the browser editor never asks for", browser_lsp_gaps, True),
    "layer_violations": ("forbidden shipped crate edges", layer_violations, True),
    "tree_walker": ("second interpreter engine", tree_walker, True),
    "v02_port_files": ("corpus ports not yet re-ported in v0.3 style", v02_port_files, True),
    "direct_python": ("direct python3 calls in gate/justfile", direct_python, True),
    "unrun_script_tests": ("script tests the gate never runs", unrun_script_tests, True),
    "decision_dup_numbers": ("decision numbers headed twice in decisions.md", decision_dup_numbers, True),
    "lint_dup_helpers": ("helper fns copied across lint rule files", lint_dup_helpers, True),
    "plan_delta_lines": ("lines of notes under plan/domains/*/delta", plan_delta_lines, True),
    "why_written_twice": ("reasoning written in two homes, or rationale under spec/", why_written_twice, True),
    "authority_drift": ("authority sentence missing from a home, or contradicted", authority_drift, True),
    "conformance_failing": ("conformance cases not passing in research/scoreboard.json (Content)", conformance_failing, True),
    "conformance_wrong_output": ("conformance cases with wrong output in research/scoreboard.json (Content)",
                                 conformance_wrong_output, True),
    "vm_fallbacks": ("tree-walker fallbacks over tests/programs", vm_fallbacks, True),
    "own_databases": ("Database built outside cpl-check", own_databases, True),
    "stress_hand_json": ("hand-built JSON in cpl-stress and cpl-wasm", stress_hand_json, True),
    "helper_env_vars": ("CPL_* helper side channels beyond CPL_SEED", helper_env_vars, True),
    "cpp_deep_copies": ("deep copies in the corpus C++ (read_only_copies)", cpp_deep_copies, True),
    "cpp_std_functions": ("std::function in the corpus C++ (read_only_copies)", cpp_std_functions, True),
    "codegen_effect_guesses": ("codegen effect or purity guesses outside the Uses table", codegen_effect_guesses, True),
    "rs_near_cap": ("rs files within 100 lines of the cap", rs_near_cap, False),
    "stale_arch_paths": ("missing paths cited by ARCHITECTURE docs", stale_arch_paths, False),
    "crate_edges": ("shipped crate edges", crate_edges, False),
    "dev_crate_edges": ("test-only crate edges", dev_crate_edges, False),
    "spec_std_refs": ("spec/std references", spec_std_refs, False),
    # The total is shown; `register_rows_by_domain` gates each domain's count.
    "register_rows": ("quality-register rows naming no existing check", register_rows, False),
    "escapes_open": ("escaped bugs whose suite isn't fixed yet", escapes_open, False),
    "uncited_rules": ("lints and gates citing no handbook rule", uncited_rules, True),
    "unwrap_input": ("unwrap/expect/panic!/unreachable! in shipped crate code", unwrap_input, True),
    "unsupported_sites": ("lowering unsupported( calls and E0501 literals", unsupported_sites, True),
    "lsp_string_types": ("LSP lines reading types out of rendered text", lsp_string_types, True),
    "lint_post_filters": ("entries of cpl-lint after-the-fact tables", lint_post_filters, True),
    "known_lists": ("entries of known-finding lists", known_lists, True),
    "prelude_gaps": ("prelude names with no conformance case", prelude_gaps, True),
    "permits": ("entries of spec/conformance/permits.txt", permits, True),
    "trait_gaps": ("entries of spec/conformance/trait-gaps.txt", trait_gaps, True),
    "rule_gaps": ("entries of spec/conformance/rule-gaps.txt", rule_gaps, True),
    "sig_gaps": ("entries of spec/conformance/sig-gaps.txt", sig_gaps, True),
    "idiom_lint_gaps": ("entries of spec/idioms/lints-known.txt", idiom_lint_gaps, True),
    "or_true": ("`|| true` anywhere (scoreboard CI2)", or_true, True),
    "continue_on_error": ("continue-on-error jobs and steps (scoreboard CI1)", continue_on_error, True),
    "trace_engine_refs": ("CPL_TRACE_ENGINE mentions (scoreboard F3, Core)", trace_engine_refs, True),
    "corpus_known": ("entries of corpus/*-known.txt (scoreboard K6, Content)", corpus_known, True),
    "allow_attrs": ("#[allow(...)] in crates (Core, Consumers)", allow_attrs, True),
    "unsafe_blocks": ("unsafe blocks, fns and impls in crates (Core)", unsafe_blocks, True),
    "cpl_env_reads": ("CPL_* env reads outside the config module (Consumers)", cpl_env_reads, True),
    "test_sleeps": ("sleeps in tests (Factory)", test_sleeps, True),
    "ci_retries": ("--retry/--retries in CI and scripts (Factory)", ci_retries, True),
    "hardcoded_codes": ("E0xxx literals outside cpl-check and cpl-diag (Consumers)", hardcoded_codes, True),
    "xfail_markers": ("xfail/known-fail markers and lists (Content)", xfail_markers, True),
    "todo_fixme": ("TODO/FIXME in std, web and scripts", todo_fixme, True),
    "web_skip_only": ("web .skip(/.only( (Factory)", web_skip_only, True),
    "ts_ignore": ("web @ts-ignore (Factory)", ts_ignore, True),
    "habit_matchers": ("fns in hand-written habit matchers (Consumers)", habit_matchers, True),
    "rust_idiom_lints": ("idiom lints in Rust a std @lint from/to could state (Consumers)", rust_idiom_lints, True),
    "unchecked_fix_paths": ("fix appliers outside the re-checking gate (Consumers)", unchecked_fix_paths, True),
    "stress_engines": ("stress-case runners picking a verdict (Consumers)", stress_engines, True),
    "dream_unverified": ("dream.cp files with no passing stress record", dream_unverified, True),
    "measure_broken": ("plan measure.py scripts with no test", measure_broken, True),
    "mono_unguarded": ("monomorphization with no termination guard", mono_unguarded, True),
    "std_shadowed": ("std names lowering bypasses off the builtin floor", std_shadowed, True),
    "corpus_walkers": ("files listing corpus/problems besides scripts/corpus.py", corpus_walkers, True),
    "json_outside_cpl_json": ("serde_json or hand-built JSON outside cpl-json", json_outside_cpl_json, True),
    "config_schemas": ("cp.toml/toml:: readers outside the config module", config_schemas, True),
    "std_doc_parsers": ("`///` doc-comment readers outside cpl-syntax/cpl-doc", std_doc_parsers, True),
    "edit_engines": ("text edits applied outside cpl-diag/cpl-fmt", edit_engines, True),
    "codemods_outside_fix": ("scripts rewriting .cp files instead of cpl fix", codemods_outside_fix, True),
    "module_loaders": ("Database::new outside cpl-check", module_loaders, True),
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


def measure(root: Path = ROOT, max_rs_lines: int = DEFAULT_MAX_RS_LINES) -> dict:
    """Current file sizes over the cap, every ratcheted count, and the crate edges."""
    data: dict = {"max_rs_lines": max_rs_lines, "rs_file_size_allowlist": rs_over_cap(root, max_rs_lines)}
    for name, (_, fn, gated) in COUNTERS.items():
        if gated:
            data[name] = len(fn(root))
    data["register_rows_by_domain"] = register_rows_by_domain(root)
    data["crate_edges"] = crate_edges(root)
    data["a4_edges"] = a4_edges(root)
    data["rs_near_cap"] = sorted(_near_cap(root, max_rs_lines))
    return data


def improved(actual: dict, baseline: dict) -> list[str]:
    """Names to pass to `--update NAME...` to tighten exactly what shrank, and nothing a stale tree measured."""
    names = []
    if actual.get("rs_file_size_allowlist") != baseline.get("rs_file_size_allowlist", actual.get("rs_file_size_allowlist")):
        names.append("rs_over_cap")
    names += [n for n, (_, _, g) in COUNTERS.items() if g and n in baseline and int(actual.get(n, 0)) < int(baseline[n])]
    if any(n < baseline.get("register_rows_by_domain", {}).get(d, n)
           for d, n in actual.get("register_rows_by_domain", {}).items()):
        names.append("register_rows_by_domain")
    if set(baseline.get("crate_edges", [])) - set(actual.get("crate_edges", [])):
        names.append("crate_edges")
    if set(baseline.get("rs_near_cap", [])) - set(actual.get("rs_near_cap", [])):
        names.append("rs_near_cap")
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

    return errors, warnings, notes


def unexplained_bumps(root: Path, baseline: dict, ref: str = "origin/main") -> list[str]:
    """Baseline entries raised above `ref`'s without a decisions.md line naming them.

    A bump is a decision: the added decisions.md lines must name the counter
    (e.g. `ir_builtin_names`) or the allowlisted file.
    """
    shown = _git(root, "show", f"{ref}:scripts/debt-ratchet.json")
    if not shown:
        return []
    old = json.loads("\n".join(shown))
    raised = [n for n, (_, _, g) in COUNTERS.items()
              if g and n in old and int(baseline.get(n, 0)) > int(old[n])]
    old_allow = old.get("rs_file_size_allowlist", {})
    raised += [rel for rel, cap in baseline.get("rs_file_size_allowlist", {}).items()
               if cap > old_allow.get(rel, 0)]
    if "rs_near_cap" in old and set(baseline.get("rs_near_cap", [])) - set(old["rs_near_cap"]) - set(old_allow):
        raised.append("rs_near_cap")
    zero = [n for n in raised if n in COUNTERS and int(old.get(n, 1)) == 0]
    if zero:
        return [f"{n}: reached 0 on {ref}; a counter at 0 is never raised" for n in zero]
    if not raised:
        return []
    base = (_git(root, "merge-base", ref, "HEAD") or [ref])[0]
    added = "\n".join(l for l in _git(root, "diff", base, "--", "decisions.md")
                      if l.startswith("+") and not l.startswith("+++"))
    return [f"{n}: baseline raised above {ref} without a decisions.md line naming it "
            f"(a bump is a decision; add one, or remove the debt)" for n in raised if n not in added]


def inherited_baseline(baseline: dict, upstream: dict) -> dict:
    """`baseline` raised to what `upstream` (main) already measures, so a check
    fails only on debt this tree adds on top of main."""
    eff = json.loads(json.dumps(baseline))
    for name, (_, _, gated) in COUNTERS.items():
        if name in HARD_ZERO:
            eff[name] = 0
        elif gated and name in eff and name in upstream:
            eff[name] = max(int(eff[name]), int(upstream[name]))
    allow = eff.setdefault("rs_file_size_allowlist", {})
    for rel, lines in upstream.get("rs_file_size_allowlist", {}).items():
        allow[rel] = max(allow.get(rel, 0), lines)
    for key in ("crate_edges", "rs_near_cap"):
        if key in eff and key in upstream:
            eff[key] = sorted(set(eff[key]) | set(upstream[key]))
    return eff


class UpstreamUnknown(Exception):
    """`ref` exists but could not be checked out to measure."""


def measure_upstream(root: Path, max_rs_lines: int, ref: str = "origin/main") -> dict | None:
    """Measures `ref` in a scratch worktree; None when `ref` is missing, or HEAD
    is `ref` in CI (a run on main), so main's own debt still fails there. A
    local commit on a fresh tip also has HEAD at `ref`, and its base is `ref`."""
    head, up = (_git(root, "rev-parse", "-q", "--verify", r, env=_git_env()) for r in ("HEAD", ref))
    if not head or not up or (head == up and os.environ.get("CI")):
        return None
    with tempfile.TemporaryDirectory() as tmp:
        wt = Path(tmp) / "main"
        add = subprocess.run(["git", "-C", str(root), "worktree", "add", "-q", "--detach", str(wt), up[0]],
                             capture_output=True, text=True, env=_git_env())
        if add.returncode != 0:
            raise UpstreamUnknown(add.stderr.strip() or f"git worktree add exited {add.returncode}")
        try:
            return measure(wt, max_rs_lines)
        finally:
            subprocess.run(["git", "-C", str(root), "worktree", "remove", "--force", str(wt)],
                           capture_output=True, env=_git_env())


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Check or update the tech-debt ratchet.")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="Verify against the ratchet file.")
    group.add_argument("--update", nargs="*", metavar="NAME",
                       help="Write current measurements to the ratchet file; with NAMEs, only those entries.")
    group.add_argument("--show", nargs="*", metavar="NAME", help="Print `name count` for counters.")
    group.add_argument("--list", metavar="NAME", help="Print the offenders behind one counter.")
    group.add_argument("--zero", action="store_true",
                       help="Fail while any gated counter is above 0: main's own debt, reported red.")
    parser.add_argument("--ratchet", type=Path, default=DEFAULT_RATCHET, help="Path to debt-ratchet.json.")
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root.")
    args = parser.parse_args(argv)

    max_lines = DEFAULT_MAX_RS_LINES
    if args.ratchet.is_file():
        max_lines = int(json.loads(args.ratchet.read_text(encoding="utf-8")).get("max_rs_lines", max_lines))

    if args.zero:
        left = {n: len(fn(args.root)) for n, (_, fn, gated) in COUNTERS.items() if gated}
        left = {"rs_over_cap": len(rs_over_cap(args.root, max_lines)), **left}
        left = {n: c for n, c in left.items() if c}
        for n, c in left.items():
            print(f"debt-ratchet: {n} {c} (target 0; `--list {n}` shows them)")
        print(f"debt-ratchet: {len(left)} counter(s) above 0" if left else "debt-ratchet: every counter at 0")
        return 1 if left else 0

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
        raised = [n for n in sorted(HARD_ZERO) if int(data.get(n, 0)) > 0]
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
    actual = measure(args.root, max_lines)
    errors, warnings, notes = evaluate(actual, baseline)
    errors += unexplained_bumps(args.root, baseline)
    if errors:
        try:
            upstream = measure_upstream(args.root, max_lines)
        except UpstreamUnknown as e:
            # Unable to tell main's debt from this change's: warn, and leave
            # the verdict to CI on main rather than block a commit.
            warnings += [f"could not measure origin/main ({e}); not blocking: {e2}" for e2 in errors]
            upstream, errors = None, []
        if upstream is not None:
            mine = evaluate(actual, inherited_baseline(baseline, upstream))[0]
            warnings += [f"already on main, not from this change (infra fixes it): {e}"
                         for e in errors if e not in mine]
            errors = mine

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
        names = " ".join(improved(actual, baseline))
        print(f"debt-ratchet: {len(notes)} item(s) improved; tighten with "
              f"`python3 scripts/debt-ratchet.py --update {names}`")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())

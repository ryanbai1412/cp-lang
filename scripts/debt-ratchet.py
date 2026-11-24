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
import functools
import glob
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
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
# Near copies: Jaccard similarity of 4-token shingles (numbers folded to `0`),
# between fns in different crates; exact copies count under dup_fn_bodies.
NEAR_DUP_MIN = 0.75
NEAR_DUP_COMMON = 30  # a shingle in more fns than this is boilerplate, not evidence
TOKEN_RE = re.compile(r'"(?:\\.|[^"\\])*"|\d[\w.]*|\w+|\S')
# Tools that read facts through cpl-check rather than recompute them.
CONSUMERS = ("cpl-lint", "cpl-lsp", "cpl-cli", "cpl-stress", "cpl-codegen", "cpl-wasm")
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
WRITTEN_TWICE_CHECKED = {"handbook/design/architecture.md", "scripts/std-lint-baseline.txt",
                         "web/tests/search-tasks.mjs"}


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
    subdirs = [f"crates/{c}/src" for c in BACK_ENDS]
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


def near_dup_fns(root: Path) -> list[str]:
    """Each pair of near-identical fns (>= DUP_MIN_LINES, similarity >= NEAR_DUP_MIN) in different crates."""
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
    return [x for _, x in sorted(out)]


def consumer_rederived(root: Path) -> list[str]:
    """Sites where a tool recomputes what cpl-check provides: text compared against a
    type name, or a hard-coded checker error code. Own databases count under own_databases."""
    types = set(CORE_TYPES)
    for p in sorted((root / "std").glob("*.cp")):
        types.update(re.findall(r"^\s*pub\s+(?:struct|enum|trait|type)\s+(\w+)", p.read_text(), re.M))
    out = []
    for crate in CONSUMERS:
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
    """`spec/std` mentions outside history (the stdlib moved to `std/`; decision 307)."""
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


def unrun_script_tests(root: Path) -> list[str]:
    """`scripts/test_*.py` files the gate never names, so nothing runs them."""
    gate = root / "scripts/check.sh"
    text = gate.read_text() if gate.is_file() else ""
    return sorted(_rel(root, p) for p in (root / "scripts").glob("test_*.py") if p.name not in text)


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


def plan_delta_lines(root: Path) -> list[str]:
    """Lines of working notes under plan/domains/*/delta, to fold into spec/ or archive."""
    return [f"{f}:{i}" for f in _git(root, "ls-files", "plan/domains/*/delta/*")
            for i, _ in enumerate(_lines(root / f), 1)]


def _near_cap(root: Path, max_rs_lines: int = DEFAULT_MAX_RS_LINES) -> dict[str, int]:
    out = {}
    for p in _rs_files(root, "crates"):
        n = len(_lines(p))
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


def own_databases(root: Path) -> list[str]:
    """`Database::new()` outside tests and cpl-check: a tool re-deriving what one session answers."""
    out = []
    for p in _rs_files(root, "crates", tests=False):
        rel = _rel(root, p)
        if rel.startswith("crates/cpl-check/") or "/examples/" in rel or rel.endswith("_tests.rs"):
            continue
        out += [f"{rel}:{i}" for i, line in enumerate(_lines(p), 1) if "Database::new()" in line]
    return out


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


def stale_arch_paths(root: Path) -> list[str]:
    """Backticked repo paths in ARCHITECTURE docs that do not exist.

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
    gate (oversize, unmatched), so the bench gates hold only once there are none."""
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


# Counters that are paid off: any instance fails, and no baseline, `--update` or
# main's own measurement can raise them above 0. Every gated counter whose
# committed baseline is 0 is listed here (scripts/test_debt_ratchet.py checks).
HARD_ZERO = frozenset({
    "pending_tests", "edition_refs", "uncited_rules", "decision_dup_numbers",
})

# name -> (label, measure, ratcheted). Informational counts are shown, never gated.
COUNTERS = {
    "stdlib_todo": ("stdlib todo stubs", stdlib_todo, True),
    "rust_ignore": ("Rust #[ignore] attributes", rust_ignore, True),
    "pending_tests": ("crates/cpl-cli/tests/pending/ files", pending_tests, True),
    "dead_code_allows": ("allow(dead_code) attributes", dead_code_allows, True),
    "external_crates": ("third-party packages in Cargo.lock", external_crates, True),
    "written_twice": ("hand-kept files listing most of a sourced fact", written_twice, True),
    "bench_lists": ("entries in bench/*.txt exemption lists", bench_lists, True),
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
    "layer_violations": ("forbidden shipped crate edges", layer_violations, True),
    "tree_walker": ("second interpreter engine", tree_walker, True),
    "v02_port_files": ("v0.2 corpus port.cp files", v02_port_files, True),
    "direct_python": ("direct python3 calls in gate/justfile", direct_python, True),
    "unrun_script_tests": ("script tests the gate never runs", unrun_script_tests, True),
    "decision_dup_numbers": ("decision numbers headed twice in decisions.md", decision_dup_numbers, True),
    "lint_dup_helpers": ("helper fns copied across lint rule files", lint_dup_helpers, True),
    "plan_delta_lines": ("lines of notes under plan/domains/*/delta", plan_delta_lines, True),
    "vm_fallbacks": ("tree-walker fallbacks over tests/programs", vm_fallbacks, True),
    "own_databases": ("Database::new() outside tests and cpl-check", own_databases, True),
    "cpp_deep_copies": ("deep copies in the corpus C++ (read_only_copies)", cpp_deep_copies, True),
    "cpp_std_functions": ("std::function in the corpus C++ (read_only_copies)", cpp_std_functions, True),
    "rs_near_cap": ("rs files within 100 lines of the cap", rs_near_cap, False),
    "stale_arch_paths": ("missing paths cited by ARCHITECTURE docs", stale_arch_paths, False),
    "crate_edges": ("shipped crate edges", crate_edges, False),
    "dev_crate_edges": ("test-only crate edges", dev_crate_edges, False),
    "spec_std_refs": ("spec/std references", spec_std_refs, False),
    # The total is shown; `register_rows_by_domain` gates each domain's count.
    "register_rows": ("quality-register rows naming no existing check", register_rows, False),
    "escapes_open": ("escaped bugs whose suite isn't fixed yet", escapes_open, False),
    "uncited_rules": ("lints and gates citing no handbook rule", uncited_rules, True),
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
        if cur > base:
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

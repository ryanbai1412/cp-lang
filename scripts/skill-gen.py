#!/usr/bin/env python3
"""Generate and check the bundled CP Lang v0.3 AI skill (`skills/cp-lang/`).

Usage:
  python3 scripts/skill-gen.py            # regenerate skills/cp-lang/stdlib/*.md
  python3 scripts/skill-gen.py --check    # verify no drift, front matter, and all cp code blocks
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent
SPEC_STD = ROOT / "spec/std"
SKILL_DIR = ROOT / "skills/cp-lang"
STDLIB_OUT = SKILL_DIR / "stdlib"

FORBIDDEN_RE = re.compile(
    r"\bCPL_SPEC\b|\bCPL_EDITION\b|(?<!spec/)stdlib/[a-z0-9_]+\.cp\b"
)
FENCE_RE = re.compile(r"^```([^\n]*)\n(.*?)^```[ \t]*$", re.M | re.S)

MAX_SKILL_LINES = 160


@dataclass
class DocInfo:
    summary: str = ""
    cost: str = ""
    traps: str = ""
    empty: str = ""
    ties: str = ""
    order: str = ""
    examples: list[str] = field(default_factory=list)


@dataclass
class Member:
    kind: str
    name: str
    signature: str
    doc: DocInfo
    is_getter: bool = False


@dataclass
class TypeDecl:
    kind: str
    name: str
    header: str
    doc: DocInfo
    methods: list[Member] = field(default_factory=list)


@dataclass
class ModuleDoc:
    name: str
    summary: str
    prelude_names: list[str]
    types: list[TypeDecl]
    functions: list[Member]
    aliases: list[Member]


MODULE_EXAMPLES: dict[str, str] = {
    "algebra": """\
let xs = [3, 1, 4, 1, 5]
print(xs.sorted(by: x => desc(x)))
// output: 5 4 3 1 1
""",
    "bag": """\
var s = SortedSet<Int>()
s.add(5)
s.add(2)
s.add(8)
print(s.range(3..).first!, s.len)
// output: 5 3
""",
    "beats": """\
domain I = 0..4
var st = BeatsSegTree<I>([5, 2, 8, 3])
st.chmin(I(0)..=I(3), 4)
print(st.sum(I(0)..=I(3)))
// output: 13
""",
    "bitset": """\
domain B = 0..8
var bs = Bitset<B>()
bs.add(B(1))
bs.add(B(4))
print(bs.count)
// output: 2
""",
    "deque": """\
var dq = Deque<Int>()
dq.push(2)
dq.push_front(1)
dq.push(3)
print(dq.pop_front()!, dq.pop()!, dq.len)
// output: 1 3 1
""",
    "dp": """\
var lc = LiChao<Line, Int>(min, 0..=10)
lc.add(Line(2, 1))
lc.add(Line(1, 5))
print(lc.at(1)!, lc.at(10)!)
// output: 3 15
""",
    "dsu": """\
domain V = 0..4
var d = Dsu<V>()
d.unite(V(0), V(1))
d.unite(V(2), V(3))
print(d.components, yes(d.same(V(0), V(1))))
// output: 2 YES
""",
    "fenwick": """\
domain I = 0..4
var fw = Fenwick<I, Int>([I] i => Int(i) + 1)
fw.add(I(1), 10)
print(fw.fold(I(0)..=I(2)))
// output: 16
""",
    "flow": """\
domain V = 0..3
var g = FlowGraph<V>()
g.add(V(0), V(1), 5)
g.add(V(1), V(2), 3)
print(g.max_flow(V(0), V(2)))
// output: 3
""",
    "geometry": """\
let a = Pt(0, 0)
let b = Pt(3, 4)
print(a.dist2(b))
// output: 25
""",
    "graph": """\
domain V = 1..=3
let g = Graph.undirected([(V(1), V(2)), (V(2), V(3))])
let p = bfs(g, from: [V(1)])
print(p.dist[V(3)]!)
// output: 2
""",
    "grid": """\
domain R = 0..3
domain C = 0..3
let nbs = adj4((R(1), C(1))).collect()
print(nbs.len)
// output: 4
""",
    "hashmap": """\
use std.hashmap.{FlatMap}
var m = FlatMap<Int>(0)
m.set(10, 42)
print(m.get(10) ?? -1)
// output: 42
""",
    "heap": """\
var h = MinHeap<Int>()
h.push(7)
h.push(2)
h.push(5)
print(h.pop()!, h.pop()!)
// output: 2 5
""",
    "interval": """\
domain X = 0..10
var im = IntervalMap<X, Int>(init: 0)
im.assign(X(2)..X(6), 7)
print(im.get(X(3)), im.get(X(7)))
// output: 7 0
""",
    "io": """\
print(yes(true), yes(false))
// output: YES NO
""",
    "lazy": """\
let f = Flip.of(true, 10, 0)
print(f.now, f.flipped)
// output: 10 0
""",
    "marks": """\
domain V = 0..5
var mk = Marks<V>()
mk.add(V(2))
print(yes(V(2) in mk), yes(V(1) in mk))
mk.clear()
print(yes(V(2) in mk))
// output: YES NO
// output: NO
""",
    "math": """\
print(gcd(12, 18), lcm(4, 6), pow_mod(2, 10, 1000))
// output: 6 12 24
""",
    "matrix": """\
let a = Mat<Int>.identity(2)
print(a.get(0, 0), a.get(0, 1))
// output: 1 0
""",
    "mo": """\
use std.mo.{mo_order}
domain I = 0..6
let ord = mo_order([I(0)..I(5), I(1)..I(3), I(2)..I(4)])
print(ord.len)
// output: 3
""",
    "mod": """\
type Z = Mod<1_000_000_007>
let c = Comb<Z>(10)
print(c.choose(5, 2), Z(3) ** 4)
// output: 10 81
""",
    "ntt": """\
let c = convolve([1, 2], [3, 4])
print(c)
// output: 3 10 8
""",
    "opt": """\
let x: ?Int = Some(6)
let y: ?Int = None
print(yes(x.is_some_and(v => v > 3)), yes(y.is_none_or(v => v > 3)))
// output: YES YES
""",
    "pair": """\
domain V = 0..4
let p = Pair.new(V(3), V(1))
print(p.lo, p.hi)
// output: 1 3
""",
    "persistent": """\
use std.persistent.{PersistentSegTree}
domain I = 0..4
var pst = PersistentSegTree<I, Int>(+, [1, 2, 3, 4])
let r0 = pst.base
let r1 = pst.set(r0, I(1), 10)
print(pst.fold(r0, I(0)..=I(3)), pst.fold(r1, I(0)..=I(3)))
// output: 10 18
""",
    "poly": """\
print(eval([1, 2, 3], 2))
// output: 17
""",
    "prefix": """\
let p = Prefix([2, 5, 3])
print(p.all)
// output: 10
""",
    "rand": """\
use std.rand.{Rng}
var rng = Rng(42)
let x = rng.int(1, 10)
assert 1 <= x <= 10
""",
    "search": """\
let x = (0..=100).first_true(k => k * k >= 50)!
print(x)
// output: 8
""",
    "segtree": """\
domain I = 0..4
var st = SegTree<I, Int>.of(+, i => Int(i) + 1)
st.set(I(2), 10)
print(st.fold(I(1)..=I(3)), st.all)
// output: 16 17
""",
    "seq": """\
let xs = [3, 1, 4, 1, 5]
print(xs.sorted(), xs.sum(), inversions(xs))
// output: 1 1 3 4 5 14 3
""",
    "slope": """\
use std.slope.{Concave}
var f = Concave.flat(0..=10)
f.add_abs(3)
f.add_abs(7)
print(f.max())
// output: -4
""",
    "sparse": """\
domain I = 0..5
let sp = SparseTable<I, Int>(min, [4, 2, 7, 1, 5])
print(sp.fold(I(0)..I(3)), sp.fold(I(2)..=I(4)))
// output: 2 1
""",
    "strings": """\
print(z_function("ababa"))
// output: 5 0 3 0 1
""",
    "subset": """\
domain I = 0..3
let s = Subset<I>.single(I(1)).with(I(2))
print(s.len, yes(I(1) in s))
// output: 2 YES
""",
    "timetree": """\
use std.timetree.{TimeTree}
var tt = TimeTree<Int>(4)
tt.add_range(0..4, 1)
print(yes(tt.steps().len > 0))
// output: YES
""",
    "tree": """\
domain V = 1..=3
let g = Graph.undirected([(V(1), V(2)), (V(1), V(3))])
let t = Rooted(g, root: V(1))
print(t.depth(V(2)), t.size(V(1)))
// output: 1 3
""",
    "window": """\
let xs = [2, 1, 3, 2, 4]
let w = xs.windows(3).map(v => v.sum()).collect()
print(w)
// output: 6 6 9
""",
}


def parse_prelude(path: Path) -> dict[str, list[str]]:
    by_mod: dict[str, list[str]] = {}
    text = path.read_text()
    text = re.sub(r"//.*", "", text)
    for m in re.finditer(r"use\s+std\.(\w+)\.\{([^}]*)\}", text, re.S):
        mod = m.group(1)
        names = [n.strip() for n in m.group(2).split(",") if n.strip()]
        by_mod.setdefault(mod, []).extend(names)
    return {k: sorted(set(v)) for k, v in by_mod.items()}


def parse_doc_lines(lines: list[str]) -> DocInfo:
    info = DocInfo()
    prose: list[str] = []
    for raw in lines:
        s = raw.strip()
        if s.startswith("///"):
            s = s[3:].strip()
        m = re.match(r"^(cost|traps|empty|ties|order|example|see):\s*(.*)$", s)
        if m:
            k, v = m.group(1), m.group(2).strip()
            if k == "example":
                info.examples.append(v)
            elif k != "see":
                setattr(info, k, v)
        elif s and not prose:
            prose.append(s)
        elif s and not any(
            getattr(info, f) for f in ("cost", "traps", "empty", "ties", "order")
        ) and not info.examples:
            prose.append(s)
    joined = " ".join(prose)
    first_sent = re.split(r"(?<=[.!?])\s+", joined, maxsplit=1)[0] if joined else ""
    info.summary = first_sent.replace("|", "\\|")
    return info


def collect_doc_above(lines: list[str], idx: int) -> DocInfo:
    doc: list[str] = []
    j = idx - 1
    while j >= 0:
        s = lines[j].strip()
        if s.startswith("///"):
            doc.insert(0, s)
            j -= 1
        elif s.startswith("@") and not s.startswith("@!"):
            j -= 1
        else:
            break
    return parse_doc_lines(doc)


def read_header(lines: list[str], idx: int) -> str:
    parts: list[str] = []
    depth = 0
    for j in range(idx, min(idx + 12, len(lines))):
        line = re.sub(r"//.*", "", lines[j]).strip()
        if not line:
            break
        parts.append(line)
        for ch in line:
            if ch in "([{<":
                depth += 1
            elif ch in ")]}>":
                depth = max(0, depth - 1)
        joined = " ".join(parts)
        if depth == 0 and (
            joined.endswith(":")
            or re.search(r"\)\s*(?:->[^=]+)?\s*=\s*", joined)
            or (j == idx and not joined.endswith((",", "(", "->")))
        ):
            break
    sig = " ".join(parts)
    depth = 0
    cut = len(sig)
    for k, ch in enumerate(sig):
        if ch in "([{<":
            depth += 1
        elif ch in ")]}>":
            depth = max(0, depth - 1)
        elif depth == 0 and ch == "=" and sig[k - 1:k] not in ("=", "<", ">", "!") and sig[k + 1:k + 2] not in ("=", ">"):
            cut = k
            break
    sig = sig[:cut]
    sig = re.sub(r":\s*$", "", sig)
    sig = re.sub(r"\s+", " ", sig).strip()
    return sig


def parse_module(path: Path, prelude_names: list[str]) -> ModuleDoc:
    lines = path.read_text().splitlines()
    mod_docs: list[str] = []
    for line in lines:
        s = line.strip()
        if s.startswith("//!"):
            mod_docs.append(s[3:].strip())
        elif s and not s.startswith("use "):
            break
    summary = " ".join(mod_docs).strip()

    types: list[TypeDecl] = []
    functions: list[Member] = []
    aliases: list[Member] = []
    cur_type: TypeDecl | None = None

    for i, line in enumerate(lines):
        if not line.strip() or line.lstrip().startswith("//"):
            continue
        indent = len(line) - len(line.lstrip())
        s = line.strip()
        if indent == 0:
            cur_type = None
            m_type = re.match(r"^pub\s+(struct|enum|trait)\s+([A-Za-z_]\w*)", s)
            if m_type:
                kind, name = m_type.group(1), m_type.group(2)
                header = read_header(lines, i)
                header = re.sub(r"^pub\s+", "", header)
                td = TypeDecl(kind=kind, name=name, header=header, doc=collect_doc_above(lines, i))
                types.append(td)
                cur_type = td
                continue
            m_alias = re.match(r"^pub\s+(type|const)\s+([A-Za-z_]\w*)", s)
            if m_alias:
                kind, name = m_alias.group(1), m_alias.group(2)
                sig = re.sub(r"//.*", "", s).strip()
                sig = re.sub(r"^pub\s+", "", sig)
                aliases.append(Member(kind=kind, name=name, signature=sig, doc=collect_doc_above(lines, i)))
                continue
            m_fn = re.match(r"^(?:@get\s+)?pub\s+(?:@get\s+)?fn\s+([A-Za-z_]\w*|\([^)]+\)\s*\S+\s*\([^)]+\))", s)
            if m_fn:
                is_get = "@get" in s or (i > 0 and lines[i - 1].strip() == "@get")
                sig = read_header(lines, i)
                sig = re.sub(r"^(?:@get\s+)?pub\s+(?:@get\s+)?", "", sig)
                functions.append(Member(kind="fn", name=m_fn.group(1), signature=sig, doc=collect_doc_above(lines, i), is_getter=is_get))
        elif cur_type is not None and indent <= 4:
            m_fn = re.match(r"^(?:@get\s+)?pub\s+(?:@get\s+)?fn\s+([A-Za-z_]\w*)", s)
            if m_fn:
                is_get = "@get" in s or (i > 0 and lines[i - 1].strip() == "@get")
                sig = read_header(lines, i)
                sig = re.sub(r"^(?:@get\s+)?pub\s+(?:@get\s+)?", "", sig)
                cur_type.methods.append(Member(kind="fn", name=m_fn.group(1), signature=sig, doc=collect_doc_above(lines, i), is_getter=is_get))

    return ModuleDoc(
        name=path.stem,
        summary=summary,
        prelude_names=prelude_names,
        types=types,
        functions=functions,
        aliases=aliases,
    )


def render_member_line(m: Member) -> str:
    prefix = "@get " if m.is_getter else ""
    parts = [f"- `{prefix}{m.signature}`"]
    meta: list[str] = []
    if m.doc.cost:
        meta.append(m.doc.cost)
    if m.doc.summary:
        meta.append(m.doc.summary)
    if m.doc.traps and m.doc.traps.lower() != "never":
        meta.append(f"traps: {m.doc.traps}")
    if meta:
        parts.append(" — " + "; ".join(meta))
    return "".join(parts)


def render_module(mod: ModuleDoc) -> str:
    out: list[str] = [f"<!-- Generated by scripts/skill-gen.py from spec/std/{mod.name}.cp. Do not edit by hand. -->", ""]
    out.append(f"# std.{mod.name}")
    out.append("")
    if mod.summary:
        out.append(mod.summary)
        out.append("")
    if mod.prelude_names:
        names = ", ".join(f"`{n}`" for n in mod.prelude_names)
        out.append(f"In prelude (no `use` needed): {names}.")
        out.append("")
    else:
        out.append(f"Import: `use std.{mod.name}.{{...}}`.")
        out.append("")

    if mod.aliases:
        out.append("## Types & Constants")
        out.append("")
        for a in mod.aliases:
            out.append(render_member_line(a))
        out.append("")

    for td in mod.types:
        out.append(f"## `{td.header}`")
        out.append("")
        if td.doc.summary:
            out.append(td.doc.summary)
            out.append("")
        for m in td.methods:
            out.append(render_member_line(m))
        if td.methods:
            out.append("")

    if mod.functions:
        out.append("## Functions")
        out.append("")
        for f in mod.functions:
            out.append(render_member_line(f))
        out.append("")

    ex = MODULE_EXAMPLES.get(mod.name)
    if ex:
        out.append("## Example")
        out.append("")
        out.append("```cp")
        out.append(ex.rstrip("\n"))
        out.append("```")
        out.append("")

    return "\n".join(out)


def render_stdlib_index(mods: list[ModuleDoc]) -> str:
    out = [
        "<!-- Generated by scripts/skill-gen.py from spec/std/. Do not edit by hand. -->",
        "",
        "# Standard Library Reference",
        "",
        "Each page is generated from `spec/std/<module>.cp`.",
        "Prelude items need no `use` in programs; everything else uses `use std.<module>.{Name}`.",
        "",
        "| Module | Prelude exports | Summary |",
        "|---|---|---|",
    ]
    for m in mods:
        pre = ", ".join(f"`{n}`" for n in m.prelude_names[:5])
        if len(m.prelude_names) > 5:
            pre += f", +{len(m.prelude_names) - 5}"
        if not pre:
            pre = "*(import)*"
        summ = (m.summary or "").replace("|", "\\|")
        out.append(f"| [`std.{m.name}`]({m.name}.md) | {pre} | {summ} |")
    out.append("")
    return "\n".join(out)


def generate_all() -> dict[Path, str]:
    prelude = parse_prelude(SPEC_STD / "prelude.cp")
    mods: list[ModuleDoc] = []
    files: dict[Path, str] = {}
    for cp in sorted(SPEC_STD.glob("*.cp")):
        if cp.stem == "prelude":
            continue
        mod = parse_module(cp, prelude.get(cp.stem, []))
        mods.append(mod)
        files[STDLIB_OUT / f"{mod.name}.md"] = render_module(mod)
    files[STDLIB_OUT / "README.md"] = render_stdlib_index(mods)
    return files


def find_cpl() -> Path:
    for cand in (
        os.environ.get("CPL_BIN"),
        str(ROOT / "target/debug/cpl"),
        str(ROOT / "target/release/cpl"),
    ):
        if cand and Path(cand).is_file():
            return Path(cand)
    raise RuntimeError("cpl binary not found; run `cargo build -p cpl-cli` first")


def extract_cp_blocks(md_path: Path) -> list[tuple[int, str, str]]:
    text = md_path.read_text()
    blocks: list[tuple[int, str, str]] = []
    for m in FENCE_RE.finditer(text):
        info = m.group(1).strip()
        if not (info == "cp" or info.startswith("cp ")):
            continue
        line_no = text.count("\n", 0, m.start()) + 1
        blocks.append((line_no, info, m.group(2)))
    return blocks


def parse_io_annotations(code: str) -> tuple[str | None, list[str] | None]:
    inputs: list[str] = []
    outputs: list[str] = []
    for line in code.splitlines():
        s = line.strip()
        if s.startswith("// input:"):
            inputs.append(s[len("// input:"):].lstrip())
        elif s.startswith("// output:"):
            outputs.append(s[len("// output:"):].lstrip())
    inp = ("\n".join(inputs) + "\n") if inputs else None
    out = outputs if outputs else None
    return inp, out


def check_skill(cpl: Path) -> list[str]:
    errors: list[str] = []

    expected = generate_all()
    actual_paths = set(STDLIB_OUT.glob("*.md")) if STDLIB_OUT.is_dir() else set()
    for path, content in sorted(expected.items()):
        if not path.is_file():
            errors.append(f"missing generated file: {path.relative_to(ROOT)} (run scripts/skill-gen.py)")
        elif path.read_text() != content:
            errors.append(f"stale generated file: {path.relative_to(ROOT)} (run scripts/skill-gen.py)")
    for extra in sorted(actual_paths - set(expected.keys())):
        errors.append(f"extra file in skills/cp-lang/stdlib/: {extra.relative_to(ROOT)}")

    skill_md = SKILL_DIR / "SKILL.md"
    if not skill_md.is_file():
        errors.append("missing skills/cp-lang/SKILL.md")
    else:
        stext = skill_md.read_text()
        lines = stext.splitlines()
        if len(lines) > MAX_SKILL_LINES:
            errors.append(f"skills/cp-lang/SKILL.md is {len(lines)} lines (max {MAX_SKILL_LINES})")
        if not re.match(r"^---\n.*?^name:\s*\S+.*?^description:\s*\S+.*?^---\n", stext, re.M | re.S):
            errors.append("skills/cp-lang/SKILL.md missing YAML front matter with name and description")
        for supp in sorted(SKILL_DIR.glob("*.md")):
            if supp.name == "SKILL.md":
                continue
            if supp.name not in stext:
                errors.append(f"skills/cp-lang/SKILL.md does not link supplement {supp.name}")

    all_md = sorted(SKILL_DIR.rglob("*.md"))
    plain_blocks: list[tuple[Path, int, str]] = []
    error_blocks: list[tuple[Path, int, list[str], str]] = []

    for md in all_md:
        txt = md.read_text()
        for m in FORBIDDEN_RE.finditer(txt):
            ln = txt.count("\n", 0, m.start()) + 1
            errors.append(f"{md.relative_to(ROOT)}:{ln}: forbidden legacy reference `{m.group(0)}`")
        for line_no, info, code in extract_cp_blocks(md):
            em = re.match(r"^cp\s+error\(([^)]+)\)$", info)
            if em:
                codes = [c.strip() for c in em.group(1).split(",") if c.strip()]
                error_blocks.append((md, line_no, codes, code))
            elif info == "cp":
                plain_blocks.append((md, line_no, code))
            else:
                errors.append(f"{md.relative_to(ROOT)}:{line_no}: unknown cp fence info `{info}`")

    with tempfile.TemporaryDirectory(prefix="cpl-skill-check-") as tmp:
        tmp_dir = Path(tmp)
        file_map: dict[str, tuple[Path, int, str]] = {}
        batch_paths: list[str] = []
        for idx, (md, line_no, code) in enumerate(plain_blocks):
            p = tmp_dir / f"ok_{idx}.cp"
            p.write_text(code)
            file_map[str(p)] = (md, line_no, code)
            batch_paths.append(str(p))

        if batch_paths:
            res = subprocess.run([str(cpl), "check", *batch_paths], capture_output=True, text=True)
            if res.returncode != 0:
                for path_str in batch_paths:
                    r1 = subprocess.run([str(cpl), "check", path_str], capture_output=True, text=True)
                    if r1.returncode != 0:
                        md, line_no, _ = file_map[path_str]
                        msg = (r1.stderr or r1.stdout).strip().splitlines()[:4]
                        errors.append(
                            f"{md.relative_to(ROOT)}:{line_no}: `cpl check` failed:\n  " + "\n  ".join(msg)
                        )

        for idx, (md, line_no, code) in enumerate(plain_blocks):
            inp, exp_out = parse_io_annotations(code)
            has_read = bool(re.search(r"\bread(?:_line|_all)?\s*[<(]", code))
            has_print = bool(re.search(r"\bprint\s*\(", code))
            if exp_out is None and (has_read or not has_print):
                continue
            p = tmp_dir / f"ok_{idx}.cp"
            r = subprocess.run(
                [str(cpl), "run", str(p)],
                input=inp or "",
                capture_output=True,
                text=True,
                timeout=15,
            )
            if r.returncode != 0:
                msg = (r.stderr or r.stdout).strip().splitlines()[:4]
                errors.append(
                    f"{md.relative_to(ROOT)}:{line_no}: `cpl run` failed:\n  " + "\n  ".join(msg)
                )
            elif exp_out is not None:
                got = [ln.rstrip() for ln in r.stdout.strip().splitlines()]
                if got != exp_out:
                    errors.append(
                        f"{md.relative_to(ROOT)}:{line_no}: `cpl run` output mismatch: expected {exp_out!r}, got {got!r}"
                    )

        for idx, (md, line_no, codes, code) in enumerate(error_blocks):
            p = tmp_dir / f"err_{idx}.cp"
            p.write_text(code)
            r = subprocess.run([str(cpl), "check", str(p)], capture_output=True, text=True)
            combined = r.stderr + "\n" + r.stdout
            if r.returncode == 0:
                errors.append(
                    f"{md.relative_to(ROOT)}:{line_no}: expected error {codes}, but `cpl check` succeeded"
                )
            else:
                for c in codes:
                    if f"[{c}]" not in combined:
                        first = combined.strip().splitlines()[:3]
                        errors.append(
                            f"{md.relative_to(ROOT)}:{line_no}: expected [{c}], got:\n  " + "\n  ".join(first)
                        )

    return errors


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="verify generated files and all cp code blocks")
    ap.add_argument("--cpl", type=Path, default=None, help="path to cpl binary for --check")
    args = ap.parse_args(argv)

    if args.check:
        cpl = args.cpl if args.cpl is not None else find_cpl()
        errs = check_skill(cpl)
        if errs:
            for e in errs:
                print(f"skill-check: {e}", file=sys.stderr)
            print(f"skill-check: {len(errs)} error(s)", file=sys.stderr)
            return 1
        all_md = sorted(SKILL_DIR.rglob("*.md"))
        n_blocks = sum(len(extract_cp_blocks(p)) for p in all_md)
        print(f"skill-check: ok ({len(all_md)} files, {n_blocks} cp blocks verified)")
        return 0

    files = generate_all()
    STDLIB_OUT.mkdir(parents=True, exist_ok=True)
    for path, content in sorted(files.items()):
        path.write_text(content)
    for extra in set(STDLIB_OUT.glob("*.md")) - set(files.keys()):
        extra.unlink()
    print(f"skill-gen: wrote {len(files)} files under {STDLIB_OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

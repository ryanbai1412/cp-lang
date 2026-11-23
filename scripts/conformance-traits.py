#!/usr/bin/env python3
"""Check that every trait method of every claimer has a conformance case.

Traits come from `pub trait` bodies in std/*.cp and from the core traits
listed in spec/conformance/traits.toml. Claims come from `impl Trait for
Type:` lines in std/*.cp and from the core claimers in traits.toml.
Each (type, trait, method) needs a `[case]` in traits.toml, which becomes
spec/conformance/traits/<trait>/<type>--<method>.cp, or a line in
spec/conformance/trait-gaps.txt. `--check` fails when a generated case is
stale, a triple has neither, or a gap line names a triple that has a case or
no longer exists. Writing never adds a gap unless `--add-gaps` is given, so
the gap list only shrinks.
"""
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance"
STD = ROOT / "std"
OUT = CONF / "traits"
GAPS = CONF / "trait-gaps.txt"

TRAIT = re.compile(r"pub trait (\w+)(<[^>]*>)?(:\s*(.*))?$")
METHOD = re.compile(r"\s+(?:@get\s+)?fn (\w+)\(")
IMPL = re.compile(r"impl (\w+)(?:<[^>]*>)? for ([\w<>, \[\]]+):$")


def std_traits():
    traits = {}
    for f in sorted(STD.glob("*.cp")):
        name = None
        for line in f.read_text().split("\n"):
            m = TRAIT.match(line)
            if m:
                name = m.group(1)
                if "=" in line:
                    name = None
                else:
                    traits.setdefault(name, [])
                continue
            if name and line and not line[0].isspace():
                name = None
            elif name:
                mm = METHOD.match(line)
                if mm and mm.group(1) not in traits[name]:
                    traits[name].append(mm.group(1))
    return traits


PUB_TYPE = re.compile(r"pub (?:struct|enum|type) (\w+)")


def std_claims():
    """(type, trait) for every impl in std on a public type; test-only types are skipped."""
    texts = [f.read_text() for f in sorted(STD.glob("*.cp"))]
    public = {m.group(1) for t in texts for m in PUB_TYPE.finditer(t)}
    claims = set()
    for t in texts:
        for line in t.split("\n"):
            m = IMPL.match(line)
            if m:
                ty = re.sub(r"<.*", "", m.group(2)).strip()
                if ty in public:
                    claims.add((ty, m.group(1)))
    return claims


def triples(data):
    traits = std_traits()
    for t, v in data.get("trait", {}).items():
        if t in traits:
            raise SystemExit(f"traits.toml: trait {t} is declared in std; drop it here")
        traits[t] = v["methods"]
    claims = std_claims() | {(c["type"], c["trait"]) for c in data.get("claim", [])}
    out, problems = set(), []
    for ty, tr in sorted(claims):
        if tr not in traits:
            problems.append(f"{ty}: claims {tr}, which neither std nor traits.toml declares")
            continue
        for m in traits[tr]:
            out.add((tr, ty, m))
    return out, problems


def case_text(c):
    head = [f'// expect: ok, output "{c["out"]}"']
    if "input" in c:
        head.append(f'// input: "{c["input"]}"')
    if "rule" in c:
        head.append(f"// rule: {c['rule']}")
    return "\n".join(head) + "\n" + c["code"].strip("\n") + "\n"


def main():
    data = tomllib.loads((CONF / "traits.toml").read_text())
    all_t, problems = triples(data)
    cases = {}
    for key, c in data.get("case", {}).items():
        tr, ty, m = key.split("/")
        if (tr, ty, m) not in all_t:
            problems.append(f"case {key}: no such claim or method")
            continue
        cases[(tr, ty, m)] = c
    missing = sorted(all_t - set(cases))
    want = {OUT / tr / f"{ty}--{m}.cp": case_text(c) for (tr, ty, m), c in cases.items()}
    have = {p: p.read_text() for p in OUT.rglob("*.cp")} if OUT.exists() else {}
    gap_lines = [l for l in GAPS.read_text().split("\n") if l and not l.startswith("#")] if GAPS.exists() else []
    gaps = {tuple(l.split("/")) for l in gap_lines}
    new_gaps = [t for t in missing if t not in gaps]
    dead_gaps = sorted(gaps - set(missing))
    summary = f"traits: {len(all_t)} methods over claims, {len(cases)} with a case, {len(missing)} gaps"
    if "--check" in sys.argv:
        problems += [f"no case and no gap line: {'/'.join(t)}" for t in new_gaps]
        problems += [f"gap line covered or gone, delete it: {'/'.join(t)}" for t in dead_gaps]
        stale = sorted(str(p.relative_to(ROOT)) for p in set(want) | set(have) if want.get(p) != have.get(p))
        problems += [f"stale: {s}" for s in stale[:20]]
        if problems:
            print("trait gate failed:\n  " + "\n  ".join(problems))
            print("rerun: uv run scripts/conformance-traits.py")
            return 1
        print(summary)
        return 0
    if new_gaps and "--add-gaps" not in sys.argv:
        problems += [f"no case: {'/'.join(t)} (write one, or rerun with --add-gaps)" for t in new_gaps]
    if problems:
        print("\n".join(problems))
        return 1
    for p in set(have) - set(want):
        p.unlink()
    for p, t in want.items():
        p.parent.mkdir(parents=True, exist_ok=True)
        if have.get(p) != t:
            p.write_text(t)
    GAPS.write_text(
        "# (trait/type/method) with no case yet. Shrink-only: scripts/conformance-traits.py\n"
        "# never adds a line without --add-gaps; write the case in traits.toml instead.\n"
        + "".join("/".join(t) + "\n" for t in missing)
    )
    print(summary)
    return 0


if __name__ == "__main__":
    sys.exit(main())

"""Check that every trait method of every claimer has a conformance case.

Traits come from `pub trait` bodies in std/*.cp and from the core traits
listed in spec/conformance/traits.toml; a std trait listed there with some
of its methods needs cases for those only (the rest are hooks with defaults). Claims come from `impl Trait for
Type:` lines in std/*.cp and from the core claimers in traits.toml.
Each (type, trait, method) needs a `[case]` in traits.toml, which becomes
spec/conformance/traits/<trait>/<type>--<method>.cp. The gate fails when
a triple has no case or a generated case is stale; `--write` regenerates.
Enforces handbook/guidelines/conformance-checklist.md#conformancer5-where-a-rule-covers-a-space-generate-the-cases
"""
from __future__ import annotations

import re
import tomllib

import specgate
from specgate import Problem

HELP = "every (trait, claimer, method) has a traits.toml case; spec/conformance/traits/ generated (--write regenerates)"

TRAIT = re.compile(r"pub trait (\w+)(<[^>]*>)?(\([^)]*\))?(:\s*(.*))?$")
METHOD = re.compile(r"\s+(?:@get\s+)?fn (\w+)\(")
IMPL = re.compile(r"impl(?:<(?:[^<>]|<[^<>]*>)*>)? (\w+)(?:<[^>]*>)? for ([\w<>, \[\]:+]+):$")


def std_traits():
    traits = {}
    for f in sorted((specgate.ROOT / "std").glob("*.cp")):
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
PRIVATE_TRAIT = re.compile(r"(?m)^trait (\w+)")


def std_claims():
    """(type, trait) for every impl in std of a trait users can name, on a public type;
    test-only types and std's private helper traits are skipped."""
    texts = [f.read_text() for f in sorted((specgate.ROOT / "std").glob("*.cp"))]
    public = {m.group(1) for t in texts for m in PUB_TYPE.finditer(t)}
    private = {m.group(1) for t in texts for m in PRIVATE_TRAIT.finditer(t)}
    claims = set()
    for t in texts:
        for line in t.split("\n"):
            m = IMPL.match(line)
            if m:
                ty = re.sub(r"<.*", "", m.group(2)).strip()
                if ty in public and m.group(1) not in private:
                    claims.add((ty, m.group(1)))
    return claims


def triples(data):
    traits = std_traits()
    out, problems = set(), []
    for t, v in data.get("trait", {}).items():
        if t in traits and not set(v["methods"]) < set(traits[t]):
            problems.append(f"traits.toml: trait {t} is declared in std; drop it here")
        traits[t] = v["methods"]
    claims = std_claims() | {(c["type"], c["trait"]) for c in data.get("claim", [])}
    for ty, tr in sorted(claims):
        if tr not in traits:
            problems.append(f"{ty}: claims {tr}, which neither std nor traits.toml declares")
            continue
        for m in traits[tr]:
            out.add((tr, ty, m))
    return out, problems


def case_text(c):
    head = [f'// expect: ok, output "{c["out"].replace(chr(10), "\\n")}"']
    if "input" in c:
        head.append(f'// input: "{c["input"]}"')
    if "rule" in c:
        head.append(f"// rule: {c['rule']}")
    if "engines" in c:
        head.append(f"// engines: {c['engines']}")
    return "\n".join(head) + "\n" + c["code"].strip("\n") + "\n"


def args(p) -> None:
    p.add_argument("--write", action="store_true")


def run(a) -> list[Problem]:
    conf = specgate.CONFORMANCE
    data = tomllib.loads((conf / "traits.toml").read_text())
    all_t, found = triples(data)
    problems = [Problem("traits.toml", w) for w in found]
    cases = {}
    for key, c in data.get("case", {}).items():
        tr, ty, m = key.split("/")
        if (tr, ty, m) not in all_t:
            problems.append(Problem(f"traits.toml case {key}", "no such claim or method"))
        else:
            cases[(tr, ty, m)] = c
    problems += [Problem("traits.toml", f"no case: {'/'.join(t)} (write one)") for t in sorted(all_t - set(cases))]
    want = {f"{tr}/{ty}--{m}.cp": case_text(c) for (tr, ty, m), c in cases.items()}
    return problems or specgate.sync(conf / "traits", want, a.write, "traits", "**/*.cp")

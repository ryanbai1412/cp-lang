"""Check that every trait method of every claimer has a conformance case.

Traits come from `pub trait` bodies in std/*.cp and from the core traits
listed in spec/conformance/traits.toml; a std trait listed there with some
of its methods needs cases for those only (the rest are hooks with defaults). Claims come from `impl Trait for
Type:` lines in std/*.cp and from the core claimers in traits.toml.
A (type, trait, method) needs a `[case]` in traits.toml when the method has
no default or the type's impl writes it again; a default every claimer keeps
is one body, so one case on any claimer covers it (the rest are hooks with
defaults). A case becomes spec/conformance/traits/<trait>/<type>--<method>.cp,
run on the frozen `cpl` only when its pinned std has the claim. The gate fails
when a needed case is missing or a generated case is stale; `--write` regenerates.
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


def signature_end(lines, i):
    """The last line of the fn signature starting at lines[i], comments dropped."""
    depth, j = 0, i
    while True:
        s = lines[j].split("//")[0].rstrip()
        depth += s.count("(") - s.count(")")
        if depth <= 0 or j + 1 == len(lines):
            return s
        j += 1


def has_body(sig):
    return sig.endswith(":") or " = " in sig.split("->")[-1]


FROZEN_STD = "crates/cpl-check/frozen/std"


def std_traits(std="std"):
    """Each std trait's methods, and which of them have a default body."""
    traits, defaults = {}, {}
    for f in sorted((specgate.ROOT / std).glob("*.cp")):
        name = None
        lines = f.read_text().split("\n")
        for i, line in enumerate(lines):
            m = TRAIT.match(line)
            if m:
                name = m.group(1)
                if "=" in line:
                    name = None
                else:
                    traits.setdefault(name, [])
                    defaults.setdefault(name, set())
                continue
            if name and line and not line[0].isspace():
                name = None
            elif name:
                mm = METHOD.match(line)
                if mm and mm.group(1) not in traits[name]:
                    traits[name].append(mm.group(1))
                    if has_body(signature_end(lines, i)):
                        defaults[name].add(mm.group(1))
    return traits, defaults


OWN_METHOD = re.compile(r" {4}(?:@get\s+)?fn (\w+)\(")
PUB_TYPE = re.compile(r"pub (?:struct|enum|type) (\w+)")
PRIVATE_TRAIT = re.compile(r"(?m)^trait (\w+)")


def std_claims(std="std", types=frozenset()):
    """(type, trait) for every impl in std of a trait users can name, on a public type;
    test-only types and std's private helper traits are skipped."""
    texts = [f.read_text() for f in sorted((specgate.ROOT / std).glob("*.cp"))]
    public = types | {m.group(1) for t in texts for m in PUB_TYPE.finditer(t)}
    private = {m.group(1) for t in texts for m in PRIVATE_TRAIT.finditer(t)}
    claims, overrides, at = set(), set(), None
    for t in texts:
        for line in t.split("\n"):
            m = IMPL.match(line)
            if m:
                ty = re.sub(r"<.*", "", m.group(2)).strip()
                at = (ty, m.group(1)) if ty in public and m.group(1) not in private else None
                if at:
                    claims.add(at)
                continue
            if line and not line[0].isspace():
                at = None
            elif at and (mm := OWN_METHOD.match(line)):
                overrides.add((at[1], at[0], mm.group(1)))
    return claims, overrides


def triples(data):
    """Every (trait, claimer, method) a case may name, those that need one (a claimer's
    methods without a default, and the defaults its impl writes again), and for each default
    the claimers that keep it: one case on any of them covers its body."""
    traits, defaults = std_traits()
    out, need, problems, kept = set(), set(), [], {}
    for t, v in data.get("trait", {}).items():
        if t in traits and not set(v["methods"]) < set(traits[t]):
            problems.append(f"traits.toml: trait {t} is declared in std; drop it here")
        traits[t], defaults[t] = v["methods"], set()
    claims, overrides = std_claims()
    claims |= {(c["type"], c["trait"]) for c in data.get("claim", [])}
    for ty, tr in sorted(claims):
        if tr not in traits:
            problems.append(f"{ty}: claims {tr}, which neither std nor traits.toml declares")
            continue
        for m in traits[tr]:
            out.add((tr, ty, m))
            if m not in defaults[tr] or (tr, ty, m) in overrides:
                need.add((tr, ty, m))
            else:
                kept.setdefault((tr, m), set()).add((tr, ty, m))
    return out, need, problems, kept


def frozen_runs(data, types):
    """Whether the frozen `cpl` can run a (trait, claimer) case: it reads the std pinned at the
    freeze, so it knows that std's claims and the core traits std doesn't declare."""
    # The pin left core types such as Bool to the compiler, so the claimers named today count.
    pinned, live = std_claims(FROZEN_STD, types)[0], std_traits()[0]
    core = set(data.get("trait", {})) - set(live)
    return lambda tr, ty: (ty, tr) in pinned or tr in core


def case_text(c, frozen=True):
    head = [f'// expect: ok, output "{c["out"].replace(chr(10), "\\n")}"']
    if "input" in c:
        head.append(f'// input: "{c["input"]}"')
    if "rule" in c:
        head.append(f"// rule: {c['rule']}")
    # A claim the pinned std lacks runs only where std as it is is read: cpl-next.
    if "engines" in c or not frozen:
        head.append(f"// engines: {c.get('engines', 'next')}")
    return "\n".join(head) + "\n" + c["code"].strip("\n") + "\n"


def args(p) -> None:
    p.add_argument("--write", action="store_true")


def run(a) -> list[Problem]:
    conf = specgate.CONFORMANCE
    data = tomllib.loads((conf / "traits.toml").read_text())
    all_t, need, found, kept = triples(data)
    problems = [Problem("traits.toml", w) for w in found]
    cases = {}
    for key, c in data.get("case", {}).items():
        tr, ty, m = key.split("/")
        if (tr, ty, m) not in all_t:
            problems.append(Problem(f"traits.toml case {key}", "no such claim or method"))
        else:
            cases[(tr, ty, m)] = c
    problems += [Problem("traits.toml", f"no case: {'/'.join(t)} (write one)") for t in sorted(need - set(cases))]
    problems += [Problem("traits.toml", f"no case: {tr}.{m}'s default on any claimer that keeps it (write one)")
                 for (tr, m), ts in sorted(kept.items()) if not ts & set(cases)]
    frozen = frozen_runs(data, frozenset(ty for _, ty, _ in cases))
    want = {f"{tr}/{ty}--{m}.cp": case_text(c, frozen(tr, ty)) for (tr, ty, m), c in cases.items()}
    return problems + specgate.sync(conf / "traits", want, a.write, "traits", "**/*.cp")

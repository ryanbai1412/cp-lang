#!/usr/bin/env python3
# Enforces handbook/guidelines/conformance-checklist.md#conformancer5-where-a-rule-covers-a-space-generate-the-cases
"""Generate spec/conformance/matrix/ from spec/conformance/matrix.toml.

Each matrix crosses rows (features) with columns (consumers); `optional`
style matrices cross rows with forms and scopes. Every cell becomes one
case, unless the matrix's `na` table gives a reason. `--check` fails when
the generated files are stale or a cell has neither a case nor a reason.
"""
import re
import sys
from pathlib import Path
from string import Template

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance"
OUT = CONF / "matrix"


def parse_toml(text):
    """The subset matrix.toml uses: [a.b.c] tables, "k" or k = "str" | int."""
    data, table = {}, None
    for raw in text.split("\n"):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        m = re.fullmatch(r"\[([\w.]+)\]", line)
        if m:
            table = data
            for part in m.group(1).split("."):
                table = table.setdefault(part, {})
            continue
        m = re.fullmatch(r'("[^"]+"|\w+)\s*=\s*(.+)', line)
        if not m or table is None:
            raise SystemExit(f"matrix.toml: can't parse {raw!r}")
        key, val = m.group(1).strip('"'), m.group(2)
        if re.fullmatch(r"-?\d+", val):
            table[key] = int(val)
        else:
            table[key] = val[1:-1].encode().decode("unicode_escape")
    return data


def indent(code):
    return "\n".join("    " + l if l else l for l in code.split("\n"))


def cells(data):
    for mname, m in data.items():
        na = m.get("na", {})
        rows = m.get("row", {})
        if "col" in m:
            cols = {c: (v["code"], v.get("out", ""), v.get("input")) for c, v in m["col"].items()}
        else:
            cols = {}
            for f, fv in m["form"].items():
                for s, sv in m["scope"].items():
                    cols[f"{f}_{s}"] = (sv["wrap"], "$v", None, fv["body"], sv["leave"])
        for r, rv in rows.items():
            for c, spec in cols.items():
                key = f"{r}--{c}"
                if key in na:
                    yield mname, key, None, na[key]
                    continue
                env = {k: str(v) for k, v in rv.items()}
                if "n" in rv:
                    env["n_less"] = str(rv["n"] - 1)
                    env["ones"] = " ".join(["1"] * rv["n"])
                if len(spec) == 5:
                    wrap, out, inp, body, leave = spec
                    env["leave"] = leave
                    env["body"] = Template(body).substitute(env)
                    env["body_in"] = indent(env["body"])
                    code = wrap
                else:
                    code, out, inp = spec
                try:
                    src = Template(code).substitute(env)
                    want = Template(out).substitute(env)
                    stdin = Template(inp).substitute(env) if inp else None
                except KeyError as e:
                    yield mname, key, None, f"MISSING field {e} in row {r}"
                    continue
                head = [f'// expect: ok, output "{want}"']
                if stdin is not None:
                    head.append(f'// input: "{stdin}"')
                yield mname, key, "\n".join(head) + "\n" + src + "\n", None


def main():
    data = parse_toml((CONF / "matrix.toml").read_text())
    want, problems, na = {}, [], 0
    for mname, key, text, reason in cells(data):
        if text is None:
            if reason.startswith("MISSING"):
                problems.append(f"{mname}: {key}: {reason}")
            else:
                na += 1
            continue
        want[OUT / mname / f"{key}.cp"] = text
    have = {p: p.read_text() for p in OUT.rglob("*.cp")} if OUT.exists() else {}
    if "--check" in sys.argv:
        stale = sorted(str(p.relative_to(ROOT)) for p in set(want) | set(have) if want.get(p) != have.get(p))
        for s in stale[:20]:
            problems.append(f"stale: {s}")
        if problems:
            print("matrix gate failed:\n  " + "\n  ".join(problems))
            print("rerun: python3 scripts/conformance-matrix.py")
            return 1
        return 0
    if problems:
        print("\n".join(problems))
        return 1
    for p in set(have) - set(want):
        p.unlink()
    for p, t in want.items():
        p.parent.mkdir(parents=True, exist_ok=True)
        if have.get(p) != t:
            p.write_text(t)
    print(f"matrix: {len(want)} cases, {na} n/a cells")
    return 0


if __name__ == "__main__":
    sys.exit(main())

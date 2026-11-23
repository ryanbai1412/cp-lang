#!/usr/bin/env python3
"""Derive random programs from the EBNF in spec/core/01-lexical-grammar.md and check each.

The grammar is read from the chapter itself, so a production added there is
generated with no edit here. Every derived program goes through `cpl check
--json`. A derivation is syntax only, so name and type errors are expected;
what is reported:

- crash: `cpl check` died (a panic, a signal, or an exit other than 0 or 1);
- syntax: the parser rejected (E01xx) a text the grammar derives, so the
  parser and the chapter disagree: one of them is wrong;
- unreached: grammar alternatives no accepted program used.

`--check` fails on any crash and when syntax rejections exceed `--max-syntax`
(a shrink-only bound). Rule: conformance/R5,
handbook/guidelines/conformance-checklist.md (generate cross-product cases).

Usage: uv run scripts/grammar-fuzz.py [--n 300] [--seed 1] [--jobs 8]
       [--check] [--max-syntax N] [--show K]
"""
import argparse
import json
import random
import re
import subprocess
import sys
import tempfile
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CHAPTER = ROOT / "spec/core/01-lexical-grammar.md"

# Productions the chapter uses but defines in prose or elsewhere.
EXTRA = r"""
binary      := unary (BINOP2 unary)* | 'not' unary
block       := ':' NL INDENT stmt+ DEDENT | ':' simple_stmts NL
place       := NAME | NAME '[' expr ']' | NAME '.' NAME
path        := NAME | NAME '.' NAME
literal     := INT | STRING | 'true' | 'false' | 'None'
args        := expr (',' expr)*
call        := NAME call_args
tuple       := '(' expr ',' expr ')'
record      := '(' NAME ':' expr (',' NAME ':' expr)* ')'
list        := '[' (expr (',' expr)*)? ']'
map         := '{' expr ':' expr (',' expr ':' expr)* '}'
comprehension := '[' expr 'for' clause ']'
table       := '[' type ']' NAME '=>' expr
block_expr  := expr
fn_sig      := 'fn' fn_name '(' params? ')' ret?
"""

LEAVES = {
    "NAME": ["x", "y", "n", "a", "f", "k"],
    "WORD": ["by", "key", "ok"],
    "DIRECTIVE": ["multitest"],
    "TYPE_NAME": ["Int", "T", "V", "Str"],
    "LABEL": ["'outer"],
    "INT": ["0", "1", "7", "42"],
    "STRING": ['"s"', '"{x}"'],
    "CHAR": ["'a'"],
    "BINOP": ["+", "*", "min"],
    "BINOP2": ["+", "-", "*", "/", "%", "==", "!=", "<", "<=", "and", "or", "..", "..="],
    "BINOP_": ["+"],
    "ASSIGN_OP": ["+=", "-=", "*=", "min="],
}

TOKEN = re.compile(r"'[^']*'|[A-Za-z_][A-Za-z_0-9]*|:=|[()|*+?]")


def grammar_text():
    text = CHAPTER.read_text()
    m = re.search(r"```ebnf\n(.*?)```", text, re.S)
    if not m:
        sys.exit("grammar-fuzz: no ```ebnf block in " + str(CHAPTER))
    return m.group(1)


def parse(text):
    rules, name, body = {}, None, []
    lines = [re.sub(r"\s//.*$", "", l) for l in text.split("\n")]
    for line in lines + ["x := 'end'"]:
        m = re.match(r"^([a-z_]+)\s*:=(.*)$", line)
        if m:
            if name:
                rules[name] = " ".join(body)
            name, body = m.group(1), [m.group(2)]
        elif line.strip():
            body.append(line)
    rules.pop("x", None)
    out = {}
    for n, b in rules.items():
        toks = TOKEN.findall(b)
        try:
            node, rest = alt(toks)
            if rest:
                raise ValueError(rest)
            out[n] = node
        except (ValueError, IndexError):
            pass  # a production written in prose; EXTRA supplies it
    return out


def refs(node):
    if node[0] == "ref":
        return {node[1]}
    if node[0] == "lit":
        return set()
    kids = node[1] if node[0] in ("seq", "alt") else [node[1]]
    return set().union(*(refs(k) for k in kids)) if kids else set()


def alt(t):
    seqs = []
    s, t = seq(t)
    seqs.append(s)
    while t and t[0] == "|":
        s, t = seq(t[1:])
        seqs.append(s)
    return ("alt", seqs), t


def seq(t):
    items = []
    while t and t[0] not in ("|", ")"):
        if t[0] == "(":
            node, t = alt(t[1:])
            if not t or t[0] != ")":
                raise ValueError("unclosed (")
            t = t[1:]
        else:
            node, t = (("lit", t[0][1:-1]) if t[0].startswith("'") else ("ref", t[0])), t[1:]
        while t and t[0] in ("*", "+", "?"):
            node, t = (t[0], node), t[1:]
        items.append(node)
    return ("seq", items), t


class Gen:
    def __init__(self, rules, rng, limit=4):
        self.rules, self.rng, self.limit = rules, rng, limit
        self.used = set()

    def gen(self, node, depth, out, where):
        kind = node[0]
        if kind == "lit":
            out.append(node[1])
        elif kind == "ref":
            n = node[1]
            if n in self.rules:
                self.gen(self.rules[n], depth + 1, out, n)
            elif n in ("NL", "INDENT", "DEDENT"):
                out.append(n)
            else:
                leaf = "DIRECTIVE" if (n, where) == ("NAME", "directive") else n
                out.append(self.rng.choice(LEAVES.get(leaf, ["x"])))
        elif kind == "seq":
            for it in node[1]:
                self.gen(it, depth, out, where)
        elif kind == "alt":
            alts = node[1]
            if depth > self.limit:
                k = min(range(len(alts)), key=lambda i: cost(alts[i], self.rules))
            else:
                k = self.rng.randrange(len(alts))
            if len(alts) > 1:
                self.used.add((where, k))
            self.gen(alts[k], depth, out, where)
        else:
            low = {"?": 0, "*": 0, "+": 1}[kind]
            hi = low if depth > self.limit else low + self.rng.choice([0, 0, 0, 1])
            for _ in range(hi):
                self.gen(node[1], depth, out, where)


def cost(node, rules, seen=()):
    kind = node[0]
    if kind == "lit":
        return 0
    if kind == "ref":
        n = node[1]
        if n not in rules or n in seen:
            return 0 if n not in rules else 50
        return 1 + cost(rules[n], rules, seen + (n,))
    if kind == "seq":
        return sum(cost(i, rules, seen) for i in node[1])
    if kind == "alt":
        return min(cost(a, rules, seen) for a in node[1])
    return 0 if kind in ("?", "*") else cost(node[1], rules, seen)


NO_SPACE_BEFORE = {".", ",", ")", "]", "!", "?", ":", "}"}
NO_SPACE_AFTER = {"(", "[", ".", "@", "@!", "{", "-", "~"}


def render(tokens, keywords):
    lines, cur, indent, prev = [], [], 0, None
    for tok in tokens:
        if tok == "NL":
            if cur:
                lines.append("    " * cur_indent + "".join(cur))
            cur, prev = [], None
            continue
        if tok == "INDENT":
            indent += 1
            continue
        if tok == "DEDENT":
            indent = max(0, indent - 1)
            continue
        if not cur:
            cur_indent = indent
        tight = prev is None or tok in NO_SPACE_BEFORE or prev in NO_SPACE_AFTER or (
            tok in ("(", "[") and prev not in keywords and (prev[-1].isalnum() or prev in (")", "]"))
        )
        cur.append(tok if tight else " " + tok)
        prev = tok
    if cur:
        lines.append("    " * cur_indent + "".join(cur))
    return "\n".join(lines) + "\n"


def check(cpl, src):
    with tempfile.NamedTemporaryFile("w", suffix=".cp", delete=False) as f:
        f.write(src)
    try:
        r = subprocess.run([cpl, "check", "--json", f.name], capture_output=True, text=True, timeout=30)
    except subprocess.TimeoutExpired:
        return "crash", "timed out after 30s"
    finally:
        Path(f.name).unlink()
    if r.returncode not in (0, 1) or "panicked" in r.stderr:
        return "crash", (r.stderr.strip().splitlines() or [f"exit {r.returncode}"])[-1]
    try:
        diags = [d for fl in json.loads(r.stdout)["files"] for d in fl["diagnostics"]]
    except (ValueError, KeyError):
        return "crash", "unreadable --json output"
    syn = [d for d in diags if str(d.get("code", "")).startswith("E01")]
    if syn:
        return "syntax", f"{syn[0]['code']}: {syn[0]['message']}"
    return "ok", ""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=300)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--max-syntax", type=int, default=None)
    ap.add_argument("--show", type=int, default=5)
    ap.add_argument("--cpl", default=str(ROOT / "target/debug/cpl"))
    a = ap.parse_args()
    rules = parse(grammar_text())
    extra = parse(EXTRA)
    known = set(rules) | set(extra) | set(LEAVES) | {"NL", "INDENT", "DEDENT"}
    for n in [n for n, r in rules.items() if refs(r) - known]:
        del rules[n]  # written in prose; EXTRA supplies it
    rules.update({k: v for k, v in extra.items() if k not in rules})
    keywords = {t for t in re.findall(r"'([a-z]+)'", grammar_text() + EXTRA)}
    alts = {(n, k) for n, r in rules.items() if r[0] == "alt" and len(r[1]) > 1 for k in range(len(r[1]))}

    progs = []
    for i in range(a.n):
        g = Gen(rules, random.Random(a.seed * 100003 + i))
        toks = []
        if g.rng.random() < 0.2:
            g.gen(("ref", "directive"), 0, toks, "file")
        for _ in range(g.rng.randint(1, 4)):
            g.gen(("ref", "item"), 0, toks, "file")
        progs.append((i, render(toks, keywords), g.used))
    with ThreadPoolExecutor(a.jobs) as ex:
        results = list(ex.map(lambda p: check(a.cpl, p[1]), progs))

    kinds = Counter(k for k, _ in results)
    used_ok = set().union(*(p[2] for p, (k, _) in zip(progs, results) if k == "ok")) if progs else set()
    unreached = sorted(alts - used_ok)
    causes = Counter(d.split(":")[0] + ": " + re.sub(r"`[^`]*`", "`_`", d.split(": ", 1)[-1])[:90]
                     for k, d in results if k == "syntax")
    print(f"grammar-fuzz: {a.n} programs from {len(rules)} productions: "
          f"{kinds['ok']} parse, {kinds['syntax']} syntax-rejected, {kinds['crash']} crashed; "
          f"{len(alts) - len(unreached)} of {len(alts)} alternatives reached by a parsing program")
    for (i, src, _), (k, d) in zip(progs, results):
        if k == "crash":
            print(f"\ncrash (seed {a.seed}, program {i}): {d}\n{src}")
    for cause, n in causes.most_common(a.show):
        print(f"  {n:4} {cause}")
    seen = set()
    for (i, src, _), (k, d) in zip(progs, results):
        c = d.split(":")[0] + ": " + re.sub(r"`[^`]*`", "`_`", d.split(": ", 1)[-1])[:90]
        if k == "syntax" and c not in seen and len(seen) < a.show:
            seen.add(c)
            print(f"\n-- program {i}: {d}\n{src}")
    if unreached:
        print("unreached: " + ", ".join(f"{n}#{k + 1}" for n, k in unreached[: 40]))
    if a.check:
        bad = kinds["crash"] > 0 or (a.max_syntax is not None and kinds["syntax"] > a.max_syntax)
        sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""gen.py seed size | --max: NOI Online 2022 P1 (macro expansion). n <= 100 lines of at most 100
printable chars, #define / #undef directives, every expanded line at most 1000 chars. --max:
n = 100 full-width lines, chains of macros expanding through each other (and back into active
names), each kept only while its expansion fits 1000 chars."""
import random, string, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 100 if MAX else r.randint(1, max(1, min(100, int(sys.argv[2]))))
W = 100 if MAX else r.randint(10, 100)
names = [f"M{i}" for i in range(12)] + ["a", "b_1", "X"]
macros = {}

def is_ident(c):
    return c.isascii() and (c.isalnum() or c == "_")

def expand(line, active):
    out, i = [], 0
    while i < len(line):
        j = i
        while j < len(line) and not is_ident(line[j]): j += 1
        out.append(line[i:j]); i = j
        if i == len(line): break
        while j < len(line) and is_ident(line[j]): j += 1
        tok = line[i:j]; i = j
        if tok not in active and tok in macros:
            out.append(expand(macros[tok], active | {tok}))
        else:
            out.append(tok)
    return "".join(out)

def words(width):
    parts, size = [], 0
    while True:
        w = r.choice(names + ["x", "+", "(", ")", "1", " ", "*", "y2"])
        if size + len(w) + 1 > width: break
        parts.append(w); size += len(w) + 1
    return " ".join(parts)

lines = []
while len(lines) < n:
    k = r.random()
    if k < 0.35:
        name = r.choice(names)
        line = f"#define {name} {words(W - 9 - len(name))}"[:W].rstrip()
        old = macros.get(name)
        macros[name] = line.split(" ", 2)[2] if line.count(" ") >= 2 else ""
        if len(expand(macros[name], set())) > 1000:  # keep every expansion small enough
            if old is None: del macros[name]
            else: macros[name] = old
            continue
    elif k < 0.42 and macros:
        line = f"#undef {r.choice(sorted(macros))}"
        del macros[line[7:]]
    else:
        line = words(W) or "x"
        if len(expand(line, set())) > 1000:
            continue
    lines.append(line)
print(n)
print("\n".join(lines))

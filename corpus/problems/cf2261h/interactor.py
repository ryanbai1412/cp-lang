#!/usr/bin/env python3
# interactor: spawn
"""Run-twice judge for CF 2261 H: `interactor.py CASE -- SOLUTION...`.

CASE is a first-run input. The solution runs as Isabel; in each tree Lord Verity
flips one node (seeded from CASE, spread over every label across cases) and
swaps its child pointers; the cases are shuffled, and the solution runs again
as Tara, who must print each original string."""
import random
import subprocess
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def run(cmd, text):
    p = subprocess.run(cmd, input=text, capture_output=True, text=True, timeout=20)
    if p.returncode != 0:
        fail(f"solution exited {p.returncode}: {p.stderr[-500:]}")
    return p.stdout.split()


def verity(h, tree, p):
    """Flip node p (1-based heap label), swap its children, relabel."""
    size = (1 << h) - 1
    s = list(tree)
    s[p - 1] = "1" if s[p - 1] == "0" else "0"
    a, b, w = 2 * p, 2 * p + 1, 1
    while a <= size:
        s[a - 1:a - 1 + w], s[b - 1:b - 1 + w] = s[b - 1:b - 1 + w], s[a - 1:a - 1 + w]
        a, b, w = 2 * a, 2 * b, 2 * w
    return "".join(s)


def main():
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        sys.stderr.write("usage: interactor.py CASE -- SOLUTION...\n")
        sys.exit(2)
    text = open(sys.argv[1], encoding="utf-8").read()
    toks = text.split()
    cmd = sys.argv[3:]
    if toks[0] != "first":
        fail("CASE must be a first-run input")
    t = int(toks[1])
    cases = [(int(toks[2 + 3 * i]), int(toks[3 + 3 * i]), toks[4 + 3 * i]) for i in range(t)]
    out = run(cmd, text)
    if len(out) != t:
        fail(f"first run: expected {t} trees, got {len(out)}")
    rng = random.Random(text)
    start = rng.randrange(1 << 11)
    sent = []
    for i, ((n, h, b), tree) in enumerate(zip(cases, out)):
        size = (1 << h) - 1
        if len(tree) != size or set(tree) - {"0", "1"}:
            fail(f"case {i + 1}: expected a binary string of length {size}")
        p = (start + i) % size + 1
        sent.append((n, h, b, verity(h, tree, p), p))
    order = list(range(t))
    rng.shuffle(order)
    second = ["second", str(t)] + [f"{sent[i][0]} {sent[i][1]}\n{sent[i][3]}" for i in order]
    ans = run(cmd, "\n".join(second) + "\n")
    if len(ans) != t:
        fail(f"second run: expected {t} strings, got {len(ans)}")
    for k, i in enumerate(order):
        n, h, b, _, p = sent[i]
        if ans[k] != b:
            fail(f"case {i + 1} (node {p} flipped): got {ans[k][:40]}..., want {b[:40]}...")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
# interactor: spawn
"""Run-twice judge for CF 2237 G: `interactor.py CASE -- SOLUTION...`.

CASE is a first-run input. The solution runs as Ja and prints each b; the cases
are shuffled and it runs again as Quack, answering `? i j` with gcd(b_i, b_j)
and checking each `! a_1 ... a_n` and the k and query limits."""
import random
import subprocess
import sys
from math import gcd


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        sys.stderr.write("usage: interactor.py CASE -- SOLUTION...\n")
        sys.exit(2)
    text = open(sys.argv[1], encoding="utf-8").read()
    toks = text.split()
    cmd = sys.argv[3:]
    if toks[0] != "first":
        fail("CASE must be a first-run input")
    t, pos, cases = int(toks[1]), 2, []
    for _ in range(t):
        n = int(toks[pos])
        cases.append([int(x) for x in toks[pos + 1:pos + 1 + n]])
        pos += 1 + n
    p = subprocess.run(cmd, input=text, capture_output=True, text=True, timeout=20)
    if p.returncode != 0:
        fail(f"first run exited {p.returncode}: {p.stderr[-500:]}")
    out, pos, bs = p.stdout.split(), 0, []
    for c, a in enumerate(cases):
        if pos >= len(out):
            fail(f"case {c + 1}: first run printed no b")
        k = int(out[pos])
        limit = -(-10 * len(a) // 9) + 150
        if not 1 <= k <= limit:
            fail(f"case {c + 1}: k = {k}, want 1..={limit}")
        b = [int(x) for x in out[pos + 1:pos + 1 + k]]
        if len(b) != k or not all(1 <= x <= 10**6 for x in b):
            fail(f"case {c + 1}: b must be {k} integers in 1..=1e6")
        bs.append(b)
        pos += 1 + k
    if pos != len(out):
        fail("first run printed extra output")
    order = list(range(t))
    random.Random(text).shuffle(order)
    proc = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)

    def send(line):
        proc.stdin.write(line + "\n")
        proc.stdin.flush()

    def recv():
        line = proc.stdout.readline()
        if not line:
            fail("second run: unexpected EOF")
        return line.split()

    send("second")
    send(str(t))
    for c in order:
        a, b = cases[c], bs[c]
        n, k = len(a), len(b)
        send(f"{n} {k}")
        queries = 0
        while True:
            w = recv()
            if w and w[0] == "?":
                queries += 1
                i, j = int(w[1]), int(w[2])
                if queries > 180 * n + 150 or not (1 <= i <= k and 1 <= j <= k and i != j):
                    send("-1")
                    fail(f"case {c + 1}: bad query {' '.join(w)} (query {queries})")
                send(str(gcd(b[i - 1], b[j - 1])))
            elif w and w[0] == "!":
                got = [int(x) for x in w[1:]]
                if got != a:
                    fail(f"case {c + 1}: Quack says {got[:10]}..., a is {a[:10]}...")
                break
            else:
                fail(f"case {c + 1}: unexpected line {' '.join(w)[:80]}")
    proc.stdin.close()
    if proc.wait(timeout=20) != 0:
        fail(f"second run exited {proc.returncode}")


if __name__ == "__main__":
    main()

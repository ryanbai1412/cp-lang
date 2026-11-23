#!/usr/bin/env python3
"""Judge for CF 2207 H1: `interactor.py CASE -- SOLUTION...`.

CASE is in the hack format: t, then per test n and a min-max expression over
1..n. The solution's `? x_1 ... x_n` queries (at most 10^4 in total) get f(x);
after its `!` it must evaluate Bowser's own inputs online (at most 5000 in
total, mixing permutations, ties and random values), then reads `0`."""
import random
import subprocess
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def parse(toks, pos):
    if toks[pos] in ("min", "max"):
        op, pos, args = min if toks[pos] == "min" else max, pos + 2, []
        while True:
            node, pos = parse(toks, pos)
            args.append(node)
            pos += 1
            if toks[pos - 1] == ")":
                return (op, args), pos
    return int(toks[pos]) - 1, pos + 1


def value(node, x):
    if isinstance(node, int):
        return x[node]
    op, args = node
    return op(value(a, x) for a in args)


def main():
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        sys.stderr.write("usage: interactor.py CASE -- SOLUTION...\n")
        sys.exit(2)
    text = open(sys.argv[1], encoding="utf-8").read()
    lines = [l.split() for l in text.splitlines() if l.strip()]
    t, tests = int(lines[0][0]), []
    for c in range(t):
        n = int(lines[1 + 2 * c][0])
        f, _ = parse(lines[2 + 2 * c], 0)
        tests.append((n, f))
    rng = random.Random(text)
    proc = subprocess.Popen(sys.argv[3:], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)

    def send(line):
        proc.stdin.write(line + "\n")
        proc.stdin.flush()

    def recv():
        line = proc.stdout.readline()
        if not line:
            fail("unexpected EOF from solution")
        return line.split()

    asked, per_test = 0, 5000 // t
    send(str(t))
    for c, (n, f) in enumerate(tests):
        send(str(n))
        while True:
            w = recv()
            if w == ["!"]:
                break
            if not w or w[0] != "?" or len(w) != n + 1:
                fail(f"case {c + 1}: bad line {' '.join(w)[:80]}")
            x = [int(v) for v in w[1:]]
            asked += 1
            if asked > 10**4 or not all(1 <= v <= 10**9 for v in x):
                send("-1")
                fail(f"case {c + 1}: query {asked} out of limits")
            send(str(value(f, x)))
        for q in range(min(per_test, 300)):
            kind = q % 3
            if kind == 0:
                x = rng.sample(range(1, n + 1), n)
            elif kind == 1:
                x = [rng.randint(1, 3) for _ in range(n)]
            else:
                x = [rng.randint(1, 10**9) for _ in range(n)]
            send(" ".join(map(str, x)))
            w = recv()
            if w != [str(value(f, x))]:
                fail(f"case {c + 1}: f({' '.join(map(str, x[:8]))}...) = {value(f, x)}, solution says {' '.join(w)[:40]}")
        send("0")
    proc.stdin.close()
    if proc.wait(timeout=20) != 0:
        fail(f"solution exited {proc.returncode}")


if __name__ == "__main__":
    main()

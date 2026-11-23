#!/usr/bin/env python3
"""Run-twice judge for CF 2255 C: `interactor.py CASE -- SOLUTION...`.

CASE is a first-run input. The solution runs as Chtholly, its swaps are applied,
each picture gets a random mix of cyclic shifts, clockwise rotations, vertical
reflections and colour inversions (the target moves with it), the cases are
shuffled, and the solution runs again as Willem, who must name the moved target."""
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


def main():
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        sys.stderr.write("usage: interactor.py CASE -- SOLUTION...\n")
        sys.exit(2)
    toks = open(sys.argv[1], encoding="utf-8").read().split()
    cmd = sys.argv[3:]
    if toks[0] != "first":
        fail("CASE must be a first-run input")
    t = int(toks[1])
    pos, cases = 2, []
    for _ in range(t):
        n = int(toks[pos])
        g = [list(row) for row in toks[pos + 1:pos + 1 + n]]
        r, c = int(toks[pos + 1 + n]) - 1, int(toks[pos + 2 + n]) - 1
        cases.append((n, g, r, c))
        pos += 3 + n
    out = [int(x) for x in run(cmd, " ".join(toks) + "\n")]
    if len(out) != 4 * t:
        fail(f"first run printed {len(out)} numbers, want {4 * t}")
    rng = random.Random(" ".join(toks))
    moved = []
    for k, (n, g, r, c) in enumerate(cases):
        r1, c1, r2, c2 = (x - 1 for x in out[4 * k:4 * k + 4])
        if not all(0 <= x < n for x in (r1, c1, r2, c2)):
            fail(f"case {k + 1}: swap cell outside the picture")
        g[r1][c1], g[r2][c2] = g[r2][c2], g[r1][c1]
        for _ in range(rng.randint(0, 6)):
            op = rng.randrange(4)
            if op == 0:
                dr, dc = rng.randrange(n), rng.randrange(n)
                h = [[None] * n for _ in range(n)]
                for i in range(n):
                    for j in range(n):
                        h[(i + dr) % n][(j + dc) % n] = g[i][j]
                g, r, c = h, (r + dr) % n, (c + dc) % n
            elif op == 1:
                h = [[None] * n for _ in range(n)]
                for i in range(n):
                    for j in range(n):
                        h[j][n - 1 - i] = g[i][j]
                g, r, c = h, c, n - 1 - r
            elif op == 2:
                g, c = [row[::-1] for row in g], n - 1 - c
            else:
                g = [["." if x == "#" else "#" for x in row] for row in g]
        moved.append((n, g, r, c))
    order = list(range(t))
    rng.shuffle(order)
    text = ["second", str(t)]
    for k in order:
        n, g, _, _ = moved[k]
        text += [str(n)] + ["".join(row) for row in g]
    ans = [int(x) for x in run(cmd, "\n".join(text) + "\n")]
    if len(ans) != 2 * t:
        fail(f"second run printed {len(ans)} numbers, want {2 * t}")
    for idx, k in enumerate(order):
        n, _, r, c = moved[k]
        if (ans[2 * idx] - 1, ans[2 * idx + 1] - 1) != (r, c):
            fail(f"case {k + 1}: Willem says ({ans[2 * idx]}, {ans[2 * idx + 1]}), target is ({r + 1}, {c + 1})")


if __name__ == "__main__":
    main()

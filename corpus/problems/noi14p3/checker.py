#!/usr/bin/env python3
"""Checker for NOI 2014 P3 (Deletion Game): at most K valid moves (a simple path of l_min..l_max
non-blank cells, the first not 0), replayed with the falling rule; the final score must be at least
the reference's."""
import sys

BASES = (2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41)


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def is_prime(n):
    if n < 2:
        return False
    for p in BASES:
        if n % p == 0:
            return n == p
    d, s = n - 1, 0
    while d % 2 == 0:
        d, s = d // 2, s + 1
    for a in BASES:
        x = pow(a, d, n)
        if x in (1, n - 1):
            continue
        for _ in range(s - 1):
            x = x * x % n
            if x == n - 1:
                break
        else:
            return False
    return True


def final_score(inp, toks, who):
    _, n, m, K, lmin, lmax, c1, c2, F = inp[:9]
    A = [inp[9 + i * m: 9 + (i + 1) * m] for i in range(n)]
    pos = 0

    def take():
        nonlocal pos
        if pos >= len(toks):
            fail(f"{who}: output ends early")
        pos += 1
        try:
            return int(toks[pos - 1])
        except ValueError:
            fail(f"{who}: {toks[pos - 1]!r} is not an integer")

    M = take()
    if not 0 <= M <= K:
        fail(f"{who}: {M} moves, want 0..{K}")
    total = 0
    for k in range(M):
        l = take()
        if not lmin <= l <= lmax:
            fail(f"{who}: move {k + 1} has length {l}, want {lmin}..{lmax}")
        cells = [(take() - 1, take() - 1) for _ in range(l)]
        if len(set(cells)) != l:
            fail(f"{who}: move {k + 1} repeats a cell")
        for i, (x, y) in enumerate(cells):
            if not (0 <= x < n and 0 <= y < m) or A[x][y] < 0:
                fail(f"{who}: move {k + 1} uses a blank or outside cell")
            if i and abs(x - cells[i - 1][0]) + abs(y - cells[i - 1][1]) != 1:
                fail(f"{who}: move {k + 1} steps between cells that are not neighbours")
        digits = "".join(str(A[x][y]) for x, y in cells)
        if digits[0] == "0":
            fail(f"{who}: move {k + 1} starts with 0")
        a = l ** c1 if is_prime(int(digits)) else 1
        b = l ** c2 if digits == digits[::-1] else 1
        if a == 1 and b == 1:
            continue
        total += a + b
        for x, y in cells:
            A[x][y] = -1
        for y in {y for _, y in cells}:
            col = [A[x][y] for x in range(n) if A[x][y] >= 0]
            col = [-1] * (n - len(col)) + col
            for x in range(n):
                A[x][y] = col[x]
    if pos != len(toks):
        fail(f"{who}: extra output after move {M}")
    if F:
        total >>= sum(v >= 0 for row in A for v in row)
    return total


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    got = final_score(inp, open(sys.argv[2]).read().split(), "output")
    want = final_score(inp, open(sys.argv[3]).read().split(), "answer")
    if got < want:
        fail(f"final score {got}, the reference reaches {want}")


if __name__ == "__main__":
    main()

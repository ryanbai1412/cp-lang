# IOI 2025 World Map checker (problem-packages/R3): any valid map is accepted.
# python3 checker.py <input> <output> <answer>; exit 0 accepts. Each scenario's map must be
# a K x K grid with 1 <= K <= 2N (full score; the task's hard cap is 240), every colour in
# 1..=N present, every listed pair side-adjacent somewhere and no unlisted pair adjacent.
# Mirrors the task archive's checker/checker.cpp, which also scores K / N.
import sys


def fail(msg):
    print(msg, file=sys.stderr)
    sys.exit(1)


def main(inp, out):
    it = iter(open(inp).read().split())
    got = open(out).read().split()
    pos = 0

    def take():
        nonlocal pos
        if pos >= len(got):
            fail("output ends early")
        tok = got[pos]
        pos += 1
        try:
            return int(tok)
        except ValueError:
            fail(f"not an integer: {tok!r}")

    for t in range(int(next(it))):
        n, m = int(next(it)), int(next(it))
        edges = set()
        for _ in range(m):
            a, b = int(next(it)), int(next(it))
            edges.add((min(a, b), max(a, b)))
        k = take()
        if not 1 <= k <= min(2 * n, 240):
            fail(f"scenario {t}: K = {k}, want 1 <= K <= 2N = {2 * n}")
        for i in range(k):
            if take() != k:
                fail(f"scenario {t}: row {i} is not of length K")
        c = [[take() for _ in range(k)] for _ in range(k)]
        if any(not 1 <= x <= n for row in c for x in row):
            fail(f"scenario {t}: a colour outside 1..={n}")
        if len({x for row in c for x in row}) != n:
            fail(f"scenario {t}: some country has no cell")
        seen = set()
        for i in range(k):
            for j in range(k):
                for y, x in ((i + 1, j), (i, j + 1)):
                    if y < k and x < k and c[i][j] != c[y][x]:
                        p = (min(c[i][j], c[y][x]), max(c[i][j], c[y][x]))
                        if p not in edges:
                            fail(f"scenario {t}: countries {p} touch but are not adjacent")
                        seen.add(p)
        if seen != edges:
            fail(f"scenario {t}: pairs {sorted(edges - seen)[:3]} never touch")
    if pos != len(got):
        fail("extra output")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])

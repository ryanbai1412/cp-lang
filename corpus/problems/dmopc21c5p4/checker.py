#!/usr/bin/env python3
"""Checker for DMOPC '21 C5 P4: per test, -1 exactly when the answer is -1; otherwise a
permutation of 1..N with prime gaps and as few distinct signed differences as the answer."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    ns = list(map(int, open(sys.argv[1]).read().split()))[1:]
    out = [l.split() for l in open(sys.argv[2]).read().splitlines() if l.strip()]
    ans = [l.split() for l in open(sys.argv[3]).read().splitlines() if l.strip()]
    if len(out) != len(ns):
        fail(f"{len(out)} lines for {len(ns)} tests")
    top = max(ns) + 1
    composite = bytearray(top + 1)
    composite[0] = composite[1] = 1
    for p in range(2, int(top ** 0.5) + 1):
        if not composite[p]:
            composite[p * p::p] = b"\x01" * len(range(p * p, top + 1, p))
    for t, (n, o, a) in enumerate(zip(ns, out, ans)):
        if a == ["-1"] or o == ["-1"]:
            if a != o:
                fail(f"test {t}: got {o[:3]}, answer {a[:3]}")
            continue
        p = list(map(int, o))
        if sorted(p) != list(range(1, n + 1)):
            fail(f"test {t}: not a permutation of 1..{n}")
        diffs = [y - x for x, y in zip(p, p[1:])]
        if any(composite[abs(d)] for d in diffs):
            fail(f"test {t}: a gap is not prime")
        q = list(map(int, a))
        want = len(set(y - x for x, y in zip(q, q[1:])))
        if len(set(diffs)) > want:
            fail(f"test {t}: {len(set(diffs))} distinct differences, answer has {want}")


main()

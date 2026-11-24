#!/usr/bin/env python3
"""Checker for YAC 6 P2: -1 exactly when the roots differ; otherwise as many moves as workers
whose boss differs, and replaying the moves (A re-hangs under B, B not in A's subtree) turns
the first tree into the second."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n = inp[0]
    have = [0] + inp[1:1 + n]
    want = [0] + inp[1 + n:1 + 2 * n]
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if ref[0] == "-1" or out[:1] == ["-1"]:
        if out[:1] != ref[:1] or len(out) != 1:
            fail(f"got {' '.join(out[:1])}, want {ref[0]}")
        return
    need = sum(have[v] != want[v] for v in range(1, n + 1))
    if out[0] != str(need) or len(out) != 1 + 2 * need:
        fail(f"expected {need} moves")
    cur = have[:]
    root = have.index(-1)
    # final[v]: v's boss and every boss above it already match the second tree.
    final = [False] * (n + 1)
    final[root] = True
    for i in range(need):
        a, b = int(out[1 + 2 * i]), int(out[2 + 2 * i])
        if not (1 <= a <= n and 1 <= b <= n) or a == b or a == root:
            fail(f"bad move {a} {b}")
        path, v = [], b
        while not final[v] and cur[v] == want[v]:
            path.append(v)
            v = cur[v]
        if final[v]:
            for u in path:
                final[u] = True
        else:
            v = b
            while v != -1:
                if v == a:
                    fail(f"move {a} {b}: {a} manages {b}")
                v = cur[v]
        cur[a] = b
    if cur[1:] != want[1:]:
        fail("the moves do not give the second tree")


if __name__ == "__main__":
    main()

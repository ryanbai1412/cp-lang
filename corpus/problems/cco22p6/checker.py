#!/usr/bin/env python3
"""Checker for CCO 2022 Good Game: the output must be -1 exactly when the reference's is,
else K moves "i j" with j in {2, 3}, each removing j blocks of one label at current
indices i..i+j-1, that clear the board."""
import sys

B = 1024


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    with open(sys.argv[1], encoding="utf-8") as f:
        inp = f.read().split()
    with open(sys.argv[2], encoding="utf-8") as f:
        out = f.read().split()
    with open(sys.argv[3], encoding="utf-8") as f:
        ref = f.read().split()
    n, s = int(inp[0]), inp[1]
    if not out:
        fail("empty output")
    want_win = ref[0] != "-1"
    try:
        nums = [int(t) for t in out]
    except ValueError:
        fail("output is not all integers")
    if nums[0] == -1:
        if len(nums) != 1:
            fail("tokens after -1")
        if want_win:
            fail("answered -1, but the board can be cleared")
        return
    k = nums[0]
    if not want_win:
        fail("printed moves, but the board cannot be cleared")
    if k < 0 or len(nums) != 1 + 2 * k:
        fail(f"K = {k} but {len(nums) - 1} numbers follow")
    # blocks of alive indices; Fenwick tree over block sizes
    blocks = [list(range(i, min(n, i + B))) for i in range(0, n, B)]
    nb = len(blocks)
    fen = [0] * (nb + 1)
    for b in range(nb):
        i = b + 1
        while i <= nb:
            fen[i] += len(blocks[b])
            i += i & -i
    top = 1
    while top * 2 <= nb:
        top *= 2
    alive = n
    for t in range(k):
        i, j = nums[1 + 2 * t], nums[2 + 2 * t]
        if j not in (2, 3):
            fail(f"move {t + 1}: removes {j} blocks")
        if i < 1 or i + j - 1 > alive:
            fail(f"move {t + 1}: [{i}, {i + j - 1}] is outside the {alive} blocks left")
        label = None
        for _ in range(j):
            rank, pos, step = i, 0, top  # find block holding the rank-th alive index
            while step:
                q = pos + step
                if q <= nb and fen[q] < rank:
                    pos = q
                    rank -= fen[q]
                step >>= 1
            c = s[blocks[pos].pop(rank - 1)]
            if label is None:
                label = c
            elif c != label:
                fail(f"move {t + 1}: blocks {i}..{i + j - 1} do not share a label")
            q = pos + 1
            while q <= nb:
                fen[q] -= 1
                q += q & -q
        alive -= j
    if alive:
        fail(f"{alive} blocks are left")


main()

#!/usr/bin/env python3
"""Checker for NOI 2002 P5 (New Tetris): drops (T, C) that never leave the grid or float a square,
clear every full row and end with every column empty, in at most 100000 commands."""
import sys

# Tile T's columns, left to right: (height of its lowest square above the tile's base, squares added).
TILES = [[(0, 4)], [(0, 1)] * 4, [(0, 2)] * 2, [(1, 1), (0, 2), (0, 1)], [(0, 1), (0, 2), (1, 1)],
         [(1, 2), (0, 2)], [(0, 2), (1, 2)], [(0, 1), (0, 2), (0, 1)], [(1, 1), (0, 2), (1, 1)],
         [(1, 1), (0, 3)], [(0, 3), (1, 1)], [(0, 2), (1, 1), (1, 1)], [(1, 1), (1, 1), (0, 2)],
         [(0, 2), (0, 1), (0, 1)], [(0, 1), (0, 1), (0, 2)], [(0, 3), (2, 1)], [(2, 1), (0, 3)],
         [(0, 3), (0, 1)], [(0, 1), (0, 3)]]
LIMIT = 100000


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, h = inp[0], inp[1:1 + inp[0]]
    out = open(sys.argv[2]).read().split()
    if len(out) % 2 or not all(t.isdigit() for t in out):
        fail("want pairs of positive integers T C")
    cmds = list(map(int, out))
    if len(cmds) // 2 > LIMIT:
        fail(f"{len(cmds) // 2} commands, more than {LIMIT}")
    for k in range(0, len(cmds), 2):
        t, c = cmds[k], cmds[k + 1]
        if not any(h):
            fail(f"command {k // 2 + 1} after the game ended")
        if not 1 <= t <= 19:
            fail(f"command {k // 2 + 1}: no tile {t}")
        tile = TILES[t - 1]
        if not 1 <= c <= n - len(tile) + 1:
            fail(f"command {k // 2 + 1}: tile {t} at column {c} leaves the grid")
        cols = range(c - 1, c - 1 + len(tile))
        if len({h[i] - tile[j][0] for j, i in enumerate(cols)}) != 1:
            fail(f"command {k // 2 + 1}: tile {t} at column {c} leaves a floating square")
        for j, i in enumerate(cols):
            h[i] += tile[j][1]
        full = min(h)
        h = [x - full for x in h]
    if any(h):
        fail("the grid is not empty at the end")


if __name__ == "__main__":
    main()

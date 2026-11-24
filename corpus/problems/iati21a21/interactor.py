#!/usr/bin/env python3
# Judge for IATI 2021 Hint, stdio-adapted. Round 1 (genHint) sends N M, A, B, K and an optimal
# C; the program prints the hint's length L and its L bits as one 0/1 line. Round 2 (solve)
# sends N M, A, B, L and the hint; the program prints a length and a sequence, which must be a
# common subsequence of A and B of length K. The official grader runs the rounds in separate
# processes; this judge runs one process, so it cannot stop a program keeping state across them.
# The statement scores by the longest hint (min((640 / (L + 1))^0.3, 1)), logged here.
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def tokens(io, k):
    out = []
    while len(out) < k:
        out += io.recv_line().split()
    if len(out) != k:
        raise RuntimeError(f"expected {k} tokens, got {len(out)}")
    return out


def drive(case_text, io):
    lines = case_text.split("\n")
    n, m = map(int, lines[0].split())
    a, b = lines[1].split(), lines[2].split()
    k = int(lines[3])
    if len(a) != n or len(b) != m:
        raise RuntimeError("bad case")
    io.send("\n".join(lines[:5]))
    (l,) = map(int, tokens(io, 1))
    if not 0 <= l <= 10_000_000:
        raise RuntimeError(f"hint length {l}")
    hint = io.recv_line().strip() if l else ""
    if len(hint) != l or set(hint) - {"0", "1"}:
        raise RuntimeError("the hint is not L bits")
    io.send(f"{lines[0]}\n{lines[1]}\n{lines[2]}\n{l}\n{hint}")
    (got,) = map(int, tokens(io, 1))
    if got != k:
        raise RuntimeError(f"length {got}, optimal {k}")
    c = tokens(io, k) if k else []
    i = j = 0
    for x in c:
        while i < n and a[i] != x:
            i += 1
        while j < m and b[j] != x:
            j += 1
        if i == n or j == m:
            raise RuntimeError("not a common subsequence")
        i, j = i + 1, j + 1
    sys.stderr.write(f"hint bits: {l}\n")


run_interactor(drive)

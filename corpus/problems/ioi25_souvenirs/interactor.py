#!/usr/bin/env python3
# Judge for IOI 2025 Souvenirs, stdio-adapted from the official manager (graders/manager.cpp).
# Sends "N P[0]". Each "? M" is one transaction(M): M must satisfy P[N-1] <= M < P[0], at most
# 5000 of them; it buys one of each type from 0 up while M still covers it and is answered
# "k L[0] .. L[k-1] R", the k types bought in increasing order and the change R. "!" ends the
# run, accepted iff exactly i souvenirs of type i were bought, for every i. The case is N then
# the hidden P[0] > ... > P[N-1] (1 <= P[N-1], P[0] <= 1e15, 2 <= N <= 100).
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT = 5000


def drive(case_text, io):
    n, *p = map(int, case_text.split())
    if not 2 <= n <= 100 or len(p) != n or p[0] > 10**15 or p[-1] < 1 \
            or any(a <= b for a, b in zip(p, p[1:])):
        raise RuntimeError("invalid case")
    io.send(f"{n} {p[0]}")
    bought = [0] * n
    calls = 0
    while True:
        parts = io.recv_line().split()
        if parts == ["!"]:
            break
        if len(parts) != 2 or parts[0] != "?" or not parts[1].lstrip("-").isdigit():
            raise RuntimeError(f"expected `? M` or `!`, got {' '.join(parts)!r}")
        m = int(parts[1])
        calls += 1
        if calls > LIMIT:
            raise RuntimeError(f"too many calls: over {LIMIT} transactions")
        if m >= p[0]:
            raise RuntimeError(f"invalid argument: M ({m}) >= P[0] ({p[0]})")
        if m < p[-1]:
            raise RuntimeError(f"invalid argument: M ({m}) < P[N-1] ({p[-1]})")
        got, r = [], m
        for i, price in enumerate(p):
            if r >= price:
                r -= price
                bought[i] += 1
                got.append(i)
        io.send(" ".join(map(str, [len(got), *got, r])))
    for i, b in enumerate(bought):
        if b != i:
            raise RuntimeError(f"item {i} is bought {b} times, expected {i}")
    sys.stderr.write(f"transactions {calls}\n")


if __name__ == "__main__":
    run_interactor(drive)

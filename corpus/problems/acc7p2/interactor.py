#!/usr/bin/env python3
# Judge for ACC 7 P2 Code Review. The case is N, then t_p, the timestamp of person p's code.
# The program prints a permutation a (person a_i reviews timestamp i) and reads 0 when nobody
# reviews their own code, else one timestamp T with t[a_T] = T, chosen by a generator seeded
# from the case (not adaptive). Twenty wrong proposals, or a bad line, get -1 and fail.
import os, random, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    head, line = case_text.strip().split("\n")
    n = int(head)
    t = [0] + list(map(int, line.split()))
    pick = random.Random(case_text)
    io.send(str(n))
    for k in range(1, 21):
        try:
            a = [0] + list(map(int, io.recv_line().split()))
        except ValueError:
            a = []
        if len(a) != n + 1 or sorted(a[1:]) != list(range(1, n + 1)):
            io.send("-1")
            raise RuntimeError(f"proposal {k} is not a permutation of 1..{n}")
        own = [i for i in range(1, n + 1) if t[a[i]] == i]
        if not own:
            io.send("0")
            return
        if k == 20:
            io.send("-1")
            raise RuntimeError("20 proposals and every one has someone reviewing their own code")
        io.send(str(pick.choice(own)))


if __name__ == "__main__":
    run_interactor(drive)

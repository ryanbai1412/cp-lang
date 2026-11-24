#!/usr/bin/env python3
# Judge for IOI 2014 Game, stdio-adapted: the program reads n and prints READY; then each
# pair "u v" is sent and answered 1 (a flight) or 0. Jian-Jia wins when connectivity
# stays undetermined until the last answer: just before it, the yes-flights leave exactly
# two components and the last pair joins them. The case is n then the pairs (gen.py).
# The questions do not depend on the answers, so a writer thread streams them while the
# answers are read, and the case and answers are checked once the program is done.
import os, sys, threading
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    head, *asked = case_text.strip().split("\n")
    n = int(head)
    io.send(str(n))
    if io.recv_line() != "READY":
        raise RuntimeError("expected READY")

    def ask():
        for at in range(0, len(asked), 4096):
            io.send("\n".join(asked[at:at + 4096]))

    asking = threading.Thread(target=ask, daemon=True)
    asking.start()
    replies = [io.recv_line() for _ in asked]
    asking.join()

    pairs = [tuple(map(int, line.split())) for line in asked]
    keys = {min(u, v) * n + max(u, v) for u, v in pairs if u != v and 0 <= min(u, v) and max(u, v) < n}
    if len(pairs) != n * (n - 1) // 2 or len(keys) != len(pairs):
        raise RuntimeError("bad case: not every pair once")
    par = list(range(n))

    def find(x):
        while par[x] != x:
            par[x] = par[par[x]]
            x = par[x]
        return x

    comps, last = n, len(pairs) - 1
    for k, ((u, v), reply) in enumerate(zip(pairs, replies)):
        if reply not in ("0", "1"):
            raise RuntimeError(f"bad reply {reply!r} to question {k + 1}")
        if k == last and (comps != 2 or find(u) == find(v)):
            raise RuntimeError("connectivity was decided before the last question")
        if reply == "1" and find(u) != find(v):
            par[find(u)] = find(v)
            comps -= 1


if __name__ == "__main__":
    run_interactor(drive)

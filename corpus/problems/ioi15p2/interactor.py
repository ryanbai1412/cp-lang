#!/usr/bin/env python3
# Judge for IOI 2015 Scales, stdio-adapted: the program reads T; for each case it prints
# weighings "H a b c", "L a b c", "M a b c" (heaviest, lightest, median) or "N a b c d"
# (next lightest above d), reading back the chosen coin, then "A w1 .. w6" with the coins
# from lightest to heaviest. Six weighings always suffice (and 3^5 < 720 outcomes force
# six), so a case allows at most six. The case is T then T lines, each the coins from
# lightest to heaviest (gen.py).
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT = 6


def drive(case_text, io):
    head, *rows = [r for r in case_text.split("\n") if r.strip()]
    orders = [list(map(int, r.split())) for r in rows]
    if len(orders) != int(head) or any(sorted(o) != [1, 2, 3, 4, 5, 6] for o in orders):
        raise RuntimeError("bad case: T orders of coins 1..6 expected")
    io.send(head)
    for k, order in enumerate(orders, 1):
        rank = {c: i for i, c in enumerate(order)}
        used = 0
        while True:
            op, *args = io.recv_line().split()
            coins = list(map(int, args))
            if op == "A":
                if coins != order:
                    raise RuntimeError(f"case {k}: answered {coins}, want {order}")
                break
            used += 1
            if used > LIMIT:
                raise RuntimeError(f"case {k}: more than {LIMIT} weighings")
            want = 4 if op == "N" else 3
            if op not in "HLMN" or len(coins) != want or len(set(coins)) != want \
                    or not all(1 <= c <= 6 for c in coins):
                raise RuntimeError(f"case {k}: bad weighing {op} {coins}")
            abc = sorted(coins[:3], key=rank.get)
            if op == "H":
                got = abc[2]
            elif op == "L":
                got = abc[0]
            elif op == "M":
                got = abc[1]
            else:
                above = [c for c in abc if rank[c] > rank[coins[3]]]
                got = above[0] if above else abc[0]
            io.send(str(got))


run_interactor(drive)

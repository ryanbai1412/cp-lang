#!/usr/bin/env python3
"""gen.py seed size | --max: COCI 2021/22 #1 Set. n distinct cards of k <= 12 characters over
{1, 2, 3}, n <= 3^k. --max: k = 12 and n = 3^12, every card (the most sets)."""
import itertools, random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
k = 12 if MAX else r.randint(1, max(1, min(12, int(sys.argv[2]) // 8)))
cards = ["".join(p) for p in itertools.product("123", repeat=k)]
if not MAX:
    cards = r.sample(cards, r.randint(1, min(len(cards), int(sys.argv[2]))))
r.shuffle(cards)
sys.stdout.write(f"{len(cards)} {k}\n" + "\n".join(cards) + "\n")

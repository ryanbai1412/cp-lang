#!/usr/bin/env python3
"""gen.py seed size | --max: NOI 2018 Your Name. A lowercase s with |s| <= 5e5 and q <= 1e5
queries "t l r" with 1 <= l <= r <= |s| and total |t| <= 1e6. --max: |s| = 5e5 over 'ab', q = 1e5
queries (total |t| = 1e6), nearly all of nine letters, a few of 100 and one of about 1e5, on
wide random blocks."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = int(sys.argv[2]) if not MAX else 0
n = 500_000 if MAX else r.randint(1, max(1, S))
A = "ab" if MAX or r.random() < 0.5 else "abc"
s = "".join(r.choice(A) for _ in range(n))
if MAX:
    lens = [9] * 99_989 + [100] * 10 + [1_000_000 - 9 * 99_989 - 1000]
    r.shuffle(lens)
else:
    lens = [r.randint(1, max(1, S)) for _ in range(r.randint(1, max(1, S // 5)))]
out = [s, str(len(lens))]
for L in lens:
    l = r.randint(1, max(1, n // 10)) if MAX else r.randint(1, n)
    rr = r.randint(max(l, n - n // 10), n) if MAX else r.randint(l, n)
    out.append(f"{''.join(r.choice(A) for _ in range(L))} {l} {rr}")
print("\n".join(out))

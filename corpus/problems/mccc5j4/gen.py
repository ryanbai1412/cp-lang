"""mccc5j4 generator: N, then S of lowercase letters and asterisks; N is near the feasible
range so both answers occur. --max: |S| = 1e5, half asterisks."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 10**5 if big else r.randint(1, 1 + size * 3)
star = 0.5 if big else r.random()
s = "".join("*" if r.random() < star else r.choice("abcdefghijklmnopqrstuvwxyz") for _ in range(n))
fixed = sum(ord(c) - 96 for c in s if c != "*")
w = s.count("*")
lo, hi = fixed + w, fixed + 26 * w
goal = (lo + hi) // 2 if big else r.randint(max(1, lo - 3), hi + 3)
print(max(1, min(10**9, goal)))
print(s)

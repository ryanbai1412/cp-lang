"""ccc21s5 checker: `checker.py input output answer`. Impossible exactly when the answer is;
otherwise N values in 1..=1e9 whose GCD over every X..=Y is that requirement's Z."""
import sys
from math import gcd

inp = list(map(int, open(sys.argv[1]).read().split()))
out = open(sys.argv[2]).read().split()
ans = open(sys.argv[3]).read().split()
n, m = inp[0], inp[1]
if ans[:1] == ["Impossible"]:
    sys.exit(0 if out == ["Impossible"] else 1)
if len(out) != n or not all(t.isdigit() for t in out):
    sys.exit(1)
a = [int(t) for t in out]
if not all(1 <= x <= 10**9 for x in a):
    sys.exit(1)
sp = [a]
while (1 << len(sp)) <= n:
    p, h = sp[-1], 1 << (len(sp) - 1)
    sp.append([gcd(p[i], p[i + h]) for i in range(len(p) - h)])
for i in range(m):
    x, y, z = inp[2 + 3 * i: 5 + 3 * i]
    k = (y - x + 1).bit_length() - 1
    if gcd(sp[k][x - 1], sp[k][y - (1 << k)]) != z:
        sys.exit(1)
sys.exit(0)

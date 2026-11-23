import sys
from math import lcm
from functools import cache
@cache
def rhythmic(keep):
    # the 10^18-long string repeats with period L = lcm(keep), so its smallest period divides L
    # and a prefix of 3L has the same one (Fine and Wilf)
    L = lcm(*keep) if keep else 1
    s = bytearray(3 * L + 1)
    for a in keep: s[a::a] = b"\1" * len(s[a::a])
    s = bytes(s[1:])
    p = next(p for p in range(1, L + 1) if L % p == 0 and s[p:] == s[:-p])
    return s[:p].count(1) == 1
it = iter(sys.stdin.read().split())
for _ in range(int(next(it))):
    n = int(next(it)); a = [int(next(it)) for _ in range(n)]
    print(min(n - bin(m).count("1") for m in range(1 << n)
              if rhythmic(frozenset(a[i] for i in range(n) if m >> i & 1))))

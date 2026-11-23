import sys
from itertools import groupby
def compress(s):
    return "".join(k for k, _ in groupby(s))
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    s = next(it); n = len(s)
    seen = {compress(s)}
    for i in range(1, n - 1):
        seen.add(compress(s[:i] + "10"[int(s[i])] + s[i + 1:]))
    print(len(seen))

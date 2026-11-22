#!/usr/bin/env python3
n = 100000
reps = 300
print(n, reps)
x = 271828182
out = []
for _ in range(n):
    x = (x * 1103515245 + 12345) & 0x7FFFFFFF
    out.append(str(x % 998244353))
print(" ".join(out))

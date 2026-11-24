#!/usr/bin/env python3
n, q = 200000, 2000000
print(n, q)
x = 246813579
out = []
for _ in range(n):
    x = (x * 1103515245 + 12345) & 0x7FFFFFFF
    out.append(str(x % 1000000 + 1))
print(" ".join(out))

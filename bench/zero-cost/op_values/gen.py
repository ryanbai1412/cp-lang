#!/usr/bin/env python3
n = 100000
reps = 600
print(n, reps)
x = 246813579
out = []
for _ in range(n):
    x = (x * 1103515245 + 12345) & 0x7FFFFFFF
    out.append(str(x % 1000))
print(" ".join(out))

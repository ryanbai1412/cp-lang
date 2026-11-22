#!/usr/bin/env python3
n = 100000
reps = 400
print(n, reps)
x = 987654321
out = []
for _ in range(n):
    x = (x * 1103515245 + 12345) & 0x7FFFFFFF
    out.append(str(x % 10000))
print(" ".join(out))

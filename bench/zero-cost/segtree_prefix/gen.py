#!/usr/bin/env python3
n = 100000
q = 400000
print(n, q)
x = 314159265
arr = []
for _ in range(n):
    x = (x * 1103515245 + 12345) & 0x7FFFFFFF
    arr.append(str(x % 1000))
print(" ".join(arr))
lines = []
for i in range(q):
    x = (x * 1103515245 + 12345) & 0x7FFFFFFF
    if (i & 3) == 0:
        idx = x % n
        x = (x * 1103515245 + 12345) & 0x7FFFFFFF
        val = x % 1000
        lines.append(f"1 {idx} {val}")
    else:
        l = x % n
        x = (x * 1103515245 + 12345) & 0x7FFFFFFF
        r = x % n
        if l > r:
            l, r = r, l
        lines.append(f"2 {l} {r}")
print("\n".join(lines))

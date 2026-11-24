#!/usr/bin/env python3
# A 1500 x 1500 grid of 1..=n*n, as imo22p6checker's largest test.
n = 1500
s = 88_172_645
print(n)
for _ in range(n):
    row = []
    for _ in range(n):
        s = (s * 1_103_515_245 + 12_345) % 2_147_483_648
        row.append(s % (n * n) + 1)
    print(" ".join(map(str, row)))

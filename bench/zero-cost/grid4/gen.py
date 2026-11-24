#!/usr/bin/env python3
# A 50 x 50 chocolate, 1..=1000 raisins per piece, as ioi09p4's largest test.
n = m = 50
s = 88_172_645
rows = []
for _ in range(n):
    row = []
    for _ in range(m):
        s = (s * 1_103_515_245 + 12_345) % 2_147_483_648
        row.append(s % 1000 + 1)
    rows.append(" ".join(map(str, row)))
print(n, m)
print("\n".join(rows))

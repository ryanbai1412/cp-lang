import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "100"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
base = 16 if big else r.choice([2, 3, 4, 5, 6, 7, 8, 9, 10, 16])
k = 100 if big else r.randint(1, max(1, min(S, 100)))
digs = "0123456789ABCDEF"[:base]
s = r.choice(digs[1:]) + "".join(r.choice(digs) for _ in range(k - 1))
if not big and r.random() < 0.15: s = s[: (k + 1) // 2] + s[: k // 2][::-1]
print(base)
print(s)

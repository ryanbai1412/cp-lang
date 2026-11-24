import sys, random
# port: 1 <= n <= 2e5 nodes, wants, gives, then for nodes 2..=n a parent p before it and an edge
# weight w. The statement is not public: weights stay within |w| <= 1e4 and amounts within 0..=1e4,
# so the accepted C++'s int sums of weights along a 2e5-deep path stay in range.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 200000
n = 200000 if big else r.randint(1, max(1, min(S, 200000)))
top = 10 ** 4 if big else r.choice([3, 100, 10 ** 4])
want = [r.randint(0, top) for _ in range(n)]
give = [r.randint(0, top) for _ in range(n)]
print(n)
print(*want)
print(*give)
deep = big or r.random() < 0.3
out = []
for i in range(1, n):
    p = i if deep and r.random() < 0.9 else r.randint(1, i)
    out.append(f"{p} {r.randint(-top, top)}")
print("\n".join(out))

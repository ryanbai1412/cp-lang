import sys, random
# seed size: T <= 3 strings of length <= 3 size over 1..3 letters; --max: T = 10 strings of
# 30000, all `a`, periodic and random over {a, b}
if sys.argv[1] == "--max":
    r = random.Random(0); n = 30000
    xs = ["a" * n, "ab" * (n // 2), ("aab" * n)[:n], ("abaab" * n)[:n], ("aabaa" * n)[:n]]
    xs += ["".join(r.choice("ab") for _ in range(n)) for _ in range(5)]
    print(len(xs)); print(*xs, sep="\n"); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 3); print(t)
for _ in range(t):
    k = r.randint(1, 3)
    print("".join(r.choice("abc"[:k]) for _ in range(r.randint(1, min(30000, 3 * S)))))

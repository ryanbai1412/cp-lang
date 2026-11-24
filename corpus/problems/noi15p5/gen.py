import sys, random
# seed size: n <= size + 1 cups over a 1..3-letter alphabet, scores of a random magnitude
# (some negative); --max: n = 3e5, all `a` (one deep group) and scores up to 1e9
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 3 * 10**5 if mx else int(sys.argv[2])
n = 3 * 10**5 if mx else r.randint(1, min(3 * 10**5, S + 1))
k = 1 if mx else r.randint(1, 3)
cap = 10**9 if mx else r.choice([3, 1000, 10**9])
print(n); print("".join(r.choice("abc"[:k]) for _ in range(n)))
print(*(r.randint(-cap, cap) for _ in range(n)))

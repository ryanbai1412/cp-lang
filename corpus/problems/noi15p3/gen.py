import sys, random
# seed size: 2 <= n <= size + 2, any modulus 1 <= p <= 1e9; --max: n = 500
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 498 if mx else int(sys.argv[2])
n = 500 if mx else r.randint(2, min(500, S + 2))
p = 999999937 if mx else r.choice([1, 2, 3, r.randint(1, 100), r.randint(1, 10**9)])
print(n, p)

import sys, random
# seed size: n, m <= size + 1 (at most 80), entries in 0..cap; --max: n = m = 80 all 1000
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 80 if mx else int(sys.argv[2])
n = 80 if mx else r.randint(1, min(80, S + 1)); m = 80 if mx else r.randint(1, min(80, S + 1))
cap = 1000 if mx else r.choice([1, 10, 1000])
print(n, m)
for _ in range(n):
    print(*(1000 if mx else r.randint(0, cap) for _ in range(m)))

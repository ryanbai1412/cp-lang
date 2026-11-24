import sys, random
# seed size: n <= size + 1 intervals on a coordinate range that is small (so they pile up) or
# wide, m <= n + 1 (sometimes no answer); --max: n = 5e5, m = 2e5, coordinates up to 1e9
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 5 * 10**5 if mx else int(sys.argv[2])
n = 5 * 10**5 if mx else r.randint(1, min(5 * 10**5, S + 1))
m = 2 * 10**5 if mx else r.randint(1, min(2 * 10**5, n + 1))
w = 10**9 if mx else r.choice([8, 50, 10**9])
print(n, m)
for _ in range(n):
    l = r.randint(0, w); print(l, r.randint(l, min(10**9, l + r.choice([w, 5]))))

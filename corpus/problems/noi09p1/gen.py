import sys, random
# seed size [rand]: n <= size + 1; D from a random permutation (solvable), or with `rand`
#   random D (mostly "No Answer"); --max: n = 1e4
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 9999 if mx else int(sys.argv[2])
n = 10**4 if mx else r.randint(1, min(10**4, S + 1))
if mx or len(sys.argv) < 4:
    t = list(range(n)); r.shuffle(t)
    d = [min(abs(i - t[i]), n - abs(i - t[i])) for i in range(n)]
else:
    d = [r.randint(0, n // 2) for _ in range(n)]
print(n); print(*d)

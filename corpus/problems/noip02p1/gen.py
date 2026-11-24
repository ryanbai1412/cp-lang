import sys, random
# seed size: n <= size + 1 piles, values in 0..cap with sum divisible by n, built by random
# transfers away from the average; --max: n = 100 alternating 0 and 10000 (every prefix uneven)
mx = sys.argv[1] == "--max"
if mx:
    n, a = 100, [0, 10000] * 50
else:
    r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
    n = r.randint(1, min(100, S + 1)); cap = r.choice([10, 100, 10000])
    a = [r.randint(0, cap)] * n
    for _ in range(4 * n):
        i, j = r.randrange(n), r.randrange(n)
        d = r.randint(0, min(a[i], cap - a[j]))
        a[i] -= d; a[j] += d
print(n); print(*a)

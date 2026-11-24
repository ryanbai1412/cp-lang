import sys, random
# seed size: n, m <= size^2 (at most 1e9), 2 <= k <= 2000 (sometimes a product of many small primes);
# --max: n = m = 1e9, k = 1890 = 2 * 3^3 * 5 * 7 (the most distinct primes under 2000, a square factor)
if sys.argv[1] == "--max":
    print(10**9, 10**9, 1890); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
c = min(10**9, S * S)
k = r.choice([r.randint(2, 2000), r.choice([2, 3, 6, 30, 210, 420, 1890, 1155, 1999])])
print(r.randint(1, c), r.randint(1, c), k)

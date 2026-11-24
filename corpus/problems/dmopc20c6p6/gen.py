import sys, random
# statement: 2 <= N <= 4e4 buckets, capacities 1..=M, M <= 1e4.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "40000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 40000
n = 40000 if big else r.randint(2, max(2, min(40000, S)))
m = 10000 if big else r.choice([1, 2, 5, r.randint(1, 10000)])
print(n, m)
print(*(r.randint(1, m) for _ in range(n)))

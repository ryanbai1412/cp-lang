"""CCO 2023 P1 inputs: `gen.py seed size` (a random bit string of up to `size` bits, often with long
runs, and its k-window sums), or `gen.py --max` (n = 1e6, k = 1000, a random string)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 10**6 if MAX else r.randint(1, max(1, S))
k = 1000 if MAX else r.randint(1, n)
p = 0.5 if MAX else r.choice([0.1, 0.5, 0.9])
bits = [int(r.random() < p) for _ in range(n)]
sums = [sum(bits[:k])]
for i in range(n - k):
    sums.append(sums[-1] + bits[i + k] - bits[i])
print(n, k)
print(*sums)

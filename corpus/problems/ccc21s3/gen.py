"""gen.py SEED SIZE | --max: CCC 2021 S3. N <= 2e5 friends (P <= 1e9, W <= 1000, D <= 1e9).
--max is N = 2e5 over the full ranges."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 2 * 10**5 if MAX else r.randint(1, 2 * int(sys.argv[2]))
M = 10**9 if MAX else r.choice([10, 100, 10**9])
print(n)
print("\n".join(f"{r.randint(0, M)} {r.randint(1, 1000)} {r.randint(0, M // r.choice([1, 5, 50]))}" for _ in range(n)))

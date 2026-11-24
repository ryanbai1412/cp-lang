import sys, random
# usage: gen.py seed size | --max (N = 5000)
MAX = sys.argv[1] == "--max"
r = random.Random(1 if MAX else int(sys.argv[1])); S = 10**9 if MAX else int(sys.argv[2])
# statement: 1 <= N <= 5000; the sample grader reads N, each switch's open position, each switch's door.
n = r.randint(1, min(5000, 1 + 2 * S)) if S < 1000 else 5000 - (0 if MAX else r.randrange(3))
doors = list(range(n)); r.shuffle(doors)
print(n)
print(*[r.randint(0, 1) for _ in range(n)])
print(*doors)

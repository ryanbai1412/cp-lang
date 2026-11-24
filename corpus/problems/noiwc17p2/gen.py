"""gen.py SEED SIZE | --max: NOI WC 2017 P2, three subtasks picked by the seed. Sorting: n up to
2e8 xorshift numbers from a seed. Game: rows of n <= 3e5 strategies in 0..=2 and q <= 3e5
queries 0 <= x, y < n, 1 <= l <= n - max(x, y). Parentheses: n <= 266666 characters of ( ) ?.
Sizes scale n (sorting: 1e3 * SIZE; the others: 4 * SIZE), capped at those bounds; --max is the
slowest full-bound test, statement test 3: sorting n = 2e8 numbers from seed 2017012503."""
import random, sys
MAX = sys.argv[1] == "--max"
seed = 0 if MAX else int(sys.argv[1])
r = random.Random(seed)
S = 0 if MAX else int(sys.argv[2])
task = 1 if MAX else seed % 3 + 1
print(task)
if MAX:
    print(200_000_000, 2017012503)
elif task == 1:
    print(min(200_000_000, r.randint(1, 1000 * S)), r.randint(0, 2**32 - 1))
elif task == 2:
    n = r.randint(1, min(300_000, 4 * S))
    q = r.randint(1, min(300_000, 4 * S))
    print(n, q)
    for _ in range(2):
        print("".join(r.choice("012") for _ in range(n)))
    for _ in range(q):
        x, y = r.randrange(n), r.randrange(n)
        print(x, y, r.randint(1, n - max(x, y)))
else:
    n = r.randint(1, min(266_666, 4 * S))
    p = r.choice([0.3, 0.7, 0.95, 1])
    s = "".join("?" if r.random() < p else r.choice("()") for _ in range(n))
    print(n)
    print(s)

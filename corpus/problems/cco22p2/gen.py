"""CCO 2022 P2 inputs: `gen.py seed size` (few stops; small stop sizes, crowds and umbrella
stocks so both dry and wet cases come up), or `gen.py --max` (n = 1e6 at full bounds, tuned so
that everyone stays dry and the spare-umbrella stack is used)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 1_000_000 if MAX else r.randint(2, max(2, S))
if MAX:
    bus = [r.randint(5 * 10**8, 2 * 10**9) for _ in range(n)]
    ppl = [r.randint(0, 10**9) for _ in range(n - 1)]
    umb = [r.randint(0, 10**9) for _ in range(n - 1)]
else:
    big = r.choice([3, 10, 100])
    bus = [r.randint(0, 2 * big) for _ in range(n)]
    ppl = [r.randint(0, 2 * big) for _ in range(n - 1)]
    umb = [r.randint(0, big) for _ in range(n - 1)]
print(n)
print(*bus)
print(*ppl)
print(*umb)

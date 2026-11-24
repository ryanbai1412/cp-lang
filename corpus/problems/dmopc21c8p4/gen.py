"""gen.py SEED SIZE | --max: DMOPC '21 Contest 8 P4 (Grid Operations). N, M, Q >= 1 with
NM <= 1e6 and Q <= 1e6, types 1-5 either uniform or restricted to one subtask's set.
--max is N = M = 1000 with 1e6 uniform operations."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
if MAX:
    n = m = 1000
    q = 10**6
    types = [1, 2, 3, 4, 5]
else:
    cap = max(1, min(10**6, int(sys.argv[2])))
    n = r.randint(1, cap)
    m = r.randint(1, max(1, cap // n))
    q = r.randint(1, cap)
    types = r.choice([[1, 2, 3, 4, 5], [1, 3, 5], [1, 2], [1, 2, 3, 4], [5], [2, 4, 5]])
sys.stdout.write(f"{n} {m} {q}\n" + "".join(f"{r.choice(types)}\n" for _ in range(q)))

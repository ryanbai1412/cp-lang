"""CEOI 2022 Homework inputs: `gen.py seed size` (an expression with up to `size` question
marks, random shape) or `gen.py --max` (N = 1e6 marks, random shape)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 10**6 if MAX else r.randint(1, max(1, int(sys.argv[2])))
chain = not MAX and r.random() < .3
out, todo = [], [n]
while todo:
    k = todo.pop()
    if isinstance(k, str): out.append(k); continue
    if k == 1: out.append("?"); continue
    out.append(r.choice(["min(", "max("]))
    left = r.choice([1, k - 1]) if chain else r.randint(1, k - 1)
    todo += [")", k - left, ",", left]
print("".join(out))

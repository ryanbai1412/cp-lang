import sys, random
if sys.argv[1:] == ["--max"]:
    # n = 1e5, the letters shuffled
    sys.argv[1:] = ["3", "100000"]

# gen.py seed size: n I's and n U's in some order, 1 <= n <= 1e5.
# No arguments, or size >= 1e5, gives n = 1e5; the seed picks the arrangement.
seed = int(sys.argv[1]) if len(sys.argv) > 1 else 0
S = min(int(sys.argv[2]) if len(sys.argv) > 2 else 10 ** 5, 10 ** 5)
r = random.Random(seed)
n = S if S == 10 ** 5 else r.randint(1, max(1, S))
kind = seed % 5
if kind == 0: s = "U" * n + "I" * n
elif kind == 1: s = "UI" * n
elif kind == 2: s = "IU" * n
else:
    s = list("I" * n + "U" * n); r.shuffle(s); s = "".join(s)
print(n); print(s)

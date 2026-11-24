"""gen.py SEED SIZE | --max: Codeforces 802 "Fake News (hard)". T <= 10 strings of 1..1e5
lowercase letters. Small cases use a 1-3 letter alphabet so substrings repeat. --max is T = 10
strings of length 1e5: all one letter, a period-2 string, Fibonacci words and random ones."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
N = 100_000
def fib(n):
    a, b = "a", "ab"
    while len(b) < n:
        a, b = b, b + a
    return b[:n]
if MAX:
    ss = ["a" * N, "ab" * (N // 2), fib(N), fib(N).replace("a", "c")]
    ss += ["".join(r.choice("ab") for _ in range(N)) for _ in range(3)]
    ss += ["".join(r.choice("abcdefghijklmnopqrstuvwxyz") for _ in range(N)) for _ in range(3)]
else:
    t = r.randint(1, 10)
    ss = []
    for _ in range(t):
        k = r.randint(1, 3)
        ss.append("".join(r.choice("abc"[:k]) for _ in range(r.randint(1, 1 + 2 * S))))
sys.stdout.write(f"{len(ss)}\n" + "".join(x + "\n" for x in ss))

"""nccc10s3 generator: s, q, then q lines "t k" (total |t| <= 2e5). t is usually a substring
of s. --max: s = 'a' * (2e5 - 1) + 'b' and 2e5 queries for 'a' (all but one suffix matches; the
one 'b' keeps the text off a single-letter shortcut)."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
if big:
    n, q = 200_000, 200_000
    s = "a" * (n - 1) + "b"
    qs = [("a", r.randint(1, n + 5)) for _ in range(q)]
else:
    abc = "abc"[: r.randint(1, 3)]
    n = r.randint(1, 1 + size * 4)
    s = "".join(r.choice(abc) for _ in range(n))
    qs = []
    for _ in range(r.randint(1, 1 + size * 2)):
        if r.random() < 0.8:
            i = r.randrange(n)
            t = s[i : i + r.randint(1, 3)]
        else:
            t = "".join(r.choice("abc") for _ in range(r.randint(1, 3)))
        qs.append((t, r.randint(1, n + 2)))
print(s)
print(len(qs))
for t, k in qs:
    print(t, k)

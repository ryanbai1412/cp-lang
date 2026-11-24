import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "50000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# the max case keeps the input under 1 MB, the read buffer of the accepted C++
abc = "abc" if big else "abcdefghijklmnopqrstuvwxyz"[:r.choice([2, 3, 26])]
lo, hi = (8, 10) if big else (1, r.randint(1, 25))
n = S if big else r.randint(1, max(1, min(S, 50000, sum(len(abc) ** k for k in range(lo, hi + 1)))))
m = 1000 if big else r.randint(1, min(1000, max(1, S)))
def word(k): return "".join(r.choice(abc) for _ in range(k))
def case(w): return "".join(c.upper() if r.random() < 0.3 else c for c in w)
seen = set()
while len(seen) < n: seen.add(word(r.randint(lo, hi)))
words = list(seen); r.shuffle(words)
print(n, m)
for w in words: print(case(w))
for _ in range(m): print(case(word(500 if big else r.randint(1, 500 if S > 50 else 30))))

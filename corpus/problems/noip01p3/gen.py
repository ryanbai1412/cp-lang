import sys, random
# seed size: p <= min(10, size) lines of 20 letters over a 1..4-letter alphabet, k parts in
# 1..min(40, 20p), up to 6 words (often substrings of the text, some of one letter);
# --max: p = 10, k = 40, all `a` with the words a, aa, ..., aaaaaa
mx = sys.argv[1] == "--max"
if mx:
    print(10, 40); print(*["a" * 20] * 10, sep="\n"); print(6); print(*("a" * i for i in range(1, 7)), sep="\n"); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
p = r.randint(1, min(10, S)); k = r.randint(1, min(40, 20 * p))
al = "abcd"[:r.randint(1, 4)]
s = "".join(r.choice(al) for _ in range(20 * p))
m = r.randint(1, 6); ws = []
for _ in range(m):
    L = r.randint(1, 4)
    if r.random() < 0.7:
        i = r.randrange(len(s) - L + 1); ws.append(s[i:i + L])
    else:
        ws.append("".join(r.choice(al) for _ in range(L)))
print(p, k); print(*(s[i:i + 20] for i in range(0, len(s), 20)), sep="\n"); print(m); print(*ws, sep="\n")

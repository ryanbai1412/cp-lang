import sys, random, itertools
# seed size: a base n puzzle a + b = c whose letters have exactly one digit assignment
# (the statement's guarantee); --max: base 26, a + a = c (a random a + b in base 26 makes
# every known solver search for minutes)
def words(r, n, perm, double=False):
    while True:
        a = r.randrange(n ** n // 2 if double else n ** (n - 1))
        b = a if double else r.randrange(n ** n - a)
        num = lambda x: [(x // n ** i) % n for i in range(n)][::-1]
        ws = [num(a), num(b), num(a + b)]
        if len({d for w in ws for d in w}) == n:
            perm = perm or list(range(n))
            return [''.join(chr(65 + perm.index(d)) for d in w) for w in ws]
def unique(n, ws):
    sols = 0
    for p in itertools.permutations(range(n)):
        v = lambda w: sum(p[ord(ch) - 65] * n ** (len(w) - 1 - i) for i, ch in enumerate(w))
        if v(ws[0]) + v(ws[1]) == v(ws[2]):
            sols += 1
            if sols > 1: return False
    return sols == 1
if sys.argv[1] == "--max":
    r = random.Random(2); n = 26
    ws = words(r, n, None, True); perm = list(range(n)); r.shuffle(perm)
    print(n); print(*ws, sep="\n"); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
while True:
    n = r.randint(2, min(6, 2 + S // 4)); perm = list(range(n)); r.shuffle(perm)
    ws = words(r, n, perm)
    if unique(n, ws): break
print(n); print(*ws, sep="\n")

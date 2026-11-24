import sys, random
# statement: T puzzles, lowercase A and B, sum of lengths <= 1e6.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])


def merged(a):
    s = list(a)
    for _ in range(r.randint(0, len(s))):
        js = [j for j in range(len(s) - 1) if s[j] == s[j + 1] and s[j] != "z"]
        if not js:
            break
        j = r.choice(js)
        s[j:j + 2] = [chr(ord(s[j]) + 1)]
    return "".join(s)


big = S >= 10 ** 6
tests = []
for _ in range(1 if big else r.randint(1, 5)):
    n = 5 * 10 ** 5 if big else r.randint(1, max(1, min(S, 10)))
    letters = "yz" if big else r.choice(["ab", "abc", "yz", "a"])
    a = "".join(r.choice(letters) for _ in range(n))
    b = a if big else (merged(a) if r.random() < 0.7 else "".join(r.choice(letters) for _ in range(r.randint(1, n))))
    tests.append((a, b))
print(len(tests))
for a, b in tests:
    print(a)
    print(b)

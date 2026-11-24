"""gen.py SEED SIZE | --max: CCC 2020 S3. Lowercase needle N and haystack H, |N|, |H| <= 2e5,
over a small alphabet so permutations repeat. --max is |N| = 1000, |H| = 2e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 2 * 10 ** 5 if MAX else max(1, min(2 * 10 ** 5, int(sys.argv[2])))
a = "ab" if MAX else "abcdefghijklmnopqrstuvwxyz"[:r.choice([1, 2, 3, 26])]
h = 2 * 10 ** 5 if MAX else r.randint(1, S)
k = 1000 if MAX else r.randint(1, max(1, min(h + 2, r.choice([3, 10, h]))))
print("".join(r.choice(a) for _ in range(k)))
print("".join(r.choice(a) for _ in range(h)))

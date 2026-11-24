"""gen.py SEED SIZE | --max: NOI 2023 P6 - Merge the Books. t test cases (1 <= t <= 10), each with
n books (1 <= n <= 100, n <= size) of weight 1 <= w_i <= 1e9. Each case draws its weights one of
five ways: all equal, tiny (many ties), up to 1e9, light books with a few heavy ones, or a sorted
range with half the books copying the lightest. --max is t = 10 cases of n = 100: all ones, all
1e9, then eight of the random shapes."""
import random
import sys

B = 10**9


def case(r, n):
    kind = r.randrange(5)
    if kind == 0:
        w = [r.randint(1, B)] * n
    elif kind == 1:
        w = [r.randint(1, 3) for _ in range(n)]
    elif kind == 2:
        w = [r.randint(1, B) for _ in range(n)]
    elif kind == 3:
        hi = r.randint(1, B)
        w = [r.randint(1, max(1, hi // 1000)) for _ in range(n)]
        for i in r.sample(range(n), r.randint(1, n)):
            w[i] = r.randint(1, hi)
    else:
        w = sorted(r.randint(1, B) for _ in range(n))
        w = [x if r.random() < 0.5 else w[0] for x in w]
    return f"{n}\n{' '.join(map(str, w))}"


if sys.argv[1] == "--max":
    r = random.Random(2023)
    cases = [f"100\n{' '.join(['1'] * 100)}", f"100\n{' '.join([str(B)] * 100)}"]
    cases += [case(r, 100) for _ in range(8)]
else:
    r = random.Random(int(sys.argv[1]))
    size = int(sys.argv[2])
    cases = [case(r, r.randint(1, min(100, size))) for _ in range(r.randint(1, 10))]
print(len(cases))
print("\n".join(cases))

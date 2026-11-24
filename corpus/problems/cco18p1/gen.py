import sys, random
if sys.argv[1:] == ["--max"]:
    # n = 1000 games each, alternating results and points up to 1e6, so most pairs are rivalries
    r = random.Random(0)
    print(1000)
    for first in "WL":
        print("".join(first if i % 2 == 0 else "WL"[first == "W"] for i in range(1000)))
        print(*[r.randint(1, 10 ** 6) for _ in range(1000)])
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 1); print(n)
for _ in range(2):
    print(''.join(r.choice('WL') for _ in range(n)))
    print(*[r.randint(1, r.choice([4, 10 ** 6])) for _ in range(n)])

import sys, random

if sys.argv[1:] == ["--max"]:
    # sum n = 1e8 with t = 1e4: four tests at the 2e7 cap and 9996 tests sharing the rest.
    t, big, total = 10_000, 20_000_000, 100_000_000
    rest = total - 4 * big
    small = [rest // (t - 4) + (1 if i < rest % (t - 4) else 0) for i in range(t - 4)]
    ns = [big] * 2 + small[: len(small) // 2] + [big] * 2 + small[len(small) // 2 :]
    print(t)
    print("\n".join(map(str, ns)))
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = 6
print(t)
for i in range(t):
    if i == 0:
        print(r.randint(1, 15))
    else:
        print(r.randint(1, 15 * S + 5))

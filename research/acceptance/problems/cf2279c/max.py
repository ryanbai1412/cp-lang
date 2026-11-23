import random
random.seed(1)
N, T, SMALL = 500000, 1000, 100
tests = [(SMALL, 10**9, [random.randint(10**8, 10**9) for _ in range(SMALL)]) for _ in range(T - 4)]
big = (N - SMALL * (T - 4)) // 4
tests.append((big, 10**9, [10**9] * big))
tests.append((big, 10**9, sorted(random.randint(1, 10**9) for _ in range(big))))
tests.append((big, random.randint(1, 10**9), [random.randint(1, 10**9) for _ in range(big)]))
tests.append((big, 1, [random.randint(1, 10**9) for _ in range(big)]))
out = [str(T)]
for n, l, a in tests: out += [f"{n} {l}", " ".join(map(str, a))]
print("\n".join(out))

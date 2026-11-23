import random
random.seed(1)
small = 9996
print(small + 4)
# many tiny tests: a per-test clear of a fixed 6e5 array is 6e9 writes here
for _ in range(small):
    print(10); print(*sorted(random.sample(range(1, 31), 10)))
n = 40000; print(n); print(*range(1, n + 1))
n = 30000; print(n); print(*[3 * (i // 2) + 1 + i % 2 for i in range(n)])
n = 30000; print(n); print(*sorted(random.sample(range(1, 3 * n + 1), n)))
n = 200000 - 10 * small - 100000; print(n); print(*range(2 * n + 1, 3 * n + 1))

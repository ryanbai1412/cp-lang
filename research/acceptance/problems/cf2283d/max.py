import random
random.seed(1)
# a deep path (recursion depth), a random tree, a star, then 9997 tiny tests (per-test resets)
big = [("path", 130001), ("random", 100001), ("star", 39999)]
print(len(big) + 9997)
for shape, n in big:
    print(n)
    lab = list(range(1, n + 1)); random.shuffle(lab)
    for v in range(1, n):
        p = {"path": v - 1, "random": random.randint(max(0, v - 50), v - 1) if v % 2 else random.randint(0, v - 1), "star": 0}[shape]
        print(lab[p], lab[v])
for _ in range(9997):
    print(3); print(random.choice(["1 2\n2 3", "2 1\n1 3", "1 3\n3 2"]))

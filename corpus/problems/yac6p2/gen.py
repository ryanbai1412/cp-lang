import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = S if big else r.randint(1, min(S, 10**6))
def tree(root, deep):
    order = [root] + r.sample([v for v in range(1, n + 1) if v != root], n - 1)
    par = [0] * (n + 1); par[root] = -1
    for i in range(1, n):
        par[order[i]] = order[i - 1] if deep and r.random() < 0.9 else order[r.randrange(max(0, i - 3) if deep else 0, i)]
    return par[1:]
root = r.randint(1, n)
other = root if big or r.random() < 0.85 else r.randint(1, n)
print(n)
print(*tree(root, big or r.random() < 0.3))
print(*tree(other, big or r.random() < 0.3))

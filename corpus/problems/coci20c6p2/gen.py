import sys, random
# statement: 2 <= n <= 1000 words (<= 20 lowercase letters, each in some connection),
# 1 <= m <= 1000 connections x y t (x != y, 1 <= t <= 1e9), 1 <= q <= 1000 rounds a b (a != b).
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10 ** 6
m = 1000 if big else r.randint(1, max(1, min(S, 1000)))
n = 1000 if big else r.randint(2, min(2 * m, max(2, min(S, 1000))))
words = set()
while len(words) < n:
    words.add("".join(r.choice("abc" if not big and n < 20 else "abcdefghijklmnopqrstuvwxyz") for _ in range(r.randint(1, 20) if big else r.randint(1, 4))))
words = list(words)
r.shuffle(words)
edges = []
for i in range(0, n - 1, 2):
    edges.append((words[i], words[i + 1]))
if n % 2:
    edges.append((words[-1], words[0]))
while len(edges) < m:
    x, y = r.sample(words, 2)
    edges.append((x, y))
r.shuffle(edges)
top = 10 ** 9 if big or r.random() < 0.5 else 10
print(n, len(edges))
for x, y in edges:
    print(x, y, r.randint(1, top))
q = 1000 if big else r.randint(1, max(1, min(S, 1000)))
print(q)
for _ in range(q):
    print(*r.sample(words, 2))

import sys
from itertools import product
# every way to give n <= 9 points to k groups whose bounding boxes are pairwise disjoint
t = list(map(int, open(sys.argv[1]).read().split())); n, k = t[0], t[1]
pts = [(t[2 + 2 * i], t[3 + 2 * i]) for i in range(n)]
best = None
for col in product(range(k), repeat=n):
    boxes = []
    for g in range(k):
        q = [pts[i] for i in range(n) if col[i] == g]
        if q: boxes.append((min(x for x, _ in q), max(x for x, _ in q), min(y for _, y in q), max(y for _, y in q)))
    if all(a[1] < b[0] or b[1] < a[0] or a[3] < b[2] or b[3] < a[2] for i, a in enumerate(boxes) for b in boxes[i + 1:]):
        s = sum((b[1] - b[0]) * (b[3] - b[2]) for b in boxes)
        best = s if best is None else min(best, s)
print(best)

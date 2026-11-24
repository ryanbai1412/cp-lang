import sys, random
from math import gcd
# seed size: T <= 5 tests of n, m <= size + 1. Planted tests pick a hidden hit count x, run the
# sword choice, and give each dragon a regrowth dividing atk x - life (gcd with L, so the lcm stays
# under 1e12), so an answer exists; other tests are random small, often -1.
# --max: T = 5, n = m = 1e5, planted; swords and rewards <= 5e5 < lives <= 1e6, so each dragon
# takes the strongest sword (a heap keeps the generator fast).
L = 64 * 81 * 25 * 7 * 11 * 13
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1]))
S = 10**5 if mx else int(sys.argv[2])
T = 5 if mx else r.randint(1, 5)
print(T)
for _ in range(T):
    n = 10**5 if mx else r.randint(1, S + 1)
    m = 10**5 if mx else r.randint(1, S + 1)
    top = 10**6 if mx else r.choice([10, 1000, 10**6])
    planted = mx or r.random() < 0.6
    lives = [r.randint(1, top) for _ in range(n)]
    rewards = [r.randint(1, top) for _ in range(n)]
    start = [r.randint(1, top) for _ in range(m)]
    if mx:
        import heapq
        lives = [r.randint(5 * 10**5 + 1, 10**6) for _ in range(n)]
        rewards = [r.randint(1, 5 * 10**5) for _ in range(n)]
        start = [r.randint(1, 5 * 10**5) for _ in range(m)]
        x, heap, regrow = r.randint(10**6, 2 * 10**6), [-s for s in start], []
        heapq.heapify(heap)
        for a, w in zip(lives, rewards):
            atk = -heapq.heapreplace(heap, -w)
            regrow.append(gcd(atk * x - a, L) or L)
    elif planted:
        import bisect
        x, bag, regrow = r.randint(top, 2 * top), sorted(start), []
        for a, w in zip(lives, rewards):
            i = bisect.bisect_right(bag, a)
            atk = bag[i - 1] if i else bag[0]
            bag.pop(bisect.bisect_left(bag, atk)); bisect.insort(bag, w)
            regrow.append(gcd(atk * x - a, L) or L)
    else:
        regrow = [r.randint(1, 12) for _ in range(n)]
    print(n, m); print(*lives); print(*regrow); print(*rewards); print(*start)

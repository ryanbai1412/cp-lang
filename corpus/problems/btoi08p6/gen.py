# usage: gen.py SEED [SIZE]: n <= min(20, SIZE) colours with glove counts; some colour has both.
import random, sys
if sys.argv[1:] == ["--max"]:
    # n = 20 colours of up to 1e8 gloves: all 2^20 colour sets are boxes
    g = random.Random(1)
    print(20)
    print(*(g.randint(1, 10**8) for _ in range(20)))
    print(*(g.randint(1, 10**8) for _ in range(20)))
    sys.exit()

r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else r.choice([3, 6, 20])
n = r.randint(1, min(20, max(1, size)))
top = r.choice([1, 3, 10, 10**8])
a = [r.choice([0, r.randint(0, top)]) for _ in range(n)]
b = [r.choice([0, r.randint(0, top)]) for _ in range(n)]
c = r.randrange(n)
a[c] = max(a[c], 1); b[c] = max(b[c], 1)
print(n)
print(*a)
print(*b)

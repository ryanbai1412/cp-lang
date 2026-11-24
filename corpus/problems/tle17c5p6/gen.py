"""TLE '17 C5 P6 inputs: `gen.py seed size`. size < 1e5 gives n <= 3 size, m <= 6 size; size >= 1e5
gives n = 1e5, m = 2e5 picked by seed % 3: random gates over all lower wires; a deep chain (each
gate reads the previous one, recursion depth m); or gates reading recent wires (output depends on x)."""
import sys, random
if sys.argv[1] == "--max":
    sys.argv[1:] = ["1", "100000"]

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S < 100000:
    n, m = r.randint(1, 3 * S), r.randint(1, 6 * S); mode = r.randint(0, 2)
else:
    n, m = 100000, 200000; mode = int(sys.argv[1]) % 3
ins = []
for g in range(1, m + 1):
    top = n + g - 1
    if mode == 0:
        a, b = r.randint(1, top), r.randint(1, top)
    elif mode == 1:
        a, b = top, r.randint(1, top)
    else:
        lo = max(1, top - 3); a, b = r.randint(lo, top), r.randint(1, top) if r.random() < 0.3 else r.randint(lo, top)
    ins += [a, b]
print(n, m)
print(*ins)

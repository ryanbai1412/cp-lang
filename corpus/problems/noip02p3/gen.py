import sys, random
# seed size: H, S1, V, L, K reals of one decimal and n, scaled by size; --max: n = 1e5 balls with
# a car that sweeps past all of them while they are in reach (every ball counted)
mx = sys.argv[1] == "--max"
if mx:
    print("100000.0 100000.0 100000.0 100000.0 100000.0 100000"); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
cap = min(100000, 10 * S)
v = lambda: r.randint(10, cap * 10) / 10
print(f"{v():.1f} {v():.1f} {v():.1f} {v():.1f} {v():.1f} {r.randint(1, cap)}")

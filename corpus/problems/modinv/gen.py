import sys, random, math
if sys.argv[1:] == ["--max"]:
    # the largest consecutive Fibonacci pair with M <= 1e19: the most Euclid steps
    fib = [1, 2]
    while fib[-1] + fib[-2] <= 10 ** 19: fib.append(fib[-1] + fib[-2])
    print(fib[-2], fib[-1])
    sys.exit()
# gen.py seed size: 0 <= N < M <= min(size, 1e19), gcd(N, M) = 1.
# No arguments, or size >= 1e19, uses the full u64 range; the seed picks the shape:
# consecutive Fibonacci numbers (the most Euclid steps), M near 1e19, N = 0 (M = 1), random.
seed = int(sys.argv[1]) if len(sys.argv) > 1 else 0
S = min(int(sys.argv[2]) if len(sys.argv) > 2 else 10 ** 19, 10 ** 19)
r = random.Random(seed)
kind = seed % 6
if kind == 0 and S >= 2:
    fib = [1, 2]
    while fib[-1] + fib[-2] <= S: fib.append(fib[-1] + fib[-2])
    i = r.randint(1, len(fib) - 1); n, m = fib[i - 1], fib[i]
elif kind == 1 or S < 2:
    n, m = 0, 1
else:
    while True:
        m = r.randint(max(2, S - 1000), S) if kind == 2 else r.randint(2, S)
        n = r.choice([1, m - 1, r.randint(1, m - 1)])
        if math.gcd(n, m) == 1: break
print(n, m)

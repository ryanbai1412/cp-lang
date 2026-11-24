import sys, random
# seed size: N in 6..40 digits (leading digit 1..9) scaled by size, K in 1..min(6, N - 1);
# --max: N = 40 nines, K = 6 (products near 10^40, past 64 and 128 bits)
mx = sys.argv[1] == "--max"
if mx:
    print(40, 6); print("9" * 40); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(6, min(40, S + 6)); k = r.randint(1, min(6, n - 1))
print(n, k); print(str(r.randint(1, 9)) + "".join(r.choice("0123456789" if r.random() < 0.8 else "0") for _ in range(n - 1)))

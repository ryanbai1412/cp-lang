import sys, random
# statement: N <= 2e6 numbers 1 <= J_i <= 5e6.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "2000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 2 * 10 ** 6
n = 2 * 10 ** 6 if big else r.randint(1, max(1, min(2 * 10 ** 6, S)))
J = 5 * 10 ** 6 if big else r.choice([10, 60, 5 * 10 ** 6])
print(n)
print(*(r.randint(1, J) for _ in range(n)))

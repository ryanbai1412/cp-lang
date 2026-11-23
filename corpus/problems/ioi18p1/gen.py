import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: 1 <= N <= 2000, S over A B X Y, S[0] never appears again. The sample grader reads S.
# Size S grows N; tails are random, one repeated letter, or two alternating letters.
n = r.randint(1, min(2000, 2 + 3 * S)) if S < 1000 else 2000 - r.randrange(3)
first = r.choice("ABXY")
rest = [c for c in "ABXY" if c != first]
mode = r.choice(["random", "random", "same", "alternate"])
if mode == "same":
    tail = r.choice(rest) * (n - 1)
elif mode == "alternate":
    p, q = r.sample(rest, 2)
    tail = "".join(p if i % 2 else q for i in range(n - 1))
else:
    tail = "".join(r.choice(rest) for _ in range(n - 1))
print(first + tail)

import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "250"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = S if big else r.randint(1, min(S, 250))
top = 26 if big else r.choice([2, 5, 26])
print(n)
print("".join(chr(65 + (25 if big and r.random() < 0.8 else r.randrange(top))) for _ in range(n)))

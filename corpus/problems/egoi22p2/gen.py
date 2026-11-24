import sys, random
# statement: 1 <= w, h with w * h <= 5e5.
if sys.argv[1] == "--max":
    print(10000, 50)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S >= 300 and r.random() < 0.5:
    w = r.randint(10001, 20000); h = r.randint(1, 500000 // w)
else:
    w = r.randint(1, max(1, min(S, 12))); h = r.randint(1, max(1, min(S, 8)))
print(w, h)

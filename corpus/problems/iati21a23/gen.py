import sys, random
# port: 1 <= n <= 1000 cells, 0 <= s, c <= 1000 turns fleeing and approaching, s + c >= 1.
if sys.argv[1] == "--max":
    print(1000, 1000, 999)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, max(1, min(S, 1000)))
top = max(1, min(S, 1000))
while True:
    s, c = r.randint(0, top), r.randint(0, top)
    if s + c >= 1:
        break
print(n, s, c)

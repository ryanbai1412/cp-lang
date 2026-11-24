import sys, random
# statement: 1 <= n <= 500 distinct names of 1..10 lowercase letters, five scores in 0..=500.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "500"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 500 if S >= 500 else r.randint(1, max(1, min(S, 500)))
names = set()
while len(names) < n:
    names.add("".join(r.choice("abc" if n < 10 else "abcdefghijklmnopqrstuvwxyz") for _ in range(r.randint(1, 10))))
top = r.choice([0, 1, 500]) if S < 500 else 500
print(n)
for s in names:
    print(s, *(r.randint(max(0, top - 2), top) if r.random() < 0.5 else r.randint(0, top) for _ in range(5)))

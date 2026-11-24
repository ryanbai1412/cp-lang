import sys, random
# statement: 1 <= n <= 100 commands, the first a section, a subsection only inside a section and a
# subsubsection only inside a subsection; titles are 1..20 lowercase letters.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 100 if S >= 100 else r.randint(1, max(1, min(S, 100)))
depth = 0
print(n)
for i in range(n):
    d = 1 if i == 0 else r.randint(1, min(3, depth + 1))
    depth = d
    title = "".join(r.choice("abcdefghijklmnopqrstuvwxyz") for _ in range(20 if S >= 100 else r.randint(1, 20)))
    print("sub" * (d - 1) + "section", title)

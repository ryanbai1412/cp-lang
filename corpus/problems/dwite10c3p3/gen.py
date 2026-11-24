import sys, random
# statement: five lines, each a width 0 <= N <= 30.
if sys.argv[1] == "--max":
    print("\n".join(["30", "29", "0", "1", "30"]))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print("\n".join(str(r.randint(0, min(30, S))) for _ in range(5)))

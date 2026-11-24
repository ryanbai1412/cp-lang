import sys, random
if sys.argv[1] == "--max":
    print(3, 8, 2); print(("a-z" + "0-9") * 16 + "a-z-"); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(1, 3), r.randint(1, 8), r.randint(1, 2))
print(''.join(r.choice("--abdx159") for _ in range(r.randint(1, 4 * S))))

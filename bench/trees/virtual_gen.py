import random, sys
n = int(sys.argv[1]); q = int(sys.argv[2]); k = int(sys.argv[3]); seed = int(sys.argv[4]) if len(sys.argv) > 4 else 1
random.seed(seed)
lab = list(range(1, n + 1)); random.shuffle(lab)
out = [str(n)]
for i in range(1, n):
    p = random.randint(max(0, i - 10), i - 1) if seed % 2 == 0 else random.randint(0, i - 1)
    out.append(f"{lab[i]} {lab[p]}")
out.append(str(q))
for _ in range(q):
    out.append(f"{k} " + " ".join(str(random.randint(1, n)) for _ in range(k)))
sys.stdout.write("\n".join(out) + "\n")

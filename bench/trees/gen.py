import random, sys
n = int(sys.argv[1]); q = int(sys.argv[2]); seed = int(sys.argv[3]) if len(sys.argv) > 3 else 1
random.seed(seed)
lab = list(range(1, n + 1)); random.shuffle(lab)
out = [str(n)]
for i in range(1, n):
    p = random.randint(max(0, i - 10), i - 1) if seed % 2 == 0 else random.randint(0, i - 1)
    out.append(f"{lab[i]} {lab[p]}")
out.append(str(q))
for _ in range(q):
    out.append(f"{random.randint(1, n)} {random.randint(1, n)}")
sys.stdout.write("\n".join(out) + "\n")

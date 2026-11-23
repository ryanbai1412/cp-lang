import random
random.seed(1)
n = 200000
print(1); print(n, 50)
pool = list(range(1, n + 1)); nxt = n + 1; out = []
for _ in range(n - 1):
    i = random.randrange(len(pool)); pool[i], pool[-1] = pool[-1], pool[i]; x = pool.pop()
    j = random.randrange(len(pool)); pool[j], pool[-1] = pool[-1], pool[j]; y = pool.pop()
    out.append(f"{x} {y}"); pool.append(nxt); nxt += 1
print("\n".join(out))

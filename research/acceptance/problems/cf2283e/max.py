import random
random.seed(1)
n, t = 5000, 500
print(t)
for i in range(t):
    k = i % 6
    if k == 0: s = "0" * n
    elif k == 1: s = "01" * (n // 2)
    elif k == 2: s = "".join(random.choice("01") for _ in range(n))
    elif k == 3: s = "".join("01"[j % 2] if random.random() < 0.9 else "1" for j in range(n))
    elif k == 4: s = "1" * (n // 2) + "0" * (n // 2)
    else: s = "".join(random.choice(["0", "1", "00", "11", "010"]) for _ in range(n))[:n].ljust(n, "1")
    print(n); print(s)

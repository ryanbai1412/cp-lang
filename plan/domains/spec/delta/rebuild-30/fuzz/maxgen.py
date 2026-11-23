import random, os
os.makedirs(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'max'), exist_ok=True)
r = random.Random(1)
W = lambda n, s: open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'max', f'{n}.in'), 'w').write(s)
n = 10**5; W('01-1918D', f"1\n{n}\n" + " ".join(str(r.randint(1, 10**9)) for _ in range(n)) + "\n")
n = 10**6; p = list(range(n)); r.shuffle(p); W('02-1905D', f"1\n{n}\n{' '.join(map(str, p))}\n")
n = 1000; es = [(v - 1, v, r.randint(1, 10**5)) for v in range(2, n + 1)] + [(r.randint(1, n), r.randint(1, n), r.randint(1, 10**5)) for _ in range(1)]
es = [(u, v, w) if u != v else (1, 2, w) for u, v, w in es]
s = list(range(1000, 0, -1))
W('03-1915G', f"1\n{n} {len(es)}\n" + "".join(f"{u} {v} {w}\n" for u, v, w in es) + " ".join(map(str, s)) + "\n")
n = 3 * 10**5; q = 3 * 10**5; free = list(range(2, n)); r.shuffle(free); qs = []
for i in range(q):
    if i % 2 == 0 and free: qs.append(f"1 {free.pop()} {r.randint(1, 10**7)}")
    else: qs.append(f"2 1 {n}")
W('04-1924B', f"{n} 2 {q}\n1 {n}\n{10**7} {10**7}\n" + "\n".join(qs) + "\n")
n = 10**6; W('05-1930D2', f"1\n{n}\n" + "".join(r.choice("01") for _ in range(n)) + "\n")
n = 10**5; k = 20
es = "".join(f"{v - 1} {v}\n" for v in range(2, n + 1)); esc = "".join(f"{v - 1} {v} {r.randint(1, 10**9)}\n" for v in range(2, n + 1))
ps = "".join(f"{r.randint(1, n)} {r.randint(1, n)}\n" for _ in range(k))
W('07-1929E', f"1\n{n}\n{es}{k}\n{ps}"); W('07-1929E-rev', f"1\n{n}\n{esc}{k}\n{ps}")
n = 10**5; m = 10**5; pairs = set()
while len(pairs) < m:
    a, b = r.sample(range(1, n + 1), 2); pairs.add((a, b) if a < b else (b, a))
W('08-1925D', f"1\n{n} {m} {2 * 10**5}\n" + "".join(f"{a} {b} {r.randint(1, 10**9)}\n" for a, b in pairs))
W('09-1948D', "1\n" + "".join(r.choice("ab") for _ in range(5000)) + "\n")
n = 3 * 10**5; W('10-1923D', f"1\n{n}\n" + " ".join(str(r.randint(1, 10**9)) for _ in range(n)) + "\n")
W('10-1923D-eq', f"1\n{n}\n" + " ".join("1" for _ in range(n)) + "\n")

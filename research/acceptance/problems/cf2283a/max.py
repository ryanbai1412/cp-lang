import random
random.seed(1)
cases = [(-100, 100), (-100, -100), (100, 100), (0, 0), (-100, 99), (-99, 100), (0, 100), (-100, 0), (1, 100), (-100, -1)]
cases += [(l, 100) for l in range(-100, 101)] + [(-100, r) for r in range(-100, 101)]
while len(cases) < 10**4:
    l, r = sorted((random.randint(-100, 100), random.randint(-100, 100)))
    if random.random() < 0.5: l, r = -random.randint(60, 100), random.randint(60, 100)
    cases.append((l, r))
print(len(cases))
for l, r in cases: print(l, r)

import random
random.seed(1)
# sum n = 3e5: one bad-heavy string, random over 0256, all '2', random digits, plus many tiny tests
tests = [("2025" * 25000)[:100000], "".join(random.choice("0256") for _ in range(80000)), "2" * 50000,
         "".join(random.choice("0123456789") for _ in range(34000)), "202" * 6000 + "5" * 12000]
tests += ["".join(random.choice("0256") for _ in range(2)) for _ in range(3000)]
print(len(tests))
for s in tests: print(len(s)); print(s)

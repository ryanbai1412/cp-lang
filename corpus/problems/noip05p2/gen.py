import sys, random
# seed size: bridge length, jump range s..=t and m distinct stones; small cases keep the
# gaps short, others mix gaps longer than the 2520 cycle; s == t appears often
if sys.argv[1] == "--max":
    r = random.Random(0); L = 10**9
    print(L); print(1, 10, 100); print(*sorted(r.sample(range(1, L), 100))); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
L = r.choice([r.randint(2, 30 * S + 2), r.randint(2, 10**9)])
s = r.randint(1, 10); t = s if r.random() < .3 else r.randint(s, 10)
m = r.randint(1, min(100, L - 1, S + 1))
print(L); print(s, t, m); print(*r.sample(range(1, L), m))

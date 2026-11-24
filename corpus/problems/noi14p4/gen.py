import sys, random
if sys.argv[1:] == ["--max"]:
    # 5 strings of 1e6: all 'a' (deepest border chains) and near-periodic ones that make `half`
    # fall back often
    r = random.Random(0)
    print(5)
    print("a" * 10 ** 6)
    print("a" * 10 ** 6)
    print(("a" * 999 + "b") * 1000)
    print(("aab" * 333334)[:10 ** 6])
    print("".join(r.choice("ab" if r.randint(0, 2) else "a") for _ in range(10 ** 6)))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t): print(''.join(r.choice("ab" if r.randint(0, 2) else "a") for _ in range(r.randint(1, 5 * S))))

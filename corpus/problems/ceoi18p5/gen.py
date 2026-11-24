"""CEOI 2018 Toys inputs: `gen.py seed size` (n a product of small primes below 10^size, or any
n up to it), or `gen.py --max` (n = 735134400, the n <= 1e9 with the most divisors, 1344)."""
import sys, random
if sys.argv[1:] == ["--max"]:
    print(735134400)
    sys.exit()
r = random.Random(int(sys.argv[1])); cap = 10 ** min(9, max(1, int(sys.argv[2])))
if r.random() < .3:
    print(r.randint(1, cap))
else:
    n = 1
    while True:
        p = r.choice([2, 2, 2, 3, 3, 5, 7, 11, 13])
        if n * p > cap: break
        n *= p
    print(n)

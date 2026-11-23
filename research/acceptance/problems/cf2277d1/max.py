import random
random.seed(1)
E = 10**18
# sum of r - l + 1 is 2*10^5 exactly: the top of the range (most bits set), just below 2^59
# (popcount jumps), a random block, then 9996 one- and few-value tests at random large n.
qs = [(E - 59999, E), (2**59 - 30000, 2**59 + 29999), (random.randint(0, E - 40000),) * 1]
qs[2] = (qs[2][0], qs[2][0] + 39999)
left = 200000 - sum(r - l + 1 for l, r in qs)
K = 10**4 - len(qs)
for i in range(K):
    w = left // (K - i); left -= w
    l = random.randint(0, E - w + 1) if i % 3 else (1 << random.randint(1, 59)) - 1
    qs.append((l, l + w - 1))
print(len(qs))
for l, r in qs: print(l, r)

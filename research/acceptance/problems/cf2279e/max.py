import random
random.seed(1)
M = 1 << 25
def independent(k, lo=1):
    basis, out = [], []
    while len(out) < k:
        x = random.randint(lo, M - 1); y = x
        for b in basis: y = min(y, y ^ b)
        if y: basis.append(y); out.append(x)
    return out
# a shuffled basis repeated: the only zero sums pair copies 25 apart, so the answer is the maximum 49
b = independent(25); rep = (b * 3)[:60]
print(3)
print(60); print(*rep)
print(25); print(*independent(25, M // 2))  # full rank: -1 after a full search
print(15); print(*[M - 1] * 15)

from fractions import Fraction
n, m, d = map(int, input().split())
a = list(map(int, input().split()))

p = Fraction(1)
csize = sum(a)
for _ in range(m):
    i = int(input().split()[1]) - 1
    p *= Fraction(a[i], csize)
    a[i] += d
    csize += d

print(f"{p.numerator}/{p.denominator}")
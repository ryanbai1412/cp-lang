type Z = Mod<998244353>
var a: Z = 1
for i in 1..=3:
    a += Z(i) ** 2
print(a)

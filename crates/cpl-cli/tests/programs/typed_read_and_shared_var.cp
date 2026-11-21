type Z = Mod<998_244_353>

var ones, one_twos, ans: Z = (0, 0, 0)
let n = read::<Int>()
for _ in 0..n:
    match read::<1..=3>():
        1 => ones += 1
        2 => one_twos = one_twos * 2 + ones
        3 => ans += one_twos
print(ans)

var x, y: 0..=100 = (5, 5)
x += read::<1..=10>()
y += read::<1..=10>()
print(x, y)

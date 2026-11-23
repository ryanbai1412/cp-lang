// CF 1994C Hungry Games
// in: t; per test n x, a_1..a_n. out: the number of subsegments [l, r] that end with g != 0.
@!multitest
let (n, x): (1..=2e5, 1..=1e9) = read()
let a = read<[n]1..=1e9>()
let pre = a.scan(+)
var good: [n + 1]Int = 0
for i in (0..n).rev:
    good[i] = match (i..n).first_true(j => pre.fold(i..=j) > x):
        Some(j) => j - i + good[j + 1]
        None => n - i
print(good.sum())

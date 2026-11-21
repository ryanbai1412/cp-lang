// Codeforces 2069 C: Beautiful Sequence.
type Z = Mod<998_244_353>

@!multitest
let n: Int = read()
var ones: Z = 0
var one_twos: Z = 0
var ans: Z = 0
for _ in 0..n:
    let x: 1..=3 = read()
    match x:
        1 => ones += 1
        2 => one_twos = one_twos * 2 + ones
        3 => ans += one_twos
print(ans)

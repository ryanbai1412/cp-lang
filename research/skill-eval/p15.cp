let (n, x) = read<(1..=1000, 1..=1e5)>()
domain B = 0..n
let h = read<[B]1..=1000>()
let s = read<[B]1..=1000>()
domain C = 0..=x
var dp: [C]Int = 0
for b in B:
    for c in C.rev:
        if let prev = C.at(Int(c) - h[b]):
            dp[c] max= dp[prev] + s[b]
print(dp[C(x)])

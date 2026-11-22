// ABC480 F — Cake Cutting
// Bitmask DP (N <= 20): Subset<I> + submask @alt(program)

let (n, m): (1..=20, 1..=100) = read()
domain I = 0..n
let a: [I]Int = read()

let total = sum(a[i] for i in I)
let target = total / m

var rem: [Subset<I>]Int = 0
for s in Subset<I>:
    if let i = s.first:
        rem[s] = (rem[s.without(i)] + a[i]) % target

var dp: [Subset<I>]Int = 0
for s in Subset<I>:
    for i in Subset<I>.full - s:
        let ns = s.with(i)
        let add = Int(rem[ns] == 0)
        dp[ns] max= dp[s] + add

print(m - dp[Subset<I>.full])

@alt(program)
fn slow():
    let tgt = sum(a[i] for i in I) / m
    var ssum: [Subset<I>]Int = 0
    for s in Subset<I>:
        if let i = s.first:
            ssum[s] = ssum[s.without(i)] + a[i]
    var best: [Subset<I>]Int = -1
    best[Subset<I>.empty] = 0
    for s in Subset<I>:
        if s == Subset<I>.empty or ssum[s] % tgt != 0:
            continue
        for sub in s.subsets:
            if sub != Subset<I>.empty and ssum[sub] % tgt == 0:
                let rest = s - sub
                if best[rest] >= 0:
                    best[s] max= best[rest] + 1
    print(m - best[Subset<I>.full])

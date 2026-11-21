// AtCoder DP N Slimes: merge adjacent slimes, paying the merged size, until
// one is left. The last merge of a[l..=r] joins a[l..=m] and the rest.
let n: 1..=400 = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()
let size = prefix(a)

@memo
fn cost(l: I, r: I) -> Int =
    if l == r: 0 else: min(cost(l, m) + cost(m + 1, r) for m in l..r)! + size.sum(l..=r)
print(cost(I.first, I.last))

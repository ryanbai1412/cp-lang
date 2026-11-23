// AtCoder DP N Slimes: merge adjacent slimes, paying the merged size, until
// one is left. cost(l, r) = best split + total size of a[l..=r].
let n: 1..=400 = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()

let pre = prefix(a)

@memo
fn cost(l: I, r: I) -> Int:
    if l == r: return 0
    min(cost(l, m) + cost(m.next!, r) for m in l..r)! + pre.sum(l..=r)
print(cost(I.first!, I.last!))

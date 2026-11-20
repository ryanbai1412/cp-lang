let (n, k): (1..=10, 1..=10) = read()
domain V = 1..=n
let a: [V]Int = read()
var cnt: [0..=k]Int = 0
cnt[1] = 7
fn total(c: [0..=k]Int) -> Int:
    return sum(c[i] for i in 0..=k)
let t = total(cnt)
let b: [V]Int = [V] v => a[v] * 2
var seen: [V]?Int = 1
seen.fill(none)
let c0 = cnt[0]
cnt.fill(2)
print(b[V(1)], b[V(2)], b[V(3)], c0, t, t - 7, cnt[k], seen[V(1)], none: "none")

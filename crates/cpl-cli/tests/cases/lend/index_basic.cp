let n: 1..=9 = read()
domain V = 1..=n
let a: [V]0..=100 = read()

// v.l / v.r are the boundaries on either side of v; prefix sums live on boundaries
var pre: [V.cuts]Int = 0
for v in V: pre[v.r] = pre[v.l] + a[v]
fn sum(l: V, r: V) -> Int:
    pre[r.r] - pre[l.l]
print(sum(V.first!, V.last!), sum(V(2), V(3)))

// v.l(k) / v.r(k): the far boundary of the k-wide window that ends / starts at v
let v3 = V(3)
print(pre[v3.r] - pre[v3.l(2)], pre[v3.r(2)] - pre[v3.l], pre[v3.r(1)] - pre[v3.l(1)])

// the Index surface of a domain
print(V.len, v3.to_idx, yes(V.at(0) == V.first))

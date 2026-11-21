@!multitest
domain I = read(1..=1e6)
domain V = 1..=I.count
let a: [I]V = read()

// "5" is the next greater after "3", "4" the latest earlier value between them,
// "1" the next smaller after "2"; best[r] is the latest left end of a 21435 ending by r
var shapes: [I]List<(V, I)> = []
var latest = SegTree<V, ?I>(max)
for i in I:
    for j in a.pops(at: i, by: <):
        if let k = latest.fold(a[j] <.. a[i]): shapes[k].push((a[j], i))
    latest[a[i]] = i

var two = SegTree<V, ?I>(max)
var best: [I]?I = none
for i in I:
    for (v, r) in shapes[i]: best[r] max= two.fold(..v)
    for j in a.pops(at: i, by: >): two[a[j]] = j

print(sum(l.to_idx + 1 for l in scan(best, max).present))

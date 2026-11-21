// Codeforces 2165 F: Arctic Acquisition.
// Count subarrays containing a 21435 pattern.
// `best[r]` is the latest left end of a 21435 ending by `r`; exchange arguments
// fix "5" as the next greater after "3", "4" as the latest earlier element
// valued strictly between them, and "1" as the next smaller after "2".

@!multitest
domain I = read(1..=1_000_000)
domain V = 1..=I.count
let a: [I]V = read()

// shapes[k]: ("3" value, "5" position) of each "435" whose "4" is at k.
var shapes: [I]List<(V, I)> = []
var latest = SegTree<V, ?I>(max)
let next = a.next_greater()
for j in I:
    if let i = next[j], let k = latest.fold(a[j] <..a[i]): shapes[k].push((a[j], i))
    latest.set(a[j], j)

// two: the latest "2" per value whose "1" (next smaller) came before i.
var two = SegTree<V, ?I>(max)
var best: [I]?I = none
let freed = a.next_smaller().inverse()
for i in I:
    for (v, r) in shapes[i]: best[r] max= two.fold(..v)
    for j in freed[i]: two.set(a[j], j)

print(sum(l.ord + 1 for l in best.scan(max).present))

// For each right end r, best[r] is the latest left end of a 21435 pattern ending by r;
// the answer is the sum of best + 1 after a prefix max.
// Exchange arguments let "5" be the next greater element after "3", "4" the latest earlier
// element valued between them, and "1" the next smaller element after "2".
@!multitest
let n: 1..=1_000_000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()

// shapes[k]: (value of "3", position of "5") for each "435" whose "4" is at k;
// every value between a[j] and a[i] sits before j, since i is j's next greater
var shapes: [I]List<(V, I)> = []
var latest = SegTree<V, ?I>(max)
let next = a.next_greater()
for j in I:
    if let i = next[j], let k = latest.fold((a[j] + 1)!..a[i]): shapes[k].push((a[j], i))
    latest.set(a[j], j)

// two[v]: position of a "2" valued v whose "1" lies before the current position
var two = SegTree<V, ?I>(max)
var best: [I]?I = none
var stack: List<I> = []
for i in I:
    for (v, r) in shapes[i]: best[r] max= two.fold(..v)
    while let j = stack.last, a[i] < a[j]:
        stack.pop()
        two.set(a[j], j)
    stack.push(i)

print([l.to_idx + 1 for l in scan(best, max) if let l = l].sum())

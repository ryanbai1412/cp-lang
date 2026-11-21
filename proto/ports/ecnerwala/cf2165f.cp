// A subarray is jagged when it holds a 21435 pattern, so for each right end r it suffices to
// know the latest left end best[r] of a pattern ending by r: the answer is sum(best + 1).
// Exchange arguments let "5" be the next greater element after "3", "4" the latest earlier
// element valued between them, and "1" the next smaller element after "2".
@!multitest
let n: Int = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()

// shapes[k]: (value of "3", position of "5") for each "435" whose "4" is at k
var shapes: [I]List<(V, I)> = [I] _ => []
var latest = SegTree<V, ?I>(max)
var stack: List<I> = []
for i in I:
    while let j = stack.last, a[j] < a[i]:
        stack.pop()
        if let k = latest.fold((a[j] + 1)!..a[i]):
            shapes[k].push((a[j], i))
    stack.push(i)
    latest.set(a[i], i)

// two[v]: position of a "2" valued v whose "1" lies before the current position
var two = SegTree<V, ?I>(max)
var best: [I]?I = none
stack = []
for i in I:
    for (v, r) in shapes[i]:
        best[r] max= two.fold(..v)
    while let j = stack.last, a[i] < a[j]:
        stack.pop()
        two.set(a[j], j)
    stack.push(i)

var ans = 0
var left: ?I = none
for r in I:
    left max= best[r]
    if let l = left:
        ans += l.to_idx + 1
print(ans)
